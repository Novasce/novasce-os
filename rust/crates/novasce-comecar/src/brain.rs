//! O brain pessoal: privado, na conta da pessoa, em ~/Brain. Nada privado da casa vem da imagem.
use crate::github::{ErroGithub, GithubApi};
use serde::Serialize;
use std::path::Path;
use std::process::Command;

pub trait Git {
    fn clonar(&self, url: &str, destino: &Path) -> Result<(), String>;
}

pub struct GitReal;

impl Git for GitReal {
    fn clonar(&self, url: &str, destino: &Path) -> Result<(), String> {
        let s = Command::new("git")
            .args(["clone", "--quiet", url])
            .arg(destino)
            .output()
            .map_err(|e| e.to_string())?;
        if s.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&s.stderr).trim().to_string())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrigemBrain {
    Modelo,
    Vazio,
    JaExistia,
}

pub fn nome_do_brain(login: &str) -> String {
    format!("{}-brain", login.to_lowercase())
}

const README: &str = "# Meu brain\n\nAs suas notas. Ficam no seu GitHub, privadas, e salvam sozinhas a cada poucos minutos.\n";
const AGENTS: &str = "# Como trabalhar neste brain\n\nEste é um brain pessoal. Ele é privado e nunca é lido por um espaço da Novasce.\nAntes de cada envio, uma varredura procura senhas e segura o arquivo que tiver uma.\n";
/// O sincronizar faz `git add -A`: sem isto, cada clique no Obsidian vira commit e conflito entre máquinas.
/// É o mesmo trecho do modelo Novasce/brain-base.
const GITIGNORE: &str = "# estado da janela do Obsidian, que muda a cada clique\n.obsidian/workspace.json\n.obsidian/workspace-mobile.json\n.trash/\n";

fn kit_minimo(destino: &Path) -> std::io::Result<()> {
    for (nome, texto) in [
        ("README.md", README),
        ("AGENTS.md", AGENTS),
        ("CLAUDE.md", "@AGENTS.md\n"),
        // o Gemini CLI só lê o GEMINI.md, e o ligar_gemini não muda isso
        ("GEMINI.md", "@./AGENTS.md\n"),
        (".gitignore", GITIGNORE),
    ] {
        let p = destino.join(nome);
        if !p.exists() {
            std::fs::write(p, texto)?;
        }
    }
    Ok(())
}

/// `pausa_ms` é a espera entre conferências depois do "generate" (0 nos testes, 1000 no app).
pub fn preparar_brain(
    api: &GithubApi,
    modelo: &str,
    login: &str,
    destino: &Path,
    git: &dyn Git,
    pausa_ms: u64,
) -> Result<OrigemBrain, ErroGithub> {
    let nome = nome_do_brain(login);
    let origem = if api.repo_existe(login, &nome)? {
        OrigemBrain::JaExistia
    } else if !modelo.trim().is_empty() && api.criar_do_modelo(modelo.trim(), login, &nome).is_ok()
    {
        for _ in 0..15 {
            if api.tem_commit(login, &nome).unwrap_or(false) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(pausa_ms));
        }
        OrigemBrain::Modelo
    } else {
        api.criar_vazio(&nome)?;
        OrigemBrain::Vazio
    };
    if !destino.join(".git").exists() {
        git.clonar(&format!("https://github.com/{login}/{nome}.git"), destino)
            .map_err(|m| ErroGithub::Recusado {
                codigo: 0,
                mensagem: m,
            })?;
    }
    if origem == OrigemBrain::Vazio {
        kit_minimo(destino).map_err(|e| ErroGithub::Recusado {
            codigo: 0,
            mensagem: e.to_string(),
        })?;
    }
    Ok(origem)
}

pub fn caminho_config_obsidian(casa: &Path) -> std::path::PathBuf {
    casa.join(".var/app/md.obsidian.Obsidian/config/obsidian/obsidian.json")
}

pub fn registrar_no_obsidian(config: &Path, cofre: &Path, agora_ms: i64) -> std::io::Result<()> {
    let mut v: serde_json::Value = std::fs::read_to_string(config)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    if !v["vaults"].is_object() {
        v["vaults"] = serde_json::json!({});
    }
    let alvo = cofre.to_string_lossy().to_string();
    let vaults = v["vaults"].as_object_mut().unwrap();
    for x in vaults.values_mut() {
        x["open"] = serde_json::Value::Bool(false);
    }
    let id = vaults
        .iter()
        .find(|(_, x)| x["path"] == alvo)
        .map(|(k, _)| k.clone())
        .unwrap_or_else(|| {
            let h = alvo.bytes().fold(0xcbf29ce484222325u64, |h, b| {
                (h ^ b as u64).wrapping_mul(0x100000001b3)
            });
            format!("{h:016x}")
        });
    vaults.insert(
        id,
        serde_json::json!({ "path": alvo, "ts": agora_ms, "open": true }),
    );
    if let Some(p) = config.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::write(config, serde_json::to_vec_pretty(&v)?)
}

/// git config global: o token vem do chaveiro pelo ajudante da casa; nome e e-mail do Tenancy.
pub fn configurar_git(nome: &str, email: &str) -> Result<(), String> {
    let g = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .output()
            .map_err(|e| e.to_string())
    };
    let chave = "credential.https://github.com.helper";
    g(&["config", "--global", "--unset-all", chave])?;
    g(&["config", "--global", "--add", chave, ""])?;
    g(&[
        "config",
        "--global",
        "--add",
        chave,
        "/usr/libexec/novasce/git-credencial",
    ])?;
    for (k, v) in [("user.name", nome), ("user.email", email)] {
        let atual = g(&["config", "--global", k])?;
        if atual.stdout.is_empty() && !v.is_empty() {
            g(&["config", "--global", k, v])?;
        }
    }
    Ok(())
}

