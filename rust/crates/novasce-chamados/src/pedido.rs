use crate::fila::Fila;
use novasce_casa::chamado::{criado_de, Criado, MeuChamado};
use novasce_casa::multipart::{corpo, fronteira_nova, Parte};
use novasce_casa::nucleo::{ErroTenancy, Tenancy};
use novasce_casa::sinal_local::{ErroLocal, SinalLocal};

#[derive(Debug, Clone, PartialEq)]
pub struct Pedido {
    pub texto: String,
    pub print: Option<Vec<u8>>,
    pub dados_maquina: serde_json::Value,
}

#[derive(Debug, PartialEq)]
pub enum ErroEnvio {
    SemRede(String),
    Recusado { codigo: u16, mensagem: String },
}

pub trait Envio {
    fn enviar(&self, p: &Pedido) -> Result<Criado, ErroEnvio>;
    fn respostas(&self, since: Option<&str>) -> Result<Vec<MeuChamado>, ErroEnvio>;
    fn retorno(&self, ticket_id: &str, resolvido: bool) -> Result<(), ErroEnvio>;
}

fn de_tenancy(e: ErroTenancy) -> ErroEnvio {
    match e {
        ErroTenancy::SemRede(m) => ErroEnvio::SemRede(m),
        ErroTenancy::Status { codigo, mensagem } => ErroEnvio::Recusado { codigo, mensagem },
        ErroTenancy::Formato(m) => ErroEnvio::Recusado {
            codigo: 502,
            mensagem: m,
        },
    }
}

fn de_local(e: ErroLocal) -> ErroEnvio {
    match e {
        ErroLocal::SemRede(m) | ErroLocal::SemServico(m) => ErroEnvio::SemRede(m),
        ErroLocal::Recusado { codigo, mensagem } => ErroEnvio::Recusado { codigo, mensagem },
        ErroLocal::Formato(m) => ErroEnvio::Recusado {
            codigo: 502,
            mensagem: m,
        },
    }
}

pub struct EnvioPessoa {
    tenancy: Tenancy,
}

impl EnvioPessoa {
    pub fn novo(tenancy_api: &str, token: String) -> Self {
        EnvioPessoa {
            tenancy: Tenancy::new(tenancy_api, Some(token)),
        }
    }
}

impl Envio for EnvioPessoa {
    fn enviar(&self, p: &Pedido) -> Result<Criado, ErroEnvio> {
        let info = p.dados_maquina.to_string();
        let mut partes = vec![
            Parte::Texto {
                nome: "text",
                valor: &p.texto,
            },
            Parte::Texto {
                nome: "machineInfo",
                valor: &info,
            },
        ];
        if let Some(b) = p.print.as_deref() {
            partes.push(Parte::Arquivo {
                nome: "screenshot",
                arquivo: "print.png",
                tipo: "image/png",
                dados: b,
            });
        }
        let (tipo, b) = corpo(&partes, &fronteira_nova());
        self.tenancy
            .post_bytes("/os/tickets", &tipo, &b)
            .map(|v| criado_de(&v))
            .map_err(de_tenancy)
    }

    fn respostas(&self, since: Option<&str>) -> Result<Vec<MeuChamado>, ErroEnvio> {
        let caminho = match since {
            Some(s) => format!("/os/tickets/mine?since={}", s.replace('+', "%2B")),
            None => "/os/tickets/mine".into(),
        };
        self.tenancy.get(&caminho).map_err(de_tenancy)
    }

    fn retorno(&self, ticket_id: &str, resolvido: bool) -> Result<(), ErroEnvio> {
        self.tenancy
            .post_sem_resposta(
                &format!("/os/tickets/{ticket_id}/feedback"),
                &serde_json::json!({ "resolved": resolvido }),
            )
            .map_err(de_tenancy)
    }
}

pub struct EnvioMaquina {
    sinal: SinalLocal,
}

impl EnvioMaquina {
    pub fn novo(sinal: SinalLocal) -> Self {
        EnvioMaquina { sinal }
    }
}

impl Envio for EnvioMaquina {
    fn enviar(&self, p: &Pedido) -> Result<Criado, ErroEnvio> {
        self.sinal
            .chamado(&p.texto, p.dados_maquina.clone(), p.print.as_deref())
            .map_err(de_local)
    }
    fn respostas(&self, since: Option<&str>) -> Result<Vec<MeuChamado>, ErroEnvio> {
        self.sinal.respostas(since).map_err(de_local)
    }
    fn retorno(&self, ticket_id: &str, resolvido: bool) -> Result<(), ErroEnvio> {
        self.sinal.retorno(ticket_id, resolvido).map_err(de_local)
    }
}

