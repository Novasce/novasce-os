//! Cliente do Tenancy para as rotas da máquina (contrato v2, seção 3).
//! A base já traz o prefixo global da API (`.../api/v1`); as rotas são `/os/...`.
use crate::contrato::{
    ErroNest, Maquina, MembroDoTime, PedidoDeDono, PedidoDeRegistro, RespostaDaEspera,
    RespostaDeRegistro, Sinal,
};
use serde::de::DeserializeOwned;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum ErroTenancy {
    #[error("o Tenancy ainda não tem o módulo das máquinas (404)")]
    SemModulo,
    #[error("o token desta máquina não vale mais (401)")]
    NaoAutorizado,
    #[error("o Tenancy recusou: {0}")]
    Proibido(String),
    #[error("o Tenancy ainda está esperando (428)")]
    Pendente,
    #[error("o código venceu ou o token já foi entregue (410)")]
    Encerrado,
    #[error("o Tenancy respondeu {status}: {mensagem}")]
    Http { status: u16, mensagem: String },
    #[error("sem conexão com o Tenancy: {0}")]
    Rede(String),
    #[error("resposta fora do contrato: {0}")]
    Formato(String),
}

pub struct ClienteTenancy {
    base: String,
    agente: ureq::Agent,
}

impl ClienteTenancy {
    pub fn novo(base: &str) -> Self {
        let agente = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .user_agent(concat!("novasce-os/", env!("CARGO_PKG_VERSION")))
            .build();
        Self {
            base: base.trim_end_matches('/').to_string(),
            agente,
        }
    }

    pub fn registrar(&self, pedido: &PedidoDeRegistro) -> Result<RespostaDeRegistro, ErroTenancy> {
        json(tratar(
            self.agente
                .post(&self.url("/os/machines/pair"))
                .send_json(pedido),
        )?)
    }

    pub fn esperar_registro(&self, poll_token: &str) -> Result<RespostaDaEspera, ErroTenancy> {
        validar_segmento(poll_token)?;
        let rota = format!("/os/machines/pair/{poll_token}");
        json(tratar(self.agente.get(&self.url(&rota)).call())?)
    }

    pub fn maquina(&self, token: &str) -> Result<Maquina, ErroTenancy> {
        json(tratar(
            com_token(self.agente.get(&self.url("/os/machine")), token).call(),
        )?)
    }

    pub fn time(&self, token: &str) -> Result<Vec<MembroDoTime>, ErroTenancy> {
        json(tratar(
            com_token(self.agente.get(&self.url("/os/machine/team")), token).call(),
        )?)
    }

    pub fn definir_dono(&self, token: &str, user_id: &str) -> Result<(), ErroTenancy> {
        let corpo = PedidoDeDono {
            user_id: user_id.to_string(),
        };
        tratar(
            com_token(self.agente.post(&self.url("/os/machine/owner")), token).send_json(&corpo),
        )?;
        Ok(())
    }

    pub fn enviar_sinal(&self, token: &str, sinal: &Sinal) -> Result<(), ErroTenancy> {
        tratar(
            com_token(self.agente.post(&self.url("/os/machine/heartbeat")), token).send_json(sinal),
        )?;
        Ok(())
    }

    fn url(&self, caminho: &str) -> String {
        format!("{}{}", self.base, caminho)
    }
}

/// A URL responde com 2xx dentro do limite? Usado pela conta Sede para decidir
/// entre a Sala e a página "A Sala chega com o Tenancy".
pub fn responde(url: &str, limite: Duration) -> bool {
    ureq::AgentBuilder::new()
        .timeout(limite)
        .build()
        .get(url)
        .call()
        .map(|r| (200..300).contains(&r.status()))
        .unwrap_or(false)
}

fn com_token(pedido: ureq::Request, token: &str) -> ureq::Request {
    pedido.set("Authorization", &format!("Bearer {token}"))
}

