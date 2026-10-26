//! Trazer a Sala para a frente, em tela cheia, com um script do KWin (Plasma 6).

pub fn script_trazer_sala(titulo: &str) -> String {
    let t = serde_json::to_string(titulo).unwrap_or_else(|_| "\"\"".into());
    format!("for (const w of workspace.windowList()) {{ if (String(w.caption).indexOf({t}) !== -1) {{ w.minimized = false; workspace.activeWindow = w; w.fullScreen = true; break; }} }}")
}

#[cfg(target_os = "linux")]
pub fn rodar_script(pasta: &std::path::Path, nome: &str, script: &str) -> Result<(), String> {
    let arq = pasta.join(format!("{nome}.js"));
    std::fs::create_dir_all(pasta).map_err(|e| e.to_string())?;
    std::fs::write(&arq, script).map_err(|e| e.to_string())?;
    let con = zbus::blocking::Connection::session().map_err(|e| e.to_string())?;
    let kwin =
        zbus::blocking::Proxy::new(&con, "org.kde.KWin", "/Scripting", "org.kde.kwin.Scripting")
            .map_err(|e| e.to_string())?;
    let _: bool = kwin.call("unloadScript", &(nome,)).unwrap_or(false);
    let id: i32 = kwin
        .call("loadScript", &(arq.to_string_lossy().to_string(), nome))
        .map_err(|e| e.to_string())?;
    let s = zbus::blocking::Proxy::new(
        &con,
        "org.kde.KWin",
        format!("/Scripting/Script{id}"),
        "org.kde.kwin.Script",
    )
    .map_err(|e| e.to_string())?;
    s.call::<_, _, ()>("run", &()).map_err(|e| e.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    let _: bool = kwin.call("unloadScript", &(nome,)).unwrap_or(false);
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn script_traz_a_sala_pelo_titulo_com_aspas_seguras() {
        let s = script_trazer_sala("Sala da \"Novasce\"");
        assert!(s.contains(r#"indexOf("Sala da \"Novasce\"")"#));
        assert!(s.contains("workspace.windowList()") && s.contains("fullScreen = true"));
    }
}
