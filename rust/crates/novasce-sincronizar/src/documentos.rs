//! Documentos ↔ Google Drive com rclone bisync. Conflito nunca sobrescreve.
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const FILTROS: &str = "- .~lock.*#\n- ~$*\n- *.tmp\n- .directory\n- .Trash-*/**\n";

pub struct ConfigDocs {
    pub local: PathBuf,
    pub remoto: String,
    pub maquina: String,
    pub filtros: PathBuf,
    pub cache_bisync: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErroDocs {
    SemRemoto,
    PrecisaResync,
    SemRede,
    Falhou(String),
}

pub fn remoto_de(pasta: &str) -> String {
    format!("gdrive:{pasta}")
}

pub fn precisa_resync(c: &ConfigDocs) -> bool {
    !std::fs::read_dir(&c.cache_bisync)
        .map(|it| {
            it.flatten()
                .any(|e| e.file_name().to_string_lossy().ends_with(".path1.lst"))
        })
        .unwrap_or(false)
}

pub fn args_bisync(c: &ConfigDocs, primeira_vez: bool) -> Vec<String> {
    let mut a: Vec<String> = vec![
        "bisync".into(),
        c.local.to_string_lossy().into(),
        c.remoto.clone(),
    ];
    for x in [
        "--conflict-resolve",
        "path2",
        "--conflict-loser",
        "num",
        "--conflict-suffix",
        &c.maquina,
        "--suffix-keep-extension",
        "--resilient",
        "--recover",
        "--max-lock",
        "2m",
        "--create-empty-src-dirs",
        "--use-json-log",
        "--stats",
        "2s",
        "--stats-log-level",
        "NOTICE",
    ] {
        a.push(x.into());
    }
    a.push("--filters-file".into());
    a.push(c.filtros.to_string_lossy().into());
    if primeira_vez {
        a.push("--resync".into());
    }
    a
}

pub fn progresso(linha: &str) -> Option<u8> {
    let v: serde_json::Value = serde_json::from_str(linha).ok()?;
    let bytes = v["stats"]["bytes"].as_f64()?;
    let total = v["stats"]["totalBytes"].as_f64()?;
    (total > 0.0).then(|| ((bytes / total) * 100.0).clamp(0.0, 100.0) as u8)
}

pub fn classificar(saida: &str) -> ErroDocs {
    if saida.contains("--resync") {
        ErroDocs::PrecisaResync
    } else if [
        "no such host",
        "dial tcp",
        "network is unreachable",
        "i/o timeout",
    ]
    .iter()
    .any(|t| saida.contains(t))
    {
        ErroDocs::SemRede
    } else {
        ErroDocs::Falhou(
            saida
                .lines()
                .rev()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("")
                .chars()
                .take(200)
                .collect(),
        )
    }
}

/// Roda um bisync. `ao_progresso` recebe o percentual enquanto sobe e desce.
pub fn rodar(c: &ConfigDocs, mut ao_progresso: impl FnMut(u8)) -> Result<(), ErroDocs> {
    std::fs::create_dir_all(&c.local).map_err(|e| ErroDocs::Falhou(e.to_string()))?;
    std::fs::write(&c.filtros, FILTROS).map_err(|e| ErroDocs::Falhou(e.to_string()))?;
    let mut filho = Command::new("rclone")
        .args(args_bisync(c, precisa_resync(c)))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ErroDocs::Falhou(e.to_string()))?;
    let mut tudo = String::new();
    if let Some(err) = filho.stderr.take() {
        for linha in BufReader::new(err).lines().map_while(Result::ok) {
            if let Some(p) = progresso(&linha) {
                ao_progresso(p);
            }
            tudo.push_str(&linha);
            tudo.push('\n');
        }
    }
    let status = filho.wait().map_err(|e| ErroDocs::Falhou(e.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(classificar(&tudo))
    }
}

pub fn cache_bisync_padrao(casa: &Path) -> PathBuf {
    casa.join(".cache/rclone/bisync")
}

#[cfg(test)]
mod testes {
    use super::*;

    fn cfg(d: &Path) -> ConfigDocs {
        ConfigDocs {
            local: d.join("Documentos"),
            remoto: remoto_de("Novasce OS/Documentos"),
            maquina: "notebook-do-luan".into(),
            filtros: d.join("filtros.txt"),
            cache_bisync: d.join("bisync"),
        }
    }

    #[test]
    fn argumentos_do_bisync() {
        let d = tempfile::tempdir().unwrap();
        let c = cfg(d.path());
        let a = args_bisync(&c, false);
        assert_eq!(a[0], "bisync");
        assert_eq!(a[2], "gdrive:Novasce OS/Documentos");
        for par in [
            ["--conflict-resolve", "path2"],
            ["--conflict-loser", "num"],
            ["--conflict-suffix", "notebook-do-luan"],
            ["--max-lock", "2m"],
        ] {
            let i = a.iter().position(|x| x == par[0]).unwrap();
            assert_eq!(a[i + 1], par[1]);
        }
        assert!(
            a.contains(&"--suffix-keep-extension".to_string())
                && a.contains(&"--use-json-log".to_string())
        );
        assert!(!a.contains(&"--resync".to_string()));
        assert!(args_bisync(&c, true).contains(&"--resync".to_string()));
    }

    #[test]
    fn primeira_vez_ate_existir_a_listagem() {
        let d = tempfile::tempdir().unwrap();
        let c = cfg(d.path());
        assert!(precisa_resync(&c));
        std::fs::create_dir_all(&c.cache_bisync).unwrap();
        std::fs::write(
            c.cache_bisync
                .join("home_luan_Documentos..gdrive_Novasce OS_Documentos.path1.lst"),
            "",
        )
        .unwrap();
        assert!(!precisa_resync(&c));
    }

    #[test]
    fn progresso_do_log_json() {
        assert_eq!(
            progresso(
                r#"{"level":"notice","msg":"...","stats":{"bytes":620,"totalBytes":1000},"time":"x"}"#
            ),
            Some(62)
        );
        assert_eq!(
            progresso(r#"{"level":"notice","stats":{"bytes":0,"totalBytes":0}}"#),
            None
        );
        assert_eq!(progresso("texto solto"), None);
    }

    #[test]
    fn erro_do_rclone_vira_motivo() {
        assert_eq!(classificar("ERROR : Bisync critical error: cannot find prior Path1 or Path2 listings, likely due to critical error on prior run. Must run --resync to recover."), ErroDocs::PrecisaResync);
        assert_eq!(
            classificar(
                "Failed to create file system: dial tcp: lookup www.googleapis.com: no such host"
            ),
            ErroDocs::SemRede
        );
        assert!(matches!(classificar("outra coisa"), ErroDocs::Falhou(_)));
    }

    #[test]
    fn filtros_ignoram_arquivo_temporario() {
        assert!(FILTROS.contains("- .~lock.*#") && FILTROS.contains("- ~$*"));
    }
}