fn tratar(resultado: Result<ureq::Response, ureq::Error>) -> Result<ureq::Response, ErroTenancy> {
    match resultado {
        Ok(resposta) => Ok(resposta),
        Err(ureq::Error::Status(404, _)) => Err(ErroTenancy::SemModulo),
        Err(ureq::Error::Status(401, _)) => Err(ErroTenancy::NaoAutorizado),
        Err(ureq::Error::Status(428, _)) => Err(ErroTenancy::Pendente),
        Err(ureq::Error::Status(410, _)) => Err(ErroTenancy::Encerrado),
        Err(ureq::Error::Status(403, resposta)) => Err(ErroTenancy::Proibido(mensagem(resposta))),
        Err(ureq::Error::Status(status, resposta)) => Err(ErroTenancy::Http {
            status,
            mensagem: mensagem(resposta),
        }),
        Err(ureq::Error::Transport(falha)) => Err(ErroTenancy::Rede(falha.to_string())),
    }
}

fn mensagem(resposta: ureq::Response) -> String {
    resposta
        .into_json::<ErroNest>()
        .map(|e| e.mensagem())
        .unwrap_or_else(|_| "sem detalhe".to_string())
}

fn json<T: DeserializeOwned>(resposta: ureq::Response) -> Result<T, ErroTenancy> {
    resposta
        .into_json()
        .map_err(|e| ErroTenancy::Formato(e.to_string()))
}

