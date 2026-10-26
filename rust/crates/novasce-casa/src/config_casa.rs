//! Configuração dos apps da casa (apps.toml). Nada aqui é segredo: a imagem é pública.
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct ConfigCasa {
    pub github: SecaoGithub,
    pub brain: SecaoBrain,
    pub google: SecaoGoogle,
    pub voz: SecaoVoz,
    pub sede: SecaoSede,
    pub atualizacao: SecaoAtualizacao,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct SecaoGithub {
    /// Client ID do OAuth App "Novasce OS" (público). Vazio: o passo 3 explica e oferece pular.
    pub client_id: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct SecaoBrain {
    /// Repositório-modelo "dono/nome" da Novasce Base. Vazio ou inacessível: brain começa vazio.
    pub modelo: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SecaoGoogle {
    pub client_id: String,
    pub client_secret: String,
    pub pasta: String,
}
impl Default for SecaoGoogle {
    fn default() -> Self {
        SecaoGoogle {
            client_id: String::new(),
            client_secret: String::new(),
            pasta: "Novasce OS/Documentos".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SecaoVoz {
    pub whisper: String,
    pub modelo: String,
    pub modelo_reserva: String,
    pub qualquer_campo: bool,
    pub limite_toque_ms: u64,
    pub vocabulario: String,
}
impl Default for SecaoVoz {
    fn default() -> Self {
        SecaoVoz {
            whisper: "/usr/libexec/novasce/whisper-cli".into(),
            modelo: "/usr/share/novasce/voz/ggml-small-q5_1.bin".into(),
            modelo_reserva: "/usr/share/novasce/voz/ggml-base-q5_1.bin".into(),
            qualquer_campo: true,
            limite_toque_ms: 350,
            vocabulario: "/usr/share/novasce/voz/vocabulario.toml".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SecaoSede {
    pub conta: String,
    pub titulo_sala: String,
    pub navegador: Vec<String>,
    pub perfil_navegador: String,
    pub volta_minutos: u32,
    pub contagem_segundos: u32,
    pub tranca_minutos: u32,
}
impl Default for SecaoSede {
    fn default() -> Self {
        SecaoSede {
            conta: "sede".into(),
            titulo_sala: "Sala da Novasce".into(),
            navegador: vec!["flatpak".into(), "run".into(), "com.google.Chrome".into()],
            perfil_navegador: ".var/app/com.google.Chrome/novasce-esquece".into(),
            volta_minutos: 10,
            contagem_segundos: 30,
            tranca_minutos: 5,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SecaoAtualizacao {
    pub notas_url: String,
    pub notas_local: String,
}
impl Default for SecaoAtualizacao {
    fn default() -> Self {
        SecaoAtualizacao {
            notas_url:
                "https://raw.githubusercontent.com/Novasce/novasce-os/main/notas/versoes.json"
                    .into(),
            notas_local: "/usr/share/novasce/notas/versoes.json".into(),
        }
    }
}

pub const ARQUIVOS: [&str; 2] = ["/usr/share/novasce/apps.toml", "/etc/novasce/apps.toml"];
/// Se o M1 abrir a seção `[apps]` no config.toml, ela vale no lugar do apps.toml.
pub const CONFIG_DO_M1: [&str; 2] = ["/usr/share/novasce/config.toml", "/etc/novasce/config.toml"];

/// O conteúdo de `[apps]` de um config.toml do M1, como se fosse um apps.toml. Sem a seção, `None`.
pub fn secao_apps(texto: &str) -> Option<String> {
    let t: toml::Table = texto.parse().ok()?;
    toml::to_string(t.get("apps")?.as_table()?).ok()
}

impl ConfigCasa {
    pub fn carregar() -> ConfigCasa {
        let ler = |lista: &[&str]| -> Vec<String> {
            lista
                .iter()
                .filter_map(|a| std::fs::read_to_string(a).ok())
                .collect()
        };
        let do_m1: Vec<String> = ler(&CONFIG_DO_M1)
            .iter()
            .map(String::as_str)
            .filter_map(secao_apps)
            .collect();
        let textos = if do_m1.is_empty() {
            ler(&ARQUIVOS)
        } else {
            do_m1
        };
        let refs: Vec<&str> = textos.iter().map(String::as_str).collect();
        ConfigCasa::de_textos(&refs).unwrap_or_default()
    }

    /// Junta os arquivos em ordem: o seguinte reforça o anterior, chave por chave.
    pub fn de_textos(textos: &[&str]) -> Result<ConfigCasa, String> {
        let mut juntos = toml::Table::new();
        for t in textos {
            let tabela: toml::Table = t.parse().map_err(|e| format!("apps.toml ilegível: {e}"))?;
            juntar(&mut juntos, tabela);
        }
        toml::Value::Table(juntos)
            .try_into()
            .map_err(|e| format!("apps.toml com valor errado: {e}"))
    }
}

fn juntar(base: &mut toml::Table, novo: toml::Table) {
    for (chave, valor) in novo {
        match (base.get_mut(&chave), valor) {
            (Some(toml::Value::Table(b)), toml::Value::Table(n)) => juntar(b, n),
            (_, v) => {
                base.insert(chave, v);
            }
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn padroes_quando_vazio() {
        let c = ConfigCasa::de_textos(&[""]).unwrap();
        assert_eq!(c.github.client_id, "");
        assert_eq!(c.google.pasta, "Novasce OS/Documentos");
        assert!(c.voz.qualquer_campo);
        assert_eq!(c.voz.limite_toque_ms, 350);
        assert_eq!(c.sede.conta, "sede");
        assert_eq!(c.sede.volta_minutos, 10);
        assert_eq!(c.sede.tranca_minutos, 5);
        assert_eq!(c.sede.titulo_sala, "Sala da Novasce");
    }

    #[test]
    fn etc_reforca_usr_share_por_chave() {
        let usr = "[tenancy]\napi = \"x\"\n[github]\nclient_id = \"Iv1.abc\"\n[voz]\nqualquer_campo = true\n";
        let etc = "[voz]\nqualquer_campo = false\n";
        let c = ConfigCasa::de_textos(&[usr, etc]).unwrap();
        assert_eq!(c.github.client_id, "Iv1.abc");
        assert!(!c.voz.qualquer_campo);
        assert_eq!(c.voz.limite_toque_ms, 350);
    }

    #[test]
    fn secao_apps_do_config_do_m1_tem_preferencia() {
        let m1 = "tenancy_api = \"https://a\"\n[apps.github]\nclient_id = \"Iv1.z\"\n[apps.voz]\nqualquer_campo = false\n";
        let c = ConfigCasa::de_textos(&[secao_apps(m1).unwrap().as_str()]).unwrap();
        assert_eq!(c.github.client_id, "Iv1.z");
        assert!(!c.voz.qualquer_campo);
        assert!(secao_apps("tenancy_api = \"https://a\"\n").is_none());
    }

    #[test]
    fn texto_quebrado_diz_o_que_aconteceu() {
        let e = ConfigCasa::de_textos(&["[voz"]).unwrap_err();
        assert!(e.contains("apps.toml"));
    }
}
