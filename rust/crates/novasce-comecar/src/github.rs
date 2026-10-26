//! GitHub sem senha: código do aparelho (Device Flow do OAuth App "Novasce OS") e a API REST.
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum ErroGithub {
    SemClientId,
    SemRede(String),
    Recusado { codigo: u16, mensagem: String },
}

impl std::fmt::Display for ErroGithub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroGithub::SemClientId => {
                write!(f, "o OAuth App da Novasce ainda não foi configurado")
            }
            ErroGithub::SemRede(m) => write!(f, "sem internet: {m}"),
            ErroGithub::Recusado { codigo, mensagem } => {
                write!(f, "o GitHub recusou ({codigo}): {mensagem}")
            }
        }
    }
}

fn erro(e: ureq::Error) -> ErroGithub {
    match e {
        ureq::Error::Status(codigo, r) => ErroGithub::Recusado {
            codigo,
            mensagem: r.into_string().unwrap_or_default(),
        },
        ureq::Error::Transport(t) => ErroGithub::SemRede(t.to_string()),
    }
}

fn agente() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(20))
        .user_agent("novasce-os")
        .build()
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CodigoGithub {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, PartialEq)]
pub enum Espera {
    Pendente,
    Devagar,
    Pronto(String),
    Expirou,
    Negado,
}

pub struct GithubDevice {
    client_id: String,
    base_login: String,
}

impl GithubDevice {
    pub fn novo(client_id: &str, base_login: &str) -> Self {
        GithubDevice {
            client_id: client_id.trim().into(),
            base_login: base_login.trim_end_matches('/').into(),
        }
    }

    pub fn do_github(client_id: &str) -> Self {
        GithubDevice::novo(client_id, "https://github.com")
    }

    pub fn pedir_codigo(&self) -> Result<CodigoGithub, ErroGithub> {
        if self.client_id.is_empty() {
            return Err(ErroGithub::SemClientId);
        }
        agente()
            .post(&format!("{}/login/device/code", self.base_login))
            .set("Accept", "application/json")
            .send_form(&[("client_id", &self.client_id), ("scope", "repo")])
            .map_err(erro)?
            .into_json()
            .map_err(|e| ErroGithub::Recusado {
                codigo: 502,
                mensagem: e.to_string(),
            })
    }