fn validar_segmento(texto: &str) -> Result<(), ErroTenancy> {
    let valido = !texto.is_empty()
        && texto
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if valido {
        Ok(())
    } else {
        Err(ErroTenancy::Formato(
            "pollToken com caracteres inesperados".to_string(),
        ))
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::contrato::{EstadoDoRegistro, Papel, Variante};
    use mockito::Matcher;
    use serde_json::json;

    fn pedido() -> PedidoDeRegistro {
        PedidoDeRegistro {
            role: Papel::Sede,
            variant: Variante::Nvidia,
            hostname: "novasce-sede".into(),
            os_version: "2026.10.24.1".into(),
        }
    }

    #[test]
    fn registrar_manda_o_pedido_e_le_o_codigo() {
        let mut servidor = mockito::Server::new();
        let rota = servidor
            .mock("POST", "/os/machines/pair")
            .match_body(Matcher::Json(json!({"role": "sede", "variant": "nvidia", "hostname": "novasce-sede", "osVersion": "2026.10.24.1"})))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"pairingCode":"R7M2PX","pollToken":"pt_1","expiresAt":"2026-10-24T12:10:00Z"}"#)
            .create();
        let r = ClienteTenancy::novo(&servidor.url())
            .registrar(&pedido())
            .unwrap();
        assert_eq!(r.pairing_code, "R7M2PX");
        assert_eq!(r.poll_token, "pt_1");
        rota.assert();
    }

    #[test]
    fn sem_o_modulo_os_vira_sem_modulo() {
        let mut servidor = mockito::Server::new();
        let _rota = servidor
            .mock("POST", "/os/machines/pair")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"statusCode":404,"message":"Cannot POST /os/machines/pair"}"#)
            .create();
        let erro = ClienteTenancy::novo(&servidor.url())
            .registrar(&pedido())
            .unwrap_err();
        assert!(matches!(erro, ErroTenancy::SemModulo));
    }

    #[test]
    fn esperar_le_o_token_aprovado() {
        let mut servidor = mockito::Server::new();
        let _rota = servidor
            .mock("GET", "/os/machines/pair/pt_1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"status":"approved","machineId":"m1","machineToken":"nvos_m_abc"}"#)
            .create();
        let r = ClienteTenancy::novo(&servidor.url())
            .esperar_registro("pt_1")
            .unwrap();
        assert_eq!(r.status, EstadoDoRegistro::Approved);
        assert_eq!(r.machine_token.as_deref(), Some("nvos_m_abc"));
    }

    #[test]
    fn poll_recusado_e_403_e_vencido_e_410() {
        let mut servidor = mockito::Server::new();
        let _recusado = servidor
            .mock("GET", "/os/machines/pair/pt_recusado")
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"statusCode":403,"message":"Código recusado"}"#)
            .create();
        let _vencido = servidor
            .mock("GET", "/os/machines/pair/pt_vencido")
            .with_status(410)
            .create();
        let cliente = ClienteTenancy::novo(&servidor.url());
        assert!(
            matches!(cliente.esperar_registro("pt_recusado"), Err(ErroTenancy::Proibido(m)) if m == "Código recusado")
        );
        assert!(matches!(
            cliente.esperar_registro("pt_vencido"),
            Err(ErroTenancy::Encerrado)
        ));
    }

    #[test]
    fn a_base_com_prefixo_da_api_leva_o_prefixo() {
        let mut servidor = mockito::Server::new();
        let rota = servidor
            .mock("GET", "/api/v1/os/machine")
            .match_header("authorization", "Bearer nvos_m_abc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"id":"m1","name":"Sede","role":"sede","owner":null,"patrimonyCode":null}"#,
            )
            .create();
        let m = ClienteTenancy::novo(&format!("{}/api/v1/", servidor.url()))
            .maquina("nvos_m_abc")
            .unwrap();
        assert_eq!(m.id, "m1");
        rota.assert();
    }

    #[test]
    fn poll_token_estranho_nem_sai_da_maquina() {
        let erro = ClienteTenancy::novo("http://127.0.0.1:9")
            .esperar_registro("../x")
            .unwrap_err();
        assert!(matches!(erro, ErroTenancy::Formato(_)));
    }

    #[test]
    fn sinal_vai_com_o_token_e_os_nulos() {
        let mut servidor = mockito::Server::new();
        let rota = servidor
            .mock("POST", "/os/machine/heartbeat")
            .match_header("authorization", "Bearer nvos_m_abc")
            .match_body(Matcher::Json(json!({"osVersion": "2026.10.24.1", "stagedVersion": null, "diskEncrypted": true, "secureBoot": true, "agentsIdle": null})))
            .with_status(204)
            .create();
        let sinal = Sinal {
            os_version: "2026.10.24.1".into(),
            staged_version: None,
            disk_encrypted: true,
            secure_boot: true,
            agents_idle: None,
        };
        ClienteTenancy::novo(&servidor.url())
            .enviar_sinal("nvos_m_abc", &sinal)
            .unwrap();
        rota.assert();
    }

    #[test]
    fn token_revogado_vira_nao_autorizado() {
        let mut servidor = mockito::Server::new();
        let _rota = servidor
            .mock("GET", "/os/machine")
            .with_status(401)
            .create();
        let erro = ClienteTenancy::novo(&servidor.url())
            .maquina("nvos_m_velho")
            .unwrap_err();
        assert!(matches!(erro, ErroTenancy::NaoAutorizado));
    }

    #[test]
    fn time_fora_do_prazo_traz_a_mensagem() {
        let mut servidor = mockito::Server::new();
        let _rota = servidor
            .mock("GET", "/os/machine/team")
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"statusCode":403,"message":"Prazo de 24 h encerrado"}"#)
            .create();
        let erro = ClienteTenancy::novo(&servidor.url())
            .time("nvos_m_abc")
            .unwrap_err();
        assert!(matches!(erro, ErroTenancy::Proibido(m) if m == "Prazo de 24 h encerrado"));
    }

    #[test]
    fn dono_vai_uma_vez_com_o_user_id() {
        let mut servidor = mockito::Server::new();
        let rota = servidor
            .mock("POST", "/os/machine/owner")
            .match_header("authorization", "Bearer nvos_m_abc")
            .match_body(Matcher::Json(json!({"userId": "u1"})))
            .with_status(204)
            .create();
        ClienteTenancy::novo(&servidor.url())
            .definir_dono("nvos_m_abc", "u1")
            .unwrap();
        rota.assert();
    }

    #[test]
    fn sem_rede_vira_rede() {
        let erro = ClienteTenancy::novo("http://127.0.0.1:9")
            .maquina("nvos_m_abc")
            .unwrap_err();
        assert!(matches!(erro, ErroTenancy::Rede(_)));
    }

    #[test]
    fn responde_so_com_2xx() {
        let mut servidor = mockito::Server::new();
        let _sala = servidor.mock("GET", "/sala").with_status(200).create();
        let _fora = servidor.mock("GET", "/fora").with_status(404).create();
        let limite = Duration::from_secs(2);
        assert!(responde(&format!("{}/sala", servidor.url()), limite));
        assert!(!responde(&format!("{}/fora", servidor.url()), limite));
        assert!(!responde("http://127.0.0.1:9/sala", limite));
    }
}
