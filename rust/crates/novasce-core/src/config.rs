//! Config do Novasce OS: a da imagem (`/usr/share/novasce/config.toml`) com a da
//! máquina (`/etc/novasce/config.toml`) por cima, chave por chave.
use serde::Deserialize;
use std::path::Path;

pub const CONFIG_DA_IMAGEM: &str = "/usr/share/novasce/config.toml";
pub const CONFIG_DA_MAQUINA: &str = "/etc/novasce/config.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub tenancy_api: String,
    pub tenancy_web: String,
    pub sala_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ErroConfig {
    #[error("não deu para ler {caminho}: {fonte}")]
    Ler {
        caminho: String,
        fonte: std::io::Error,
    },
    #[error("config com erro ({origem}): {fonte}")]
    Toml {
        origem: &'static str,
        fonte: toml::de::Error,
    },
    #[error("falta a chave {0} na config")]
    Falta(&'static str),
    #[error("a chave {chave} precisa começar com http:// ou https://, veio {valor}")]
    UrlInvalida { chave: &'static str, valor: String },
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Parcial {
    tenancy_api: Option<String>,
    tenancy_web: Option<String>,
    sala_url: Option<String>,
}

impl Config {
    pub fn carregar() -> Result<Self, ErroConfig> {
        Self::carregar_de(Path::new(CONFIG_DA_IMAGEM), Path::new(CONFIG_DA_MAQUINA))
    }

    pub fn carregar_de(imagem: &Path, maquina: &Path) -> Result<Self, ErroConfig> {
        let texto_imagem = ler(imagem)?.ok_or_else(|| ErroConfig::Ler {
            caminho: imagem.display().to_string(),
            fonte: std::io::Error::from(std::io::ErrorKind::NotFound),
        })?;
        let texto_maquina = ler(maquina)?;
        Self::de_texto(&texto_imagem, texto_maquina.as_deref())
    }

    pub fn de_texto(imagem: &str, maquina: Option<&str>) -> Result<Self, ErroConfig> {
        let base: Parcial = toml::from_str(imagem).map_err(|fonte| ErroConfig::Toml {
            origem: "imagem",
            fonte,
        })?;
        let local: Parcial = match maquina {
            Some(texto) => toml::from_str(texto).map_err(|fonte| ErroConfig::Toml {
                origem: "máquina",
                fonte,
            })?,
            None => Parcial::default(),
        };
        let tenancy_api = url("tenancy_api", local.tenancy_api.or(base.tenancy_api))?;
        let tenancy_web = url("tenancy_web", local.tenancy_web.or(base.tenancy_web))?;
        let sala = local
            .sala_url
            .or(base.sala_url)
            .ok_or(ErroConfig::Falta("sala_url"))?
            .replace("${TENANCY_WEB}", &tenancy_web)
            .replace("${TENANCY_API}", &tenancy_api);
        let sala_url = url("sala_url", Some(sala))?;
        Ok(Self {
            tenancy_api,
            tenancy_web,
            sala_url,
        })
    }

    /// URLs que a máquina abre: as do Tenancy saem da config, as de fora são fixas.
    pub fn url(&self, nome: &str) -> Option<String> {
        match nome {
            "tenancy" => Some(self.tenancy_web.clone()),
            "material" => Some(format!("{}/time/material", self.tenancy_web)),
            "sala" => Some(self.sala_url.clone()),
            "claude" => Some("https://claude.ai".to_string()),
            "chatgpt" => Some("https://chatgpt.com".to_string()),
            "gemini" => Some("https://gemini.google.com".to_string()),
            "whatsapp" => Some("https://web.whatsapp.com".to_string()),
            _ => None,
        }
    }
}

fn ler(caminho: &Path) -> Result<Option<String>, ErroConfig> {
    match std::fs::read_to_string(caminho) {
        Ok(texto) => Ok(Some(texto)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(fonte) => Err(ErroConfig::Ler {
            caminho: caminho.display().to_string(),
            fonte,
        }),
    }
}

fn url(chave: &'static str, valor: Option<String>) -> Result<String, ErroConfig> {
    let valor = valor.ok_or(ErroConfig::Falta(chave))?;
    let valor = valor.trim().trim_end_matches('/').to_string();
    if valor.starts_with("https://") || valor.starts_with("http://") {
        Ok(valor)
    } else {
        Err(ErroConfig::UrlInvalida { chave, valor })
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    const IMAGEM: &str = r#"
tenancy_api = "https://api.exemplo"
tenancy_web = "https://web.exemplo/"
sala_url = "${TENANCY_WEB}/sala"
"#;

    #[test]
    fn le_a_imagem_e_expande_a_sala() {
        let c = Config::de_texto(IMAGEM, None).unwrap();
        assert_eq!(c.tenancy_api, "https://api.exemplo");
        assert_eq!(c.tenancy_web, "https://web.exemplo");
        assert_eq!(c.sala_url, "https://web.exemplo/sala");
    }

    #[test]
    fn a_maquina_sobrescreve_so_o_que_traz() {
        let c = Config::de_texto(IMAGEM, Some(r#"sala_url = "https://outra.exemplo/tv""#)).unwrap();
        assert_eq!(c.tenancy_api, "https://api.exemplo");
        assert_eq!(c.sala_url, "https://outra.exemplo/tv");
    }

    #[test]
    fn chave_desconhecida_e_erro() {
        let erro = Config::de_texto(IMAGEM, Some("tenancy = 1")).unwrap_err();
        assert!(matches!(
            erro,
            ErroConfig::Toml {
                origem: "máquina",
                ..
            }
        ));
    }

    #[test]
    fn url_sem_http_e_erro() {
        let erro = Config::de_texto(IMAGEM, Some(r#"tenancy_api = "ftp://x""#)).unwrap_err();
        assert!(matches!(
            erro,
            ErroConfig::UrlInvalida {
                chave: "tenancy_api",
                ..
            }
        ));
    }

    #[test]
    fn carregar_de_aceita_maquina_ausente() {
        let pasta = tempfile::tempdir().unwrap();
        let imagem = pasta.path().join("config.toml");
        std::fs::write(&imagem, IMAGEM).unwrap();
        let c = Config::carregar_de(&imagem, &pasta.path().join("nao-existe.toml")).unwrap();
        assert_eq!(c.sala_url, "https://web.exemplo/sala");
    }

    #[test]
    fn urls_da_casa() {
        let c = Config::de_texto(IMAGEM, None).unwrap();
        assert_eq!(c.url("tenancy").as_deref(), Some("https://web.exemplo"));
        assert_eq!(
            c.url("material").as_deref(),
            Some("https://web.exemplo/time/material")
        );
        assert_eq!(c.url("sala").as_deref(), Some("https://web.exemplo/sala"));
        assert_eq!(c.url("claude").as_deref(), Some("https://claude.ai"));
        assert_eq!(
            c.url("whatsapp").as_deref(),
            Some("https://web.whatsapp.com")
        );
        assert_eq!(c.url("outra"), None);
    }

    #[test]
    fn a_config_publicada_na_imagem_vale() {
        let texto = include_str!("../../../../system_files/usr/share/novasce/config.toml");
        let c = Config::de_texto(texto, None).unwrap();
        assert_eq!(
            c.tenancy_api,
            "https://novasce-tenancy-api-production.up.railway.app/api/v1"
        );
        assert_eq!(c.tenancy_web, "https://tenancy.novasce.com");
        assert_eq!(c.sala_url, "https://tenancy.novasce.com/sala");
    }

    #[test]
    fn a_api_com_prefixo_mantem_o_prefixo() {
        let c = Config::de_texto(
            IMAGEM,
            Some(r#"tenancy_api = "https://api.exemplo/api/v1/""#),
        )
        .unwrap();
        assert_eq!(c.tenancy_api, "https://api.exemplo/api/v1");
    }
}
