//! Extras do fim: Google Drive (rclone), assinatura de e-mail e sair do dock.
use base64::Engine;
pub use novasce_casa::config_casa::SecaoGoogle;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn args_autorizar(g: &SecaoGoogle) -> Vec<String> {
    let mut a: Vec<String> = vec!["rclone".into(), "authorize".into(), "drive".into()];
    if !g.client_id.is_empty() && !g.client_secret.is_empty() {
        a.push(g.client_id.clone());
        a.push(g.client_secret.clone());
    }
    a
}

pub fn extrair_token(saida: &str) -> Option<String> {
    let inicio = saida.find("--->")? + 4;
    let fim = saida[inicio..].find("<---")? + inicio;
    let bloco = saida[inicio..fim].trim();
    if bloco.starts_with('{') {
        return Some(bloco.to_string());
    }
    let motores = [
        base64::engine::general_purpose::URL_SAFE_NO_PAD,
        base64::engine::general_purpose::URL_SAFE,
        base64::engine::general_purpose::STANDARD,
    ];
    for m in motores {
        if let Ok(b) = m.decode(bloco) {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&b) {
                if let Some(t) = v.get("token").and_then(|t| t.as_str()) {
                    return Some(t.to_string());
                }
            }
        }
    }
    None
}

pub fn args_criar_remoto(g: &SecaoGoogle, token: &str) -> Vec<String> {
    let mut a: Vec<String> = [
        "rclone",
        "config",
        "create",
        "gdrive",
        "drive",
        "scope=drive",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    a.push(format!("token={token}"));
    if !g.client_id.is_empty() && !g.client_secret.is_empty() {
        a.push(format!("client_id={}", g.client_id));
        a.push(format!("client_secret={}", g.client_secret));
    }
    a.push("--non-interactive".into());
    a
}

pub fn remoto_configurado(listremotes: &str) -> bool {
    listremotes.lines().any(|l| l.trim() == "gdrive:")
}

/// Abre o navegador para a pessoa entrar no Google e cria o remoto. Bloqueia até ela terminar.
pub fn conectar(g: &SecaoGoogle) -> Result<(), String> {
    let a = args_autorizar(g);
    let s = Command::new(&a[0])
        .args(&a[1..])
        .output()
        .map_err(|e| e.to_string())?;
    let texto = format!(
        "{}{}",
        String::from_utf8_lossy(&s.stdout),
        String::from_utf8_lossy(&s.stderr)
    );
    let token =
        extrair_token(&texto).ok_or("O Google não devolveu a autorização. Nada foi conectado.")?;
    let c = args_criar_remoto(g, &token);
    let s = Command::new(&c[0])
        .args(&c[1..])
        .output()
        .map_err(|e| e.to_string())?;
    if s.status.success() {
        Ok(())
    } else {
        Err("A conexão com o Google não foi salva. Nada foi enviado.".into())
    }
}

pub fn conectado() -> bool {
    Command::new("rclone")
        .arg("listremotes")
        .output()
        .map(|s| remoto_configurado(&String::from_utf8_lossy(&s.stdout)))
        .unwrap_or(false)
}

pub fn salvar_assinatura(documentos: &Path, html: &str) -> std::io::Result<PathBuf> {
    let pasta = documentos.join("Novasce");
    std::fs::create_dir_all(&pasta)?;
    let p = pasta.join("Assinatura de e-mail.html");
    std::fs::write(&p, html)?;
    Ok(p)
}

const SCRIPT_DOCK: &str = r#"for (const p of panels()) { for (const w of p.widgets("org.kde.plasma.icontasks")) { w.currentConfigGroup = ["General"]; const l = String(w.readConfig("launchers", "")); const n = l.split(",").filter((x) => x && x.indexOf("novasce-comecar.desktop") < 0).join(","); if (n !== l) w.writeConfig("launchers", n); } }"#;

pub fn args_tirar_do_dock() -> Vec<String> {
    [
        "busctl",
        "--user",
        "call",
        "org.kde.plasmashell",
        "/PlasmaShell",
        "org.kde.PlasmaShell",
        "evaluateScript",
        "s",
        SCRIPT_DOCK,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

pub fn tirar_do_dock() -> Result<(), String> {
    let a = args_tirar_do_dock();
    Command::new(&a[0])
        .args(&a[1..])
        .output()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod testes {
    use super::*;
    use base64::Engine;

    fn cfg(id: &str, segredo: &str) -> SecaoGoogle {
        SecaoGoogle {
            client_id: id.into(),
            client_secret: segredo.into(),
            pasta: "Novasce OS/Documentos".into(),
        }
    }

    #[test]
    fn autorizar_com_e_sem_cliente_proprio() {
        assert_eq!(
            args_autorizar(&cfg("id", "s")),
            vec!["rclone", "authorize", "drive", "id", "s"]
        );
        assert_eq!(
            args_autorizar(&cfg("", "")),
            vec!["rclone", "authorize", "drive"]
        );
    }

    #[test]
    fn token_em_json_ou_em_base64() {
        let json = r#"{"access_token":"a","token_type":"Bearer","refresh_token":"r","expiry":"2026-10-23T15:00:00Z"}"#;
        let saida = format!(
            "Abra o navegador\nPaste the following into your remote machine --->\n{json}\n<---End paste\n"
        );
        assert_eq!(extrair_token(&saida).as_deref(), Some(json));
        let bloco = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(serde_json::json!({ "token": json }).to_string());
        let saida2 =
            format!("Paste the following into your remote machine --->\n{bloco}\n<---End paste\n");
        assert_eq!(extrair_token(&saida2).as_deref(), Some(json));
        assert_eq!(extrair_token("nada"), None);
    }

    #[test]
    fn criar_remoto_e_conferir() {
        let a = args_criar_remoto(&cfg("id", "s"), "{\"x\":1}");
        assert_eq!(&a[..5], &["rclone", "config", "create", "gdrive", "drive"]);
        assert!(
            a.contains(&"scope=drive".to_string())
                && a.contains(&"client_id=id".to_string())
                && a.contains(&"--non-interactive".to_string())
        );
        assert!(remoto_configurado("local:\ngdrive:\n"));
        assert!(!remoto_configurado("local:\n"));
    }

    #[test]
    fn script_do_dock_tira_so_o_comecar() {
        let a = args_tirar_do_dock();
        assert_eq!(
            &a[..7],
            &[
                "busctl",
                "--user",
                "call",
                "org.kde.plasmashell",
                "/PlasmaShell",
                "org.kde.PlasmaShell",
                "evaluateScript"
            ]
        );
        assert!(
            a[8].contains("novasce-comecar.desktop") && a[8].contains("org.kde.plasma.icontasks")
        );
    }

    #[test]
    fn assinatura_vai_para_documentos() {
        let d = tempfile::tempdir().unwrap();
        let p = salvar_assinatura(d.path(), "<p>Luan</p>").unwrap();
        assert_eq!(p, d.path().join("Novasce/Assinatura de e-mail.html"));
        assert_eq!(std::fs::read_to_string(p).unwrap(), "<p>Luan</p>");
    }
}
