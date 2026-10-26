//! Registro da máquina no Tenancy por código (contrato v2, seção 3).
//! No poll, 200 traz pending ou approved; 403 é recusado; 410 é vencido ou já entregue.
use crate::contrato::{EstadoDoRegistro, PedidoDeRegistro, RespostaDeRegistro};
use crate::tenancy::{ClienteTenancy, ErroTenancy};
use crate::token::{ErroToken, GuardaDoToken, MaquinaSalva};
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub enum FimDoRegistro {
    Aprovado { machine_id: String },
    Negado,
    Expirado,
}

#[derive(Debug, thiserror::Error)]
pub enum ErroDoRegistro {
    #[error(transparent)]
    Tenancy(#[from] ErroTenancy),
    #[error(transparent)]
    Token(#[from] ErroToken),
    #[error("o Tenancy aprovou, mas não mandou o token da máquina")]
    SemToken,
}

pub struct Espera {
    pub intervalo: Duration,
    pub tentativas: u32,
}

impl Default for Espera {
    /// De 5 em 5 segundos, por pouco mais que os 10 minutos do código.
    fn default() -> Self {
        Self {
            intervalo: Duration::from_secs(5),
            tentativas: 125,
        }
    }
}

pub fn registrar(
    cliente: &ClienteTenancy,
    guarda: &GuardaDoToken,
    pedido: &PedidoDeRegistro,
    espera: &Espera,
    dormir: &dyn Fn(Duration),
    ao_receber_codigo: &mut dyn FnMut(&RespostaDeRegistro),
) -> Result<FimDoRegistro, ErroDoRegistro> {
    let resposta = cliente.registrar(pedido)?;
    ao_receber_codigo(&resposta);
    for _ in 0..espera.tentativas {
        dormir(espera.intervalo);
        let situacao = match cliente.esperar_registro(&resposta.poll_token) {
            Ok(situacao) => situacao,
            Err(ErroTenancy::Proibido(_)) => return Ok(FimDoRegistro::Negado),
            Err(ErroTenancy::Encerrado) => return Ok(FimDoRegistro::Expirado),
            Err(outro) => return Err(outro.into()),
        };
        match situacao.status {
            EstadoDoRegistro::Pending => continue,
            EstadoDoRegistro::Approved => {
                let (Some(machine_id), Some(machine_token)) =
                    (situacao.machine_id, situacao.machine_token)
                else {
                    return Err(ErroDoRegistro::SemToken);
                };
                guarda.salvar(&MaquinaSalva {
                    machine_id: machine_id.clone(),
                    machine_token,
                })?;
                return Ok(FimDoRegistro::Aprovado { machine_id });
            }
        }
    }
    Ok(FimDoRegistro::Expirado)
}

/// "R7M2PX" vira "R7M 2PX", como no canvas (prancha 1).
pub fn formatar_codigo(codigo: &str) -> String {
    if codigo.is_ascii() && codigo.len() == 6 {
        let (a, b) = codigo.split_at(3);
        format!("{a} {b}")
    } else {
        codigo.to_string()
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::contrato::{Papel, Variante};

    fn pedido() -> PedidoDeRegistro {
        PedidoDeRegistro {
            role: Papel::Notebook,
            variant: Variante::Padrao,
            hostname: "novasce".into(),
            os_version: "2026.10.24.1".into(),
        }
    }

    fn sem_dormir(_: Duration) {}

    fn rapido(tentativas: u32) -> Espera {
        Espera {
            intervalo: Duration::ZERO,
            tentativas,
        }
    }

    /// Devolve o servidor e a rota criada: a rota precisa ficar viva no teste.
    fn servidor_com_codigo() -> (mockito::ServerGuard, mockito::Mock) {
        let mut servidor = mockito::Server::new();
        let rota = servidor
            .mock("POST", "/os/machines/pair")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"pairingCode":"R7M2PX","pollToken":"pt_1","expiresAt":"2026-10-24T12:10:00Z"}"#,
            )
            .create();
        (servidor, rota)
    }

    /// A rota da espera, ainda sem `.create()`: cada teste cria (e pode pôr `.expect`).
    fn espera(servidor: &mut mockito::ServerGuard, status: usize, corpo: &str) -> mockito::Mock {
        servidor
            .mock("GET", "/os/machines/pair/pt_1")
            .with_status(status)
            .with_header("content-type", "application/json")
            .with_body(corpo)
    }

    #[test]
    fn aprovado_avisa_o_codigo_e_guarda_o_token() {
        let (mut servidor, _codigo) = servidor_com_codigo();
        let _espera = espera(
            &mut servidor,
            200,
            r#"{"status":"approved","machineId":"m1","machineToken":"nvos_m_abc"}"#,
        )
        .create();
        let pasta = tempfile::tempdir().unwrap();
        let guarda = GuardaDoToken::em(pasta.path().join("novasce/maquina.json"));
        let mut codigos = Vec::new();
        let fim = registrar(
            &ClienteTenancy::novo(&servidor.url()),
            &guarda,
            &pedido(),
            &rapido(3),
            &sem_dormir,
            &mut |r| codigos.push(r.pairing_code.clone()),
        )
        .unwrap();
        assert_eq!(
            fim,
            FimDoRegistro::Aprovado {
                machine_id: "m1".into()
            }
        );
        assert_eq!(codigos, vec!["R7M2PX".to_string()]);
        assert_eq!(guarda.ler().unwrap().unwrap().machine_token, "nvos_m_abc");
    }

    #[test]
    fn recusado_403_vira_negado_e_nao_guarda_nada() {
        let (mut servidor, _codigo) = servidor_com_codigo();
        let _espera = espera(
            &mut servidor,
            403,
            r#"{"statusCode":403,"message":"Código recusado"}"#,
        )
        .create();
        let pasta = tempfile::tempdir().unwrap();
        let guarda = GuardaDoToken::em(pasta.path().join("maquina.json"));
        let fim = registrar(
            &ClienteTenancy::novo(&servidor.url()),
            &guarda,
            &pedido(),
            &rapido(3),
            &sem_dormir,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fim, FimDoRegistro::Negado);
        assert_eq!(guarda.ler().unwrap(), None);
    }

    #[test]
    fn vencido_ou_ja_entregue_410_vira_expirado() {
        let (mut servidor, _codigo) = servidor_com_codigo();
        let _espera = espera(
            &mut servidor,
            410,
            r#"{"statusCode":410,"message":"Código expirado"}"#,
        )
        .create();
        let pasta = tempfile::tempdir().unwrap();
        let guarda = GuardaDoToken::em(pasta.path().join("maquina.json"));
        let fim = registrar(
            &ClienteTenancy::novo(&servidor.url()),
            &guarda,
            &pedido(),
            &rapido(3),
            &sem_dormir,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fim, FimDoRegistro::Expirado);
        assert_eq!(guarda.ler().unwrap(), None);
    }

    #[test]
    fn pendente_ate_acabar_as_tentativas_vira_expirado() {
        let (mut servidor, _codigo) = servidor_com_codigo();
        let rota = espera(&mut servidor, 200, r#"{"status":"pending"}"#)
            .expect(3)
            .create();
        let pasta = tempfile::tempdir().unwrap();
        let guarda = GuardaDoToken::em(pasta.path().join("maquina.json"));
        let fim = registrar(
            &ClienteTenancy::novo(&servidor.url()),
            &guarda,
            &pedido(),
            &rapido(3),
            &sem_dormir,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fim, FimDoRegistro::Expirado);
        rota.assert();
    }

    #[test]
    fn aprovado_sem_token_e_erro() {
        let (mut servidor, _codigo) = servidor_com_codigo();
        let _espera = espera(
            &mut servidor,
            200,
            r#"{"status":"approved","machineId":"m1"}"#,
        )
        .create();
        let pasta = tempfile::tempdir().unwrap();
        let guarda = GuardaDoToken::em(pasta.path().join("maquina.json"));
        let erro = registrar(
            &ClienteTenancy::novo(&servidor.url()),
            &guarda,
            &pedido(),
            &rapido(3),
            &sem_dormir,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(matches!(erro, ErroDoRegistro::SemToken));
    }

    #[test]
    fn sem_o_modulo_os_o_erro_diz_isso() {
        let mut servidor = mockito::Server::new();
        let _rota = servidor
            .mock("POST", "/os/machines/pair")
            .with_status(404)
            .create();
        let pasta = tempfile::tempdir().unwrap();
        let guarda = GuardaDoToken::em(pasta.path().join("maquina.json"));
        let erro = registrar(
            &ClienteTenancy::novo(&servidor.url()),
            &guarda,
            &pedido(),
            &rapido(3),
            &sem_dormir,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(matches!(
            erro,
            ErroDoRegistro::Tenancy(ErroTenancy::SemModulo)
        ));
    }

    #[test]
    fn codigo_aparece_em_dois_blocos() {
        assert_eq!(formatar_codigo("R7M2PX"), "R7M 2PX");
        assert_eq!(formatar_codigo("ABC"), "ABC");
    }
}