/// A conta Sede sempre pede pela máquina. A pessoa pede com o próprio token; sem ele, pela máquina.
pub fn envio_para(
    conta: &str,
    conta_sede: &str,
    token_pessoa: Option<String>,
    tenancy_api: &str,
) -> Box<dyn Envio> {
    match token_pessoa {
        Some(t) if conta != conta_sede => Box::new(EnvioPessoa::novo(tenancy_api, t)),
        _ => Box::new(EnvioMaquina::novo(SinalLocal::padrao())),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resultado {
    Enviado {
        ticket_id: String,
        numero: Option<String>,
    },
    Guardado {
        id_local: String,
    },
    Recusado {
        mensagem: String,
    },
}

pub fn enviar_ou_guardar(envio: &dyn Envio, fila: &Fila, p: &Pedido) -> Resultado {
    match envio.enviar(p) {
        Ok(c) => Resultado::Enviado {
            ticket_id: c.ticket_id,
            numero: c.numero,
        },
        Err(ErroEnvio::SemRede(_)) => match fila.guardar(p) {
            Ok(id_local) => Resultado::Guardado { id_local },
            Err(e) => Resultado::Recusado {
                mensagem: format!("O pedido não foi enviado nem guardado: {e}"),
            },
        },
        Err(ErroEnvio::Recusado { mensagem, .. }) => Resultado::Recusado { mensagem },
    }
}

/// Manda o que estava guardado, na ordem. Para no primeiro sem rede; recusado sai da fila.
pub fn esvaziar_fila(envio: &dyn Envio, fila: &Fila) -> Vec<Criado> {
    let mut enviados = Vec::new();
    for (id, p) in fila.pendentes() {
        match envio.enviar(&p) {
            Ok(t) => {
                fila.remover(&id);
                enviados.push(t);
            }
            Err(ErroEnvio::SemRede(_)) => break,
            Err(ErroEnvio::Recusado { .. }) => fila.descartar(&id),
        }
    }
    enviados
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::fila::Fila;
    use mockito::Matcher;

    fn pedido() -> Pedido {
        Pedido {
            texto: "A impressora não aparece.".into(),
            print: Some(b"PNG".to_vec()),
            dados_maquina: serde_json::json!({"versao":"1.0.3"}),
        }
    }

    #[test]
    fn pessoa_envia_multipart_com_print() {
        let mut s = mockito::Server::new();
        let m = s
            .mock("POST", "/os/tickets")
            .match_header("authorization", "Bearer nvos_u_x")
            .match_header(
                "content-type",
                Matcher::Regex("multipart/form-data; boundary=".into()),
            )
            .match_body(Matcher::AllOf(vec![
                Matcher::Regex("name=\"screenshot\"; filename=\"print.png\"".into()),
                Matcher::Regex("\"versao\":\"1.0.3\"".into()),
            ]))
            .with_status(201)
            .with_body(r#"{"ticketId":"t-9","number":128}"#)
            .create();
        let e = EnvioPessoa::novo(&s.url(), "nvos_u_x".into());
        assert_eq!(
            e.enviar(&pedido()).unwrap(),
            Criado {
                ticket_id: "t-9".into(),
                numero: Some("#128".into())
            }
        );
        m.assert();
    }

    #[test]
    fn sem_rede_guarda_na_fila() {
        let d = tempfile::tempdir().unwrap();
        let fila = Fila::em(d.path().join("fila"));
        let e = EnvioPessoa::novo("http://127.0.0.1:9", "nvos_u_x".into());
        assert!(matches!(
            enviar_ou_guardar(&e, &fila, &pedido()),
            Resultado::Guardado { .. }
        ));
        let p = fila.pendentes();
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].1.print.as_deref(), Some(&b"PNG"[..]));
    }

    #[test]
    fn recusado_nao_vai_para_a_fila() {
        let mut s = mockito::Server::new();
        s.mock("POST", "/os/tickets")
            .with_status(400)
            .with_body(r#"{"statusCode":400,"message":"text é obrigatório"}"#)
            .create();
        let d = tempfile::tempdir().unwrap();
        let fila = Fila::em(d.path().join("fila"));
        let r = enviar_ou_guardar(&EnvioPessoa::novo(&s.url(), "t".into()), &fila, &pedido());
        assert_eq!(
            r,
            Resultado::Recusado {
                mensagem: "text é obrigatório".into()
            }
        );
        assert!(fila.pendentes().is_empty());
    }

    #[test]
    fn esvaziar_fila_manda_e_remove() {
        let mut s = mockito::Server::new();
        s.mock("POST", "/os/tickets")
            .with_status(201)
            .with_body(r#"{"ticketId":"t-1","number":7}"#)
            .expect(2)
            .create();
        let d = tempfile::tempdir().unwrap();
        let fila = Fila::em(d.path().join("fila"));
        fila.guardar(&pedido()).unwrap();
        fila.guardar(&pedido()).unwrap();
        let enviados = esvaziar_fila(&EnvioPessoa::novo(&s.url(), "t".into()), &fila);
        assert_eq!(
            enviados
                .iter()
                .map(|c| c.numero.clone().unwrap())
                .collect::<Vec<_>>(),
            vec!["#7", "#7"]
        );
        assert!(fila.pendentes().is_empty());
    }
}
