//! Serviço root, ativado por socket: entra no Tenancy, abre chamado e lê respostas em nome da máquina.
//! O token da máquina nunca sai daqui. A conta que pediu vem do SO_PEERCRED, nunca do cliente.
use base64::Engine;
use novasce_casa::multipart::{corpo, fronteira_nova, Parte};
use novasce_casa::nucleo::{ler_maquina, ErroTenancy, Tenancy};
pub use novasce_casa::sinal_local::{PedidoLocal, RespostaLocal};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const LIMITE_PRINT: usize = 5 * 1024 * 1024;

pub struct Contexto {
    pub tenancy_api: String,
    pub arquivo_maquina: PathBuf,
    cache_maquina: Mutex<Option<(Instant, serde_json::Value)>>,
    ultimos_fluxos: Mutex<HashMap<u32, Instant>>,
}

impl Contexto {
    pub fn novo(tenancy_api: String, arquivo_maquina: PathBuf) -> Contexto {
        Contexto {
            tenancy_api,
            arquivo_maquina,
            cache_maquina: Mutex::new(None),
            ultimos_fluxos: Mutex::new(HashMap::new()),
        }
    }

    fn tenancy(&self) -> Option<Tenancy> {
        let t = ler_maquina(&self.arquivo_maquina).ok()?;
        Some(Tenancy::new(&self.tenancy_api, Some(t.machine_token)))
    }
}

fn erro(e: ErroTenancy) -> RespostaLocal {
    match e {
        ErroTenancy::Status { codigo, mensagem } => RespostaLocal::Erro { codigo, mensagem },
        ErroTenancy::SemRede(m) => RespostaLocal::Erro {
            codigo: 0,
            mensagem: m,
        },
        ErroTenancy::Formato(m) => RespostaLocal::Erro {
            codigo: 502,
            mensagem: m,
        },
    }
}

fn ok(dados: serde_json::Value) -> RespostaLocal {
    RespostaLocal::Ok { dados }
}

