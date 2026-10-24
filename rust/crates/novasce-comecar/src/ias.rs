//! As IAs da pessoa: app web para conversar, CLI por pessoa para trabalhar no brain.
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ia {
    Claude,
    Chatgpt,
    Gemini,
}

impl Ia {
    pub fn todas() -> [Ia; 3] {
        [Ia::Claude, Ia::Chatgpt, Ia::Gemini]
    }
    pub fn de_id(id: &str) -> Option<Ia> {
        Ia::todas().into_iter().find(|i| i.id() == id)
    }
    pub fn id(self) -> &'static str {
        match self {
            Ia::Claude => "claude",
            Ia::Chatgpt => "chatgpt",
            Ia::Gemini => "gemini",
        }
    }
    pub fn nome(self) -> &'static str {
        match self {
            Ia::Claude => "Claude",
            Ia::Chatgpt => "ChatGPT",
            Ia::Gemini => "Gemini",
        }
    }
    pub fn cli(self) -> &'static str {
        match self {
            Ia::Claude => "claude",
            Ia::Chatgpt => "codex",
            Ia::Gemini => "gemini",
        }
    }
    pub fn nome_cli(self) -> &'static str {
        match self {
            Ia::Claude => "Claude Code",
            Ia::Chatgpt => "Codex",
            Ia::Gemini => "Gemini CLI",
        }
    }
    pub fn url_web(self) -> &'static str {
        match self {
            Ia::Claude => "https://claude.ai/login",
            Ia::Chatgpt => "https://chatgpt.com/auth/login",
            Ia::Gemini => "https://gemini.google.com/",
        }
    }
    pub fn desktop(self) -> &'static str {
        match self {
            Ia::Claude => "novasce-claude",
            Ia::Chatgpt => "novasce-chatgpt",
            Ia::Gemini => "novasce-gemini",
        }
    }
    pub fn comando_instalar(self, casa: &Path) -> Vec<String> {
        let prefixo = casa.join(".local").to_string_lossy().to_string();
        match self {
            Ia::Claude => vec![
                "bash".into(),
                "-c".into(),
                "curl -fsSL https://claude.ai/install.sh | bash".into(),
            ],
            Ia::Chatgpt => vec![
                "npm".into(),
                "install".into(),
                "-g".into(),
                "--prefix".into(),
                prefixo,
                "@openai/codex".into(),
            ],
            Ia::Gemini => vec![
                "npm".into(),
                "install".into(),
                "-g".into(),
                "--prefix".into(),
                prefixo,
                "@google/gemini-cli".into(),
            ],
        }
    }
}

pub fn instalada(ia: Ia, casa: &Path) -> bool {
    casa.join(".local/bin").join(ia.cli()).exists()
}

/// Roda o instalador por pessoa. Devolve as últimas linhas do erro se falhar.
pub fn instalar(ia: Ia, casa: &Path) -> Result<(), String> {
    if instalada(ia, casa) {
        return Ok(());
    }
    let argv = ia.comando_instalar(casa);
    let caminho = format!(
        "{}:{}",
        casa.join(".local/bin").display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let s = Command::new(&argv[0])
        .args(&argv[1..])
        .env("PATH", caminho)
        .output()
        .map_err(|e| e.to_string())?;
    if s.status.success() && instalada(ia, casa) {
        Ok(())
    } else {
        let erro = String::from_utf8_lossy(&s.stderr);
        Err(erro
            .lines()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

pub fn comando_ligar_claude(url_mcp: &str, chave: &str) -> Vec<String> {
    [
        "claude",
        "mcp",
        "add",
        "--transport",
        "http",
        "--scope",
        "user",
        "tenancy",
        url_mcp,
        "--header",
    ]
    .iter()
    .map(|s| s.to_string())
    .chain([format!("Authorization: Bearer {chave}")])
    .collect()
}

/// Para "Mostrar o comando por trás": a chave vira ••••.
pub fn mascarar(argv: &[String]) -> String {
    argv.iter()
        .map(|a| match a.find("Bearer ") {
            Some(i) => format!("\"{}Bearer ••••\"", &a[..i]),
            None => a.clone(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn ligar_claude(casa: &Path, url_mcp: &str, chave: &str) -> Result<(), String> {
    let argv = comando_ligar_claude(url_mcp, chave);
    let bin = casa.join(".local/bin/claude");
    let s = Command::new(bin)
        .args(&argv[1..])
        .output()
        .map_err(|e| e.to_string())?;
    if s.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&s.stderr).trim().to_string())
    }
}

pub fn ligar_gemini(settings: &Path, url_mcp: &str, chave: &str) -> std::io::Result<()> {
    let mut v: serde_json::Value = std::fs::read_to_string(settings)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    if !v["mcpServers"].is_object() {
        v["mcpServers"] = serde_json::json!({});
    }
    v["mcpServers"]["tenancy"] = serde_json::json!({ "httpUrl": url_mcp, "headers": { "Authorization": format!("Bearer {chave}") } });
    if let Some(p) = settings.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::write(settings, serde_json::to_vec_pretty(&v)?)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn cada_ia_sabe_sua_cli() {
        assert_eq!(Ia::de_id("chatgpt"), Some(Ia::Chatgpt));
        assert_eq!(Ia::Chatgpt.cli(), "codex");
        assert_eq!(Ia::Gemini.nome_cli(), "Gemini CLI");
        assert_eq!(Ia::de_id("outra"), None);
    }

    #[test]
    fn instalar_vai_para_a_casa_da_pessoa() {
        let casa = Path::new("/home/luan");
        assert_eq!(
            Ia::Chatgpt.comando_instalar(casa),
            vec![
                "npm",
                "install",
                "-g",
                "--prefix",
                "/home/luan/.local",
                "@openai/codex"
            ]
        );
        assert_eq!(
            Ia::Claude.comando_instalar(casa)[2],
            "curl -fsSL https://claude.ai/install.sh | bash"
        );
        assert!(!instalada(Ia::Claude, casa));
    }

    #[test]
    fn chave_nunca_aparece_no_comando_por_tras() {
        let c = comando_ligar_claude("https://tenancy.novasce.com/mcp", "segredo123");
        assert!(c.contains(&"Authorization: Bearer segredo123".to_string()));
        let m = mascarar(&c);
        assert!(!m.contains("segredo123"));
        assert!(m.contains("Bearer ••••"));
    }

    #[test]
    fn gemini_ganha_o_mcp_sem_perder_o_resto() {
        let d = tempfile::tempdir().unwrap();
        let s = d.path().join(".gemini/settings.json");
        std::fs::create_dir_all(s.parent().unwrap()).unwrap();
        std::fs::write(
            &s,
            r#"{"theme":"Default","mcpServers":{"outro":{"command":"x"}}}"#,
        )
        .unwrap();
        ligar_gemini(&s, "https://t/mcp", "k").unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&s).unwrap()).unwrap();
        assert_eq!(v["theme"], "Default");
        assert_eq!(v["mcpServers"]["outro"]["command"], "x");
        assert_eq!(v["mcpServers"]["tenancy"]["httpUrl"], "https://t/mcp");
        assert_eq!(
            v["mcpServers"]["tenancy"]["headers"]["Authorization"],
            "Bearer k"
        );
    }
}
