//! O brain no GitHub. Varredura antes de cada envio; conflito vira segunda cópia com o nome da máquina.
use serde::Deserialize;
use std::path::Path;
use std::process::{Command, Output};

#[derive(Debug, Clone, PartialEq)]
pub struct Achado {
    pub arquivo: String,
    pub regra: String,
}

pub trait Varredura {
    fn varrer_preparados(&self, repo: &Path) -> Result<Vec<Achado>, String>;
}

pub struct Gitleaks;

impl Varredura for Gitleaks {
    fn varrer_preparados(&self, repo: &Path) -> Result<Vec<Achado>, String> {
        let relatorio =
            std::env::temp_dir().join(format!("novasce-gitleaks-{}.json", std::process::id()));
        let s = Command::new("gitleaks")
            .args([
                "git",
                "--pre-commit",
                "--staged",
                "--redact",
                "--no-banner",
                "--exit-code",
                "0",
                "--report-format",
                "json",
                "--report-path",
            ])
            .arg(&relatorio)
            .arg(repo)
            .output()
            .map_err(|e| format!("gitleaks: {e}"))?;
        if !s.status.success() {
            return Err(String::from_utf8_lossy(&s.stderr).trim().to_string());
        }
        let texto = std::fs::read_to_string(&relatorio).unwrap_or_default();
        let _ = std::fs::remove_file(&relatorio);
        ler_relatorio(&texto)
    }
}