/// Protocolo de ajudante de credencial do git: responde só "get" para github.com.
pub fn resposta_credencial(operacao: &str, entrada: &str, token: Option<&str>) -> String {
    let host_github = entrada.lines().any(|l| l.trim() == "host=github.com");
    match (operacao, host_github, token) {
        ("get", true, Some(t)) => format!("username=x-access-token\npassword={t}\n"),
        _ => String::new(),
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    struct GitFalso(RefCell<Vec<String>>);
    impl Git for GitFalso {
        fn clonar(&self, url: &str, destino: &Path) -> Result<(), String> {
            self.0.borrow_mut().push(url.into());
            std::fs::create_dir_all(destino.join(".git")).map_err(|e| e.to_string())
        }
    }

    #[test]
    fn nome_minusculo() {
        assert_eq!(nome_do_brain("Luan"), "luan-brain");
    }

    #[test]
    fn ja_existia_so_clona() {
        let mut s = mockito::Server::new();
        s.mock("GET", "/repos/luan/luan-brain")
            .with_status(200)
            .with_body("{}")
            .create();
        let d = tempfile::tempdir().unwrap();
        let git = GitFalso::default();
        let o = preparar_brain(
            &GithubApi::novo(&s.url(), "t".into()),
            "Novasce/brain-base",
            "luan",
            &d.path().join("Brain"),
            &git,
            0,
        )
        .unwrap();
        assert_eq!(o, OrigemBrain::JaExistia);
        assert_eq!(git.0.borrow()[0], "https://github.com/luan/luan-brain.git");
    }

    #[test]
    fn modelo_inacessivel_cai_para_vazio_com_kit_minimo() {
        let mut s = mockito::Server::new();
        s.mock("GET", "/repos/luan/luan-brain")
            .with_status(404)
            .create();
        s.mock("POST", "/repos/Novasce/brain-base/generate")
            .with_status(404)
            .create();
        let m = s
            .mock("POST", "/user/repos")
            .with_status(201)
            .with_body("{}")
            .create();
        let d = tempfile::tempdir().unwrap();
        let destino = d.path().join("Brain");
        let o = preparar_brain(
            &GithubApi::novo(&s.url(), "t".into()),
            "Novasce/brain-base",
            "luan",
            &destino,
            &GitFalso::default(),
            0,
        )
        .unwrap();
        m.assert();
        assert_eq!(o, OrigemBrain::Vazio);
        assert!(destino.join("AGENTS.md").exists() && destino.join("CLAUDE.md").exists());
    }

    #[test]
    fn kit_minimo_ignora_o_estado_da_janela_do_obsidian() {
        let d = tempfile::tempdir().unwrap();
        kit_minimo(d.path()).unwrap();
        let ignorados = std::fs::read_to_string(d.path().join(".gitignore")).unwrap();
        assert!(ignorados.lines().any(|l| l == ".obsidian/workspace.json"));
        assert!(ignorados.lines().any(|l| l == ".trash/"));
    }

    #[test]
    fn kit_minimo_leva_o_gemini_ao_agents() {
        let d = tempfile::tempdir().unwrap();
        kit_minimo(d.path()).unwrap();
        let gemini = std::fs::read_to_string(d.path().join("GEMINI.md")).unwrap();
        assert_eq!(gemini.trim(), "@./AGENTS.md");
    }

    #[test]
    fn modelo_espera_ter_commit() {
        let mut s = mockito::Server::new();
        s.mock("GET", "/repos/luan/luan-brain")
            .with_status(404)
            .create();
        s.mock("POST", "/repos/Novasce/brain-base/generate")
            .with_status(201)
            .with_body("{}")
            .create();
        s.mock("GET", "/repos/luan/luan-brain/commits?per_page=1")
            .with_status(200)
            .with_body("[]")
            .create();
        let d = tempfile::tempdir().unwrap();
        let o = preparar_brain(
            &GithubApi::novo(&s.url(), "t".into()),
            "Novasce/brain-base",
            "luan",
            &d.path().join("Brain"),
            &GitFalso::default(),
            0,
        )
        .unwrap();
        assert_eq!(o, OrigemBrain::Modelo);
    }

    #[test]
    fn obsidian_ganha_o_cofre_sem_perder_os_outros() {
        let d = tempfile::tempdir().unwrap();
        let cfg = d.path().join("obsidian.json");
        std::fs::write(
            &cfg,
            r#"{"vaults":{"abc":{"path":"/home/luan/Outro","ts":1}}}"#,
        )
        .unwrap();
        registrar_no_obsidian(&cfg, Path::new("/home/luan/Brain"), 5).unwrap();
        registrar_no_obsidian(&cfg, Path::new("/home/luan/Brain"), 6).unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
        let vaults = v["vaults"].as_object().unwrap();
        assert_eq!(vaults.len(), 2);
        assert!(vaults
            .values()
            .any(|x| x["path"] == "/home/luan/Brain" && x["open"] == true));
    }

    #[test]
    fn ajudante_de_credencial_so_responde_ao_github() {
        assert_eq!(
            resposta_credencial("get", "protocol=https\nhost=github.com\n", Some("gho_x")),
            "username=x-access-token\npassword=gho_x\n"
        );
        assert_eq!(
            resposta_credencial("get", "protocol=https\nhost=gitlab.com\n", Some("gho_x")),
            ""
        );
        assert_eq!(
            resposta_credencial("store", "host=github.com\n", Some("gho_x")),
            ""
        );
        assert_eq!(resposta_credencial("get", "host=github.com\n", None), "");
    }
}
