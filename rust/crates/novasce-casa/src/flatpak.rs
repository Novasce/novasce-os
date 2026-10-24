//! Recomendados da casa: instalar só para a pessoa, sem senha de administrador.
use std::process::Command;

pub fn id_valido(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 255
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

pub fn args_remoto() -> Vec<&'static str> {
    vec![
        "flatpak",
        "remote-add",
        "--user",
        "--if-not-exists",
        "flathub",
        "https://dl.flathub.org/repo/flathub.flatpakrepo",
    ]
}

pub fn args_instalar(id: &str) -> Vec<&str> {
    vec![
        "flatpak",
        "install",
        "--user",
        "--noninteractive",
        "-y",
        "flathub",
        id,
    ]
}

pub fn instalados(lista: &str, ids: &[String]) -> Vec<String> {
    let presentes: Vec<&str> = lista.lines().map(str::trim).collect();
    ids.iter()
        .filter(|i| presentes.contains(&i.as_str()))
        .cloned()
        .collect()
}

pub fn listar() -> String {
    Command::new("flatpak")
        .args(["list", "--app", "--columns=application"])
        .output()
        .map(|s| String::from_utf8_lossy(&s.stdout).into_owned())
        .unwrap_or_default()
}

pub fn instalar(id: &str) -> Result<(), String> {
    if !id_valido(id) {
        return Err("App inválido.".into());
    }
    let r = args_remoto();
    let _ = Command::new(r[0]).args(&r[1..]).output();
    let a = args_instalar(id);
    let s = Command::new(a[0])
        .args(&a[1..])
        .output()
        .map_err(|e| e.to_string())?;
    if s.status.success() {
        Ok(())
    } else {
        Err(format!(
            "O app não foi instalado. Nada mudou no computador. {}",
            String::from_utf8_lossy(&s.stderr)
                .lines()
                .last()
                .unwrap_or("")
        ))
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn instala_so_para_a_pessoa() {
        assert_eq!(
            args_remoto(),
            vec![
                "flatpak",
                "remote-add",
                "--user",
                "--if-not-exists",
                "flathub",
                "https://dl.flathub.org/repo/flathub.flatpakrepo"
            ]
        );
        assert_eq!(
            args_instalar("us.zoom.Zoom"),
            vec![
                "flatpak",
                "install",
                "--user",
                "--noninteractive",
                "-y",
                "flathub",
                "us.zoom.Zoom"
            ]
        );
    }

    #[test]
    fn instalados_da_lista() {
        let saida = "org.libreoffice.LibreOffice\ncom.bitwarden.desktop\norg.kde.Platform\n";
        assert_eq!(
            instalados(
                saida,
                &["us.zoom.Zoom".into(), "com.bitwarden.desktop".into()]
            ),
            vec!["com.bitwarden.desktop".to_string()]
        );
    }

    #[test]
    fn id_estranho_e_recusado() {
        assert!(id_valido("org.gimp.GIMP"));
        assert!(!id_valido("org.gimp.GIMP; rm -rf ~"));
    }
}
