//! O navegador da conta Sede: perfil descartável, apagado quando a Sala volta.
use novasce_casa::config_casa::SecaoSede;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn perfil(casa: &Path, cfg: &SecaoSede) -> PathBuf {
    casa.join(&cfg.perfil_navegador)
}

pub fn args_navegador(cfg: &SecaoSede, perfil: &Path, extra: &[String]) -> Vec<String> {
    let mut a = cfg.navegador.clone();
    a.push(format!("--user-data-dir={}", perfil.display()));
    a.push("--no-first-run".into());
    a.push("--no-default-browser-check".into());
    a.extend(extra.iter().cloned());
    a
}

pub fn padrao_processo(perfil: &Path) -> String {
    format!("user-data-dir={}", perfil.display())
}

pub fn aberto(perfil: &Path) -> bool {
    Command::new("pgrep")
        .args(["-f", &padrao_processo(perfil)])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Fecha o navegador da Sede e apaga o perfil inteiro: nada fica para trás.
pub fn esquecer(perfil: &Path) -> Result<(), String> {
    let padrao = padrao_processo(perfil);
    let _ = Command::new("pkill")
        .args(["-TERM", "-f", &padrao])
        .status();
    for _ in 0..30 {
        if !aberto(perfil) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let _ = Command::new("pkill")
        .args(["-KILL", "-f", &padrao])
        .status();
    match std::fs::remove_dir_all(perfil) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use novasce_casa::config_casa::SecaoSede;

    #[test]
    fn perfil_descartavel_e_argumentos() {
        let cfg = SecaoSede::default();
        let p = perfil(Path::new("/var/home/sede"), &cfg);
        assert_eq!(
            p,
            PathBuf::from("/var/home/sede/.var/app/com.google.Chrome/novasce-esquece")
        );
        let a = args_navegador(&cfg, &p, &["https://meet.google.com".into()]);
        assert_eq!(&a[..3], &["flatpak", "run", "com.google.Chrome"]);
        assert!(a.contains(
            &"--user-data-dir=/var/home/sede/.var/app/com.google.Chrome/novasce-esquece"
                .to_string()
        ));
        assert_eq!(a.last().unwrap(), "https://meet.google.com");
        assert_eq!(
            padrao_processo(&p),
            "user-data-dir=/var/home/sede/.var/app/com.google.Chrome/novasce-esquece"
        );
    }
}
