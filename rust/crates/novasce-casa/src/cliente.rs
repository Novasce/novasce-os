//! Cliente HTTP genérico do Tenancy para o M3: pessoa (nvos_u_) ou máquina (nvos_m_, só no root).
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

#[derive(Debug)]
pub enum ErroTenancy {
    Status { codigo: u16, mensagem: String },
    SemRede(String),
    Formato(String),
}

pub struct Tenancy {
    base: String,
    token: Option<String>,
    agente: ureq::Agent,
}

impl Tenancy {
    pub fn new(base: &str, token: Option<String>) -> Tenancy {
        let agente = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .user_agent("novasce-os-m3")
            .build();
        Tenancy {
            base: base.trim_end_matches('/').into(),
            token,
            agente,
        }
    }

    fn pedido(&self, metodo: &str, caminho: &str) -> ureq::Request {
        let r = self
            .agente
            .request(metodo, &format!("{}{caminho}", self.base))
            .set("Accept", "application/json");
        match &self.token {
            Some(t) => r.set("Authorization", &format!("Bearer {t}")),
            None => r,
        }
    }

    fn erro(e: ureq::Error) -> ErroTenancy {
        match e {
            ureq::Error::Status(codigo, r) => {
                let texto = r.into_string().unwrap_or_default();
                let mensagem = serde_json::from_str::<serde_json::Value>(&texto)
                    .ok()
                    .and_then(|v| match &v["message"] {
                        serde_json::Value::String(s) => Some(s.clone()),
                        serde_json::Value::Array(a) => Some(
                            a.iter()
                                .filter_map(|x| x.as_str())
                                .collect::<Vec<_>>()
                                .join("; "),
                        ),
                        _ => None,
                    })
                    .unwrap_or(texto);
                ErroTenancy::Status { codigo, mensagem }
            }
            ureq::Error::Transport(t) => ErroTenancy::SemRede(t.to_string()),
        }
    }

    fn corpo<T: DeserializeOwned>(r: ureq::Response) -> Result<T, ErroTenancy> {
        let texto = r
            .into_string()
            .map_err(|e| ErroTenancy::Formato(e.to_string()))?;
        let texto = if texto.trim().is_empty() {
            "null".to_string()
        } else {
            texto
        };
        serde_json::from_str(&texto).map_err(|e| ErroTenancy::Formato(e.to_string()))
    }

    pub fn get<T: DeserializeOwned>(&self, caminho: &str) -> Result<T, ErroTenancy> {
        Self::corpo(self.pedido("GET", caminho).call().map_err(Self::erro)?)
    }

    pub fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        caminho: &str,
        corpo: &B,
    ) -> Result<T, ErroTenancy> {
        Self::corpo(
            self.pedido("POST", caminho)
                .send_json(corpo)
                .map_err(Self::erro)?,
        )
    }

    pub fn post_sem_resposta<B: Serialize>(
        &self,
        caminho: &str,
        corpo: &B,
    ) -> Result<(), ErroTenancy> {
        self.pedido("POST", caminho)
            .send_json(corpo)
            .map(|_| ())
            .map_err(Self::erro)
    }

    pub fn post_bytes(
        &self,
        caminho: &str,
        content_type: &str,
        corpo: &[u8],
    ) -> Result<serde_json::Value, ErroTenancy> {
        Self::corpo(
            self.pedido("POST", caminho)
                .set("Content-Type", content_type)
                .send_bytes(corpo)
                .map_err(Self::erro)?,
        )
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn erro_nest_vira_mensagem_e_428_fica_no_codigo() {
        let mut s = mockito::Server::new();
        s.mock("POST", "/os/auth/device/token")
            .with_status(428)
            .with_body(r#"{"statusCode":428,"message":"pendente"}"#)
            .create();
        s.mock("POST", "/os/tickets")
            .with_status(400)
            .with_body(r#"{"statusCode":400,"message":["text vazio","print grande"]}"#)
            .create();
        let t = Tenancy::new(&s.url(), Some("nvos_u_x".into()));
        match t.post::<_, serde_json::Value>("/os/auth/device/token", &serde_json::json!({})) {
            Err(ErroTenancy::Status {
                codigo: 428,
                mensagem,
            }) => assert_eq!(mensagem, "pendente"),
            outro => panic!("{outro:?}"),
        }
        match t.post_bytes("/os/tickets", "text/plain", b"x") {
            Err(ErroTenancy::Status {
                codigo: 400,
                mensagem,
            }) => assert_eq!(mensagem, "text vazio; print grande"),
            outro => panic!("{outro:?}"),
        }
    }

    #[test]
    fn manda_o_token_e_aceita_corpo_vazio() {
        let mut s = mockito::Server::new();
        let m = s
            .mock("POST", "/os/tickets/1/feedback")
            .match_header("authorization", "Bearer nvos_u_x")
            .with_status(204)
            .create();
        Tenancy::new(&s.url(), Some("nvos_u_x".into()))
            .post_sem_resposta(
                "/os/tickets/1/feedback",
                &serde_json::json!({"resolved": true}),
            )
            .unwrap();
        m.assert();
    }
}