fn id_valido(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn atender(ctx: &Contexto, uid: u32, conta: &str, pedido: PedidoLocal) -> RespostaLocal {
    let Some(t) = ctx.tenancy() else {
        return RespostaLocal::Erro {
            codigo: 412,
            mensagem: "Esta máquina ainda não foi registrada no Tenancy.".into(),
        };
    };
    match pedido {
        PedidoLocal::Maquina => {
            if let Some((quando, v)) = ctx.cache_maquina.lock().unwrap().as_ref() {
                if quando.elapsed() < Duration::from_secs(600) {
                    return ok(v.clone());
                }
            }
            match t.get::<serde_json::Value>("/os/machine") {
                Ok(v) => {
                    *ctx.cache_maquina.lock().unwrap() = Some((Instant::now(), v.clone()));
                    ok(v)
                }
                Err(e) => erro(e),
            }
        }
        PedidoLocal::Entrar => {
            let mut f = ctx.ultimos_fluxos.lock().unwrap();
            if f.get(&uid)
                .is_some_and(|q| q.elapsed() < Duration::from_secs(5))
            {
                return RespostaLocal::Erro {
                    codigo: 429,
                    mensagem: "Espere alguns segundos e tente de novo.".into(),
                };
            }
            f.insert(uid, Instant::now());
            drop(f);
            t.post::<_, serde_json::Value>("/os/auth/device", &serde_json::json!({}))
                .map(ok)
                .unwrap_or_else(erro)
        }
        PedidoLocal::EntrarToken { device_code } => {
            match t.post::<_, serde_json::Value>(
                "/os/auth/device/token",
                &serde_json::json!({ "deviceCode": device_code }),
            ) {
                Ok(v) => ok(v),
                Err(ErroTenancy::Status { codigo: 428, .. }) => RespostaLocal::Pendente,
                Err(e) => erro(e),
            }
        }
        PedidoLocal::Chamado {
            texto,
            machine_info,
            print,
        } => {
            let mut info = match machine_info {
                serde_json::Value::Object(m) => m,
                _ => serde_json::Map::new(),
            };
            info.insert("conta".into(), serde_json::Value::String(conta.into()));
            let info = serde_json::Value::Object(info).to_string();
            let png = match print.map(|b| base64::engine::general_purpose::STANDARD.decode(b)) {
                Some(Ok(b)) if b.len() <= LIMITE_PRINT => Some(b),
                Some(Ok(_)) => {
                    return RespostaLocal::Erro {
                        codigo: 413,
                        mensagem: "O print passou de 5 MB.".into(),
                    }
                }
                Some(Err(_)) => {
                    return RespostaLocal::Erro {
                        codigo: 400,
                        mensagem: "Print ilegível.".into(),
                    }
                }
                None => None,
            };
            let mut partes = vec![
                Parte::Texto {
                    nome: "text",
                    valor: &texto,
                },
                Parte::Texto {
                    nome: "machineInfo",
                    valor: &info,
                },
            ];
            if let Some(b) = png.as_deref() {
                partes.push(Parte::Arquivo {
                    nome: "screenshot",
                    arquivo: "print.png",
                    tipo: "image/png",
                    dados: b,
                });
            }
            let (tipo, b) = corpo(&partes, &fronteira_nova());
            t.post_bytes("/os/tickets", &tipo, &b)
                .map(ok)
                .unwrap_or_else(erro)
        }
        PedidoLocal::Respostas { since } => {
            let caminho = match since {
                Some(s) => format!("/os/tickets/mine?since={}", s.replace('+', "%2B")),
                None => "/os/tickets/mine".into(),
            };
            t.get::<serde_json::Value>(&caminho)
                .map(ok)
                .unwrap_or_else(erro)
        }
        PedidoLocal::Retorno {
            ticket_id,
            resolved,
        } => {
            if !id_valido(&ticket_id) {
                return RespostaLocal::Erro {
                    codigo: 400,
                    mensagem: "Chamado inválido.".into(),
                };
            }
            t.post_sem_resposta(
                &format!("/os/tickets/{ticket_id}/feedback"),
                &serde_json::json!({ "resolved": resolved }),
            )
            .map(|_| ok(serde_json::Value::Null))
            .unwrap_or_else(erro)
        }
    }
}

/// Uma linha de pedido entra, uma linha de resposta (com `\n`) sai. Linha ilegível vira 400.
pub fn responder(ctx: &Contexto, uid: u32, conta: &str, linha: &str) -> String {
    let resposta = match serde_json::from_str::<PedidoLocal>(linha) {
        Ok(p) => atender(ctx, uid, conta, p),
        Err(_) => RespostaLocal::Erro {
            codigo: 400,
            mensagem: "Pedido ilegível.".into(),
        },
    };
    let mut saida = serde_json::to_string(&resposta).unwrap_or_default();
    saida.push('\n');
    saida
}

#[cfg(test)]
mod testes {
    use super::*;
    use mockito::Matcher;

    fn ctx(url: &str, token: Option<&str>) -> (tempfile::TempDir, Contexto) {
        let d = tempfile::tempdir().unwrap();
        let arq = d.path().join("maquina.json");
        if let Some(t) = token {
            use std::os::unix::fs::PermissionsExt;
            std::fs::write(
                &arq,
                format!("{{\"machineId\":\"m1\",\"machineToken\":\"{t}\"}}"),
            )
            .unwrap();
            // a guarda do M1 recusa token aberto para outros usuários
            std::fs::set_permissions(&arq, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        (d, Contexto::novo(url.into(), arq))
    }

    #[test]
    fn sem_registro_responde_412() {
        let (_d, c) = ctx("http://127.0.0.1:9", None);
        let r = atender(&c, 1000, "luan", PedidoLocal::Entrar);
        assert!(matches!(r, RespostaLocal::Erro { codigo: 412, .. }));
    }

    #[test]
    fn entrar_usa_token_da_maquina() {
        let mut s = mockito::Server::new();
        let m = s
            .mock("POST", "/os/auth/device")
            .match_header("authorization", "Bearer nvos_m_teste")
            .with_status(200)
            .with_body(r#"{"userCode":"ABCD-EFGH","verificationUri":"https://t/os/entrar","deviceCode":"dc","interval":5,"expiresIn":600}"#)
            .create();
        let (_d, c) = ctx(&s.url(), Some("nvos_m_teste"));
        let r = atender(&c, 1000, "luan", PedidoLocal::Entrar);
        m.assert();
        assert!(matches!(r, RespostaLocal::Ok { .. }));
    }

    #[test]
    fn token_pendente_428_vira_pendente() {
        let mut s = mockito::Server::new();
        s.mock("POST", "/os/auth/device/token")
            .with_status(428)
            .with_body(r#"{"statusCode":428,"message":"pendente"}"#)
            .create();
        let (_d, c) = ctx(&s.url(), Some("nvos_m_teste"));
        let r = atender(
            &c,
            1000,
            "luan",
            PedidoLocal::EntrarToken {
                device_code: "dc".into(),
            },
        );
        assert_eq!(r, RespostaLocal::Pendente);
    }

    #[test]
    fn token_recusado_ou_vencido_volta_com_o_codigo() {
        let mut s = mockito::Server::new();
        s.mock("POST", "/os/auth/device/token")
            .with_status(410)
            .with_body(r#"{"statusCode":410,"message":"expirado"}"#)
            .create();
        let (_d, c) = ctx(&s.url(), Some("nvos_m_teste"));
        let r = atender(
            &c,
            1000,
            "luan",
            PedidoLocal::EntrarToken {
                device_code: "dc".into(),
            },
        );
        assert_eq!(
            r,
            RespostaLocal::Erro {
                codigo: 410,
                mensagem: "expirado".into()
            }
        );
    }

    #[test]
    fn chamado_leva_a_conta_que_pediu_e_nao_a_que_o_cliente_disse() {
        let mut s = mockito::Server::new();
        let m = s
            .mock("POST", "/os/tickets")
            .match_body(Matcher::AllOf(vec![
                Matcher::Regex("name=\"text\"\r\n\r\nNão imprime".into()),
                Matcher::Regex("\"conta\":\"luan\"".into()),
            ]))
            .with_status(201)
            .with_body(r#"{"ticketId":"128"}"#)
            .create();
        let (_d, c) = ctx(&s.url(), Some("nvos_m_teste"));
        let info = serde_json::json!({"conta":"outra","app":"Arquivos"});
        let r = atender(
            &c,
            1000,
            "luan",
            PedidoLocal::Chamado {
                texto: "Não imprime".into(),
                machine_info: info,
                print: None,
            },
        );
        m.assert();
        assert_eq!(
            r,
            RespostaLocal::Ok {
                dados: serde_json::json!({"ticketId":"128"})
            }
        );
    }

    #[test]
    fn retorno_recusa_id_estranho() {
        let (_d, c) = ctx("http://127.0.0.1:9", Some("nvos_m_teste"));
        let r = atender(
            &c,
            1000,
            "luan",
            PedidoLocal::Retorno {
                ticket_id: "../x".into(),
                resolved: true,
            },
        );
        assert!(matches!(r, RespostaLocal::Erro { codigo: 400, .. }));
    }

    #[test]
    fn sem_rede_vira_codigo_zero() {
        let (_d, c) = ctx("http://127.0.0.1:9", Some("nvos_m_teste"));
        let r = atender(&c, 1000, "luan", PedidoLocal::Respostas { since: None });
        assert!(matches!(r, RespostaLocal::Erro { codigo: 0, .. }));
    }

    #[test]
    fn linha_ilegivel_vira_400_em_uma_linha() {
        let (_d, c) = ctx("http://127.0.0.1:9", Some("nvos_m_teste"));
        let s = responder(&c, 1000, "luan", "isto nao e json");
        assert!(s.ends_with('\n'));
        assert_eq!(s.matches('\n').count(), 1);
        assert!(s.contains("\"codigo\":400"));
    }
}
