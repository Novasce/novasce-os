//! Abrir URL, o Tenancy em janela própria e apps pelo arquivo .desktop, sem esperar.
use std::process::{Command, Stdio};

pub fn rodar_solto(argv: &[&str]) -> Result<(), String> {
    let (prog, args) = argv.split_first().ok_or("comando vazio")?;
    Command::new(prog)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("{prog}: {e}"))
}

pub fn juntar(base: &str, caminho: &str) -> String {
    if caminho.starts_with("http://") || caminho.starts_with("https://") {
        return caminho.to_string();
    }
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        caminho.trim_start_matches('/')
    )
}

pub fn abrir_url(url: &str) -> Result<(), String> {
    rodar_solto(&["xdg-open", url])
}

/// O Tenancy abre em janela própria, como o `abrir-web` do M1 (Chrome Flatpak com `--app=`);
/// sem o Chrome (primeiro boot), no navegador padrão.
pub fn abrir_tenancy(web: &str, caminho: &str) -> Result<(), String> {
    let url = juntar(web, caminho);
    let tem_chrome = Command::new("flatpak")
        .args(["info", "com.google.Chrome"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if tem_chrome {
        rodar_solto(&[
            "flatpak",
            "run",
            "com.google.Chrome",
            &format!("--app={url}"),
        ])
    } else {
        abrir_url(&url)
    }
}

/// Abre um app pelo .desktop do sistema; sem ele, a URL de reserva.
pub fn abrir_app(desktop_id: &str, url_reserva: &str) -> Result<(), String> {
    let arquivo = format!("/usr/share/applications/{desktop_id}.desktop");
    if std::path::Path::new(&arquivo).exists() {
        rodar_solto(&["kioclient", "exec", &arquivo])
    } else {
        abrir_url(url_reserva)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn junta_base_e_caminho() {
        assert_eq!(
            juntar("https://tenancy.novasce.com/", "/meu-dia"),
            "https://tenancy.novasce.com/meu-dia"
        );
        assert_eq!(
            juntar("https://tenancy.novasce.com", "cofre"),
            "https://tenancy.novasce.com/cofre"
        );
        assert_eq!(juntar("https://t", "https://outra/x"), "https://outra/x");
    }
}