    pub fn esperar(&self, device_code: &str) -> Result<Espera, ErroGithub> {
        #[derive(Deserialize)]
        struct Resposta {
            access_token: Option<String>,
            error: Option<String>,
        }
        let r: Resposta = agente()
            .post(&format!("{}/login/oauth/access_token", self.base_login))
            .set("Accept", "application/json")
            .send_form(&[
                ("client_id", self.client_id.as_str()),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .map_err(erro)?
            .into_json()
            .map_err(|e| ErroGithub::Recusado {
                codigo: 502,
                mensagem: e.to_string(),
            })?;
        Ok(match (r.access_token, r.error.as_deref()) {
            (Some(t), _) => Espera::Pronto(t),
            (None, Some("authorization_pending")) => Espera::Pendente,
            (None, Some("slow_down")) => Espera::Devagar,
            (None, Some("expired_token")) => Espera::Expirou,
            (None, Some("access_denied")) => Espera::Negado,
            (None, outro) => {
                return Err(ErroGithub::Recusado {
                    codigo: 400,
                    mensagem: outro.unwrap_or("sem resposta").into(),
                })
            }
        })
    }
}

pub struct GithubApi {
    base: String,
    token: String,
}

impl GithubApi {
    pub fn novo(base: &str, token: String) -> Self {
        GithubApi {
            base: base.trim_end_matches('/').into(),
            token,
        }
    }

    pub fn do_github(token: String) -> Self {
        GithubApi::novo("https://api.github.com", token)
    }

    fn pedido(&self, metodo: &str, caminho: &str) -> ureq::Request {
        agente()
            .request(metodo, &format!("{}{caminho}", self.base))
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Accept", "application/vnd.github+json")
    }

    pub fn login(&self) -> Result<String, ErroGithub> {
        #[derive(Deserialize)]
        struct U {
            login: String,
        }
        let u: U = self
            .pedido("GET", "/user")
            .call()
            .map_err(erro)?
            .into_json()
            .map_err(|e| ErroGithub::Recusado {
                codigo: 502,
                mensagem: e.to_string(),
            })?;
        Ok(u.login)
    }

    fn existe(&self, caminho: &str) -> Result<bool, ErroGithub> {
        match self.pedido("GET", caminho).call() {
            Ok(_) => Ok(true),
            Err(ureq::Error::Status(403 | 404, _)) => Ok(false),
            Err(e) => Err(erro(e)),
        }
    }

    pub fn repo_existe(&self, dono: &str, nome: &str) -> Result<bool, ErroGithub> {
        self.existe(&format!("/repos/{dono}/{nome}"))
    }

    /// "Novasce/studio": a conta da pessoa lê o repositório? 403 ou 404 é "sem permissão".
    pub fn pode_ler(&self, repo: &str) -> Result<bool, ErroGithub> {
        self.existe(&format!("/repos/{repo}"))
    }

    pub fn criar_do_modelo(&self, modelo: &str, dono: &str, nome: &str) -> Result<(), ErroGithub> {
        self.pedido("POST", &format!("/repos/{modelo}/generate"))
            .send_json(serde_json::json!({ "owner": dono, "name": nome, "private": true, "description": "Meu brain, criado pelo Novasce OS" }))
            .map(|_| ())
            .map_err(erro)
    }

    pub fn criar_vazio(&self, nome: &str) -> Result<(), ErroGithub> {
        self.pedido("POST", "/user/repos")
            .send_json(serde_json::json!({ "name": nome, "private": true, "auto_init": true, "description": "Meu brain, criado pelo Novasce OS" }))
            .map(|_| ())
            .map_err(erro)
    }

    /// Depois do "generate", o GitHub copia o modelo em segundo plano. 409 = ainda vazio.
    pub fn tem_commit(&self, dono: &str, nome: &str) -> Result<bool, ErroGithub> {
        match self
            .pedido("GET", &format!("/repos/{dono}/{nome}/commits?per_page=1"))
            .call()
        {
            Ok(_) => Ok(true),
            Err(ureq::Error::Status(409, _)) => Ok(false),
            Err(e) => Err(erro(e)),
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use mockito::Matcher;

    #[test]
    fn sem_client_id_nem_tenta() {
        let g = GithubDevice::novo("", "http://127.0.0.1:9");
        assert!(matches!(g.pedir_codigo(), Err(ErroGithub::SemClientId)));
    }

    #[test]
    fn pede_codigo_com_escopo_repo() {
        let mut s = mockito::Server::new();
        let m = s
            .mock("POST", "/login/device/code")
            .match_body(Matcher::AllOf(vec![
                Matcher::UrlEncoded("client_id".into(), "Iv1.x".into()),
                Matcher::UrlEncoded("scope".into(), "repo".into()),
            ]))
            .with_body(r#"{"device_code":"dc","user_code":"4C2D-9KQX","verification_uri":"https://github.com/login/device","expires_in":900,"interval":5}"#)
            .create();
        let c = GithubDevice::novo("Iv1.x", &s.url())
            .pedir_codigo()
            .unwrap();
        m.assert();
        assert_eq!(c.user_code, "4C2D-9KQX");
    }

    #[test]
    fn espera_entende_cada_resposta() {
        let mut s = mockito::Server::new();
        let g = GithubDevice::novo("Iv1.x", &s.url());
        let m = s
            .mock("POST", "/login/oauth/access_token")
            .with_body(r#"{"error":"authorization_pending"}"#)
            .create();
        assert_eq!(g.esperar("dc").unwrap(), Espera::Pendente);
        m.remove();
        let m = s
            .mock("POST", "/login/oauth/access_token")
            .with_body(r#"{"error":"slow_down","interval":10}"#)
            .create();
        assert_eq!(g.esperar("dc").unwrap(), Espera::Devagar);
        m.remove();
        let m = s
            .mock("POST", "/login/oauth/access_token")
            .with_body(r#"{"error":"access_denied"}"#)
            .create();
        assert_eq!(g.esperar("dc").unwrap(), Espera::Negado);
        m.remove();
        s.mock("POST", "/login/oauth/access_token")
            .match_body(Matcher::UrlEncoded(
                "grant_type".into(),
                "urn:ietf:params:oauth:grant-type:device_code".into(),
            ))
            .with_body(r#"{"access_token":"gho_x","token_type":"bearer","scope":"repo"}"#)
            .create();
        assert_eq!(g.esperar("dc").unwrap(), Espera::Pronto("gho_x".into()));
    }

    #[test]
    fn api_login_repo_e_modelo() {
        let mut s = mockito::Server::new();
        s.mock("GET", "/user")
            .match_header("authorization", "Bearer gho_x")
            .with_body(r#"{"login":"Luan"}"#)
            .create();
        s.mock("GET", "/repos/Luan/luan-brain")
            .with_status(404)
            .create();
        let m = s
            .mock("POST", "/repos/Novasce/brain-base/generate")
            .match_body(Matcher::PartialJsonString(
                r#"{"owner":"Luan","name":"luan-brain","private":true}"#.into(),
            ))
            .with_status(201)
            .with_body("{}")
            .create();
        s.mock("GET", "/repos/Novasce/studio")
            .with_status(403)
            .create();
        let api = GithubApi::novo(&s.url(), "gho_x".into());
        assert_eq!(api.login().unwrap(), "Luan");
        assert!(!api.repo_existe("Luan", "luan-brain").unwrap());
        api.criar_do_modelo("Novasce/brain-base", "Luan", "luan-brain")
            .unwrap();
        m.assert();
        assert!(!api.pode_ler("Novasce/studio").unwrap());
    }
}
