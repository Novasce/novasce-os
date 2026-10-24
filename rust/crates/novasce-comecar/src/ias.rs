//! As IAs da pessoa: app web para conversar, CLI por pessoa para trabalhar no brain.
use novasce_casa::arquivo_privado;
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

/// A entrada do Tenancy no `mcpServers` do Claude Code, no mesmo formato que o próprio
/// `claude mcp add --transport http --scope user` grava no `~/.claude.json`.
///
/// Não passamos pelo `claude mcp add`: ele só aceita o cabeçalho com a chave na linha de comando
/// (`--header`, e `add-json` leva o JSON inteiro), e sem `hidepid` qualquer conta da máquina lê
/// /proc/<pid>/cmdline. Escrevemos o arquivo da pessoa nós mesmos, 0600, e o resto dele fica como está.
pub fn entrada_mcp_claude(url_mcp: &str, chave: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "http",
        "url": url_mcp,
        "headers": { "Authorization": format!("Bearer {chave}") },
    })
}

/// Para "Mostrar o comando por trás": o que de fato é gravado, com a chave vista como ••••.
pub fn mascarar(entrada: &serde_json::Value) -> String {
    let mut m = entrada.clone();
    if let Some(cabecalhos) = m.get_mut("headers").and_then(|h| h.as_object_mut()) {
        for valor in cabecalhos.values_mut() {
            *valor = serde_json::json!("Bearer ••••");
        }
    }
    format!("~/.claude.json: mcpServers.tenancy = {m}")
}

fn arquivo_do_claude(casa: &Path) -> std::path::PathBuf {
    casa.join(".claude.json")
}

pub fn ligar_claude(casa: &Path, url_mcp: &str, chave: &str) -> Result<(), String> {
    ligar_claude_em(&arquivo_do_claude(casa), url_mcp, chave)
}

fn ligar_claude_em(arquivo: &Path, url_mcp: &str, chave: &str) -> Result<(), String> {
    // O ~/.claude.json guarda o estado inteiro do Claude Code: arquivo que não lemos direito
    // não se reescreve, senão a pessoa perde o que já tinha.
    let mut v: serde_json::Value = match std::fs::read_to_string(arquivo) {
        Ok(t) if t.trim().is_empty() => serde_json::json!({}),
        Ok(t) => serde_json::from_str(&t)
            .ok()
            .filter(serde_json::Value::is_object)
            .ok_or(
                "O arquivo do Claude Code (~/.claude.json) não está legível. Nada foi mudado.",
            )?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => serde_json::json!({}),
        Err(e) => return Err(e.to_string()),
    };
    if !v["mcpServers"].is_object() {
        v["mcpServers"] = serde_json::json!({});
    }
    v["mcpServers"]["tenancy"] = entrada_mcp_claude(url_mcp, chave);
    let json = serde_json::to_vec_pretty(&v).map_err(|e| e.to_string())?;
    arquivo_privado::gravar(arquivo, &json).map_err(|e| e.to_string())
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
    arquivo_privado::gravar(settings, &serde_json::to_vec_pretty(&v)?)
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

    fn modo(p: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(p).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn chave_nunca_aparece_no_comando_por_tras() {
        let e = entrada_mcp_claude("https://tenancy.novasce.com/mcp", "segredo123");
        assert_eq!(e["headers"]["Authorization"], "Bearer segredo123");
        let m = mascarar(&e);
        assert!(!m.contains("segredo123"));
        assert!(m.contains("Bearer ••••") && m.contains("https://tenancy.novasce.com/mcp"));
    }

    #[test]
    fn claude_ganha_o_mcp_sem_perder_o_resto_e_o_arquivo_sai_0600() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join(".claude.json");
        std::fs::write(
            &a,
            r#"{"numStartups":7,"projects":{"/x":{"allowedTools":[]}},"mcpServers":{"outro":{"command":"y"}}}"#,
        )
        .unwrap();
        std::fs::set_permissions(&a, std::os::unix::fs::PermissionsExt::from_mode(0o644)).unwrap();
        ligar_claude(d.path(), "https://t/mcp", "chave-de-teste").unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&a).unwrap()).unwrap();
        assert_eq!(v["numStartups"], 7);
        assert_eq!(v["projects"]["/x"]["allowedTools"], serde_json::json!([]));
        assert_eq!(v["mcpServers"]["outro"]["command"], "y");
        assert_eq!(
            v["mcpServers"]["tenancy"],
            entrada_mcp_claude("https://t/mcp", "chave-de-teste")
        );
        assert_eq!(modo(&a), 0o600);
    }

    #[test]
    fn o_comando_por_tras_mostra_o_que_foi_gravado() {
        let d = tempfile::tempdir().unwrap();
        ligar_claude(d.path(), "https://t/mcp", "chave-de-teste").unwrap();
        let gravado: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(d.path().join(".claude.json")).unwrap())
                .unwrap();
        let mostrado = mascarar(&gravado["mcpServers"]["tenancy"]);
        assert!(!mostrado.contains("chave-de-teste"));
        let mut esperado = gravado["mcpServers"]["tenancy"].clone();
        esperado["headers"]["Authorization"] = serde_json::json!("Bearer ••••");
        assert_eq!(
            mostrado,
            format!("~/.claude.json: mcpServers.tenancy = {esperado}")
        );
        assert_eq!(modo(&d.path().join(".claude.json")), 0o600);
    }

    #[test]
    fn claude_sem_arquivo_nasce_e_arquivo_ilegivel_nao_e_reescrito() {
        let d = tempfile::tempdir().unwrap();
        ligar_claude(d.path(), "https://t/mcp", "k").unwrap();
        assert!(d.path().join(".claude.json").exists());
        let ruim = tempfile::tempdir().unwrap();
        let a = ruim.path().join(".claude.json");
        std::fs::write(&a, "{ meio escrito").unwrap();
        assert!(ligar_claude(ruim.path(), "https://t/mcp", "k").is_err());
        assert_eq!(std::fs::read_to_string(&a).unwrap(), "{ meio escrito");
        std::fs::write(&a, "[1,2]").unwrap();
        assert!(ligar_claude(ruim.path(), "https://t/mcp", "k").is_err());
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
        assert_eq!(modo(&s), 0o600);
    }
}