pub fn ler_relatorio(json: &str) -> Result<Vec<Achado>, String> {
    #[derive(Deserialize)]
    struct F {
        #[serde(rename = "File")]
        file: String,
        #[serde(rename = "RuleID")]
        rule: String,
    }
    if json.trim().is_empty() {
        return Ok(vec![]);
    }
    let lista: Vec<F> =
        serde_json::from_str(json).map_err(|e| format!("relatório do gitleaks ilegível: {e}"))?;
    let mut saida: Vec<Achado> = Vec::new();
    for f in lista {
        if !saida.iter().any(|a| a.arquivo == f.file) {
            saida.push(Achado {
                arquivo: f.file,
                regra: f.rule,
            });
        }
    }
    Ok(saida)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErroBrain {
    SemBrain,
    SemRede,
    Varredura(String),
    Git(String),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResumoBrain {
    pub enviou: bool,
    pub retidos: Vec<Achado>,
    pub copias: Vec<String>,
    pub erro: Option<ErroBrain>,
}

fn git(repo: &Path, args: &[&str]) -> Result<Output, String> {
    Command::new("git")
        .args(args)
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| e.to_string())
}

fn git_ok(repo: &Path, args: &[&str]) -> Result<String, String> {
    let s = git(repo, args)?;
    if s.status.success() {
        Ok(String::from_utf8_lossy(&s.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&s.stderr).trim().to_string())
    }
}

pub fn nome_da_copia(caminho: &str, maquina: &str) -> String {
    let p = Path::new(caminho);
    let base = p
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = p
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let nome = format!("{base} ({maquina}){ext}");
    match p
        .parent()
        .map(|x| x.to_string_lossy().to_string())
        .filter(|x| !x.is_empty())
    {
        Some(d) => format!("{d}/{nome}"),
        None => nome,
    }
}

fn resolver_conflitos(repo: &Path, maquina: &str) -> Result<Vec<String>, String> {
    let lista = git_ok(repo, &["diff", "--name-only", "--diff-filter=U"])?;
    let mut copias = Vec::new();
    for caminho in lista.lines().filter(|l| !l.is_empty()) {
        let versao = |n: u8| {
            git(repo, &["show", &format!(":{n}:{caminho}")])
                .ok()
                .filter(|o| o.status.success())
                .map(|o| o.stdout)
        };
        match (versao(2), versao(3)) {
            (Some(nossa), Some(_)) => {
                let copia = nome_da_copia(caminho, maquina);
                std::fs::write(repo.join(&copia), nossa).map_err(|e| e.to_string())?;
                git_ok(repo, &["checkout", "--theirs", "--", caminho])?;
                git_ok(repo, &["add", "--", caminho, &copia])?;
                copias.push(copia);
            }
            (Some(nossa), None) => {
                std::fs::write(repo.join(caminho), nossa).map_err(|e| e.to_string())?;
                git_ok(repo, &["add", "--", caminho])?;
            }
            (None, Some(_)) => {
                git_ok(repo, &["checkout", "--theirs", "--", caminho])?;
                git_ok(repo, &["add", "--", caminho])?;
            }
            (None, None) => {
                git_ok(repo, &["rm", "-q", "--cached", "--", caminho])?;
            }
        }
    }
    git_ok(repo, &["commit", "-q", "--no-edit"])?;
    Ok(copias)
}

pub fn sincronizar_brain(
    repo: &Path,
    maquina: &str,
    varredura: &dyn Varredura,
    agora: &str,
) -> ResumoBrain {
    let mut r = ResumoBrain::default();
    if !repo.join(".git").exists() {
        r.erro = Some(ErroBrain::SemBrain);
        return r;
    }
    if let Err(e) = git_ok(repo, &["add", "-A"]) {
        r.erro = Some(ErroBrain::Git(e));
        return r;
    }
    match varredura.varrer_preparados(repo) {
        Ok(achados) => {
            for a in &achados {
                let _ = git_ok(repo, &["reset", "-q", "--", &a.arquivo]);
            }
            r.retidos = achados;
        }
        Err(e) => {
            let _ = git_ok(repo, &["reset", "-q"]);
            r.erro = Some(ErroBrain::Varredura(e));
            return r;
        }
    }
    if !git_ok(repo, &["diff", "--cached", "--name-only"])
        .unwrap_or_default()
        .is_empty()
    {
        if let Err(e) = git_ok(
            repo,
            &[
                "commit",
                "-q",
                "-m",
                &format!("Salvo de {maquina} em {agora}"),
            ],
        ) {
            r.erro = Some(ErroBrain::Git(e));
            return r;
        }
    }
    let ramo =
        git_ok(repo, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|_| "main".into());
    if git_ok(repo, &["fetch", "-q", "origin"]).is_err() {
        r.erro = Some(ErroBrain::SemRede);
        return r;
    }
    let remoto = format!("origin/{ramo}");
    let conta = |faixa: String| {
        git_ok(repo, &["rev-list", "--count", &faixa])
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
    };
    if conta(format!("HEAD..{remoto}")).unwrap_or(0) > 0
        && git_ok(repo, &["merge", "--no-edit", "-q", &remoto]).is_err()
    {
        match resolver_conflitos(repo, maquina) {
            Ok(c) => r.copias = c,
            Err(e) => {
                let _ = git_ok(repo, &["merge", "--abort"]);
                r.erro = Some(ErroBrain::Git(e));
                return r;
            }
        }
    }
    if conta(format!("{remoto}..HEAD")).unwrap_or(1) > 0 {
        if git_ok(repo, &["push", "-q", "origin", &format!("HEAD:{ramo}")]).is_err() {
            r.erro = Some(ErroBrain::SemRede);
            return r;
        }
        r.enviou = true;
    }
    r
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::path::PathBuf;

    struct SemAchados;
    impl Varredura for SemAchados {
        fn varrer_preparados(&self, _: &Path) -> Result<Vec<Achado>, String> {
            Ok(vec![])
        }
    }
    struct AchaSenha;
    impl Varredura for AchaSenha {
        fn varrer_preparados(&self, _: &Path) -> Result<Vec<Achado>, String> {
            Ok(vec![Achado {
                arquivo: "senha.md".into(),
                regra: "generic-api-key".into(),
            }])
        }
    }
    struct Quebrada;
    impl Varredura for Quebrada {
        fn varrer_preparados(&self, _: &Path) -> Result<Vec<Achado>, String> {
            Err("gitleaks não encontrado".into())
        }
    }

    fn g(dir: &Path, args: &[&str]) {
        let s = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(
            s.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&s.stderr)
        );
    }

    fn montar() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let remoto = d.path().join("remoto.git");
        g(
            d.path(),
            &[
                "init",
                "-q",
                "--bare",
                "-b",
                "main",
                remoto.to_str().unwrap(),
            ],
        );
        let a = d.path().join("a");
        g(
            d.path(),
            &["clone", "-q", remoto.to_str().unwrap(), a.to_str().unwrap()],
        );
        g(&a, &["config", "user.name", "Teste"]);
        g(&a, &["config", "user.email", "t@t"]);
        g(&a, &["symbolic-ref", "HEAD", "refs/heads/main"]);
        (d, remoto, a)
    }

    fn clonar(remoto: &Path, destino: &Path) {
        g(
            remoto.parent().unwrap(),
            &[
                "clone",
                "-q",
                remoto.to_str().unwrap(),
                destino.to_str().unwrap(),
            ],
        );
        g(destino, &["config", "user.name", "Teste"]);
        g(destino, &["config", "user.email", "t@t"]);
    }

    #[test]
    fn copia_leva_o_nome_da_maquina() {
        assert_eq!(
            nome_da_copia("notas/a.md", "notebook-do-luan"),
            "notas/a (notebook-do-luan).md"
        );
        assert_eq!(nome_da_copia("LEIA", "sede"), "LEIA (sede)");
    }

    #[test]
    fn salva_e_envia() {
        let (_d, remoto, a) = montar();
        std::fs::write(a.join("nota.md"), "oi").unwrap();
        let r = sincronizar_brain(&a, "notebook-a", &SemAchados, "2026-10-23T14:00:00Z");
        assert!(r.enviou && r.erro.is_none(), "{r:?}");
        let b = remoto.parent().unwrap().join("b");
        clonar(&remoto, &b);
        assert_eq!(std::fs::read_to_string(b.join("nota.md")).unwrap(), "oi");
    }

    #[test]
    fn conflito_vira_copia_com_o_nome_da_maquina() {
        let (_d, remoto, a) = montar();
        std::fs::create_dir_all(a.join("notas")).unwrap();
        std::fs::write(a.join("notas/a.md"), "base\n").unwrap();
        sincronizar_brain(&a, "notebook-a", &SemAchados, "t1");
        let b = remoto.parent().unwrap().join("b");
        clonar(&remoto, &b);
        std::fs::write(a.join("notas/a.md"), "versão do A\n").unwrap();
        std::fs::write(b.join("notas/a.md"), "versão do B\n").unwrap();
        assert!(sincronizar_brain(&a, "notebook-a", &SemAchados, "t2").enviou);
        let r = sincronizar_brain(&b, "notebook-b", &SemAchados, "t3");
        assert!(r.erro.is_none(), "{r:?}");
        assert_eq!(r.copias, vec!["notas/a (notebook-b).md".to_string()]);
        assert_eq!(
            std::fs::read_to_string(b.join("notas/a.md")).unwrap(),
            "versão do A\n"
        );
        assert_eq!(
            std::fs::read_to_string(b.join("notas/a (notebook-b).md")).unwrap(),
            "versão do B\n"
        );
        assert!(r.enviou);
    }

    #[test]
    fn arquivo_com_senha_fica_so_aqui() {
        let (_d, remoto, a) = montar();
        std::fs::write(a.join("senha.md"), "chave").unwrap();
        std::fs::write(a.join("ok.md"), "nota").unwrap();
        let r = sincronizar_brain(&a, "n", &AchaSenha, "t");
        assert_eq!(r.retidos.len(), 1);
        assert!(r.enviou);
        let b = remoto.parent().unwrap().join("b");
        clonar(&remoto, &b);
        assert!(b.join("ok.md").exists());
        assert!(!b.join("senha.md").exists());
        assert!(a.join("senha.md").exists());
    }

    #[test]
    fn varredura_quebrada_nada_sai() {
        let (_d, _remoto, a) = montar();
        std::fs::write(a.join("x.md"), "x").unwrap();
        let r = sincronizar_brain(&a, "n", &Quebrada, "t");
        assert!(matches!(r.erro, Some(ErroBrain::Varredura(_))));
        assert!(!r.enviou);
    }

    #[test]
    fn sem_brain() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(
            sincronizar_brain(d.path(), "n", &SemAchados, "t").erro,
            Some(ErroBrain::SemBrain)
        );
    }

    #[test]
    fn le_o_relatorio_do_gitleaks() {
        let a = ler_relatorio(r#"[{"File":"notas/x.md","RuleID":"github-pat","StartLine":3},{"File":"notas/x.md","RuleID":"generic-api-key","StartLine":9}]"#).unwrap();
        assert_eq!(
            a,
            vec![Achado {
                arquivo: "notas/x.md".into(),
                regra: "github-pat".into()
            }]
        );
        assert_eq!(ler_relatorio("").unwrap(), vec![]);
    }
}
