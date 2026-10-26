//! "Nesta tela": qual app estava na frente quando a pessoa clicou no "?".

/// Script do KWin (Plasma 6): devolve a classe da janela ativa para a barra por D-Bus.
pub const SCRIPT: &str = r#"const w = workspace.activeWindow; callDBus("org.novasce.Barra", "/janela", "org.novasce.Janela", "Ativa", w ? String(w.resourceClass) : "");"#;

pub fn tela_da_classe(classe: &str, conta_sede: bool) -> &'static str {
    if conta_sede {
        return "sala";
    }
    let c = classe.to_lowercase();
    if c.contains("novasce.comecar") || c.contains("novasce-comecar") {
        "comecar"
    } else if c.contains("dolphin") {
        "arquivos"
    } else if c.contains("obsidian") {
        "brain"
    } else if c.contains("chrome") || c.contains("chromium") || c.contains("firefox") {
        "navegador"
    } else if c.contains("discover") {
        "loja"
    } else {
        "geral"
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn app_ativo_vira_tela_da_ajuda() {
        assert_eq!(tela_da_classe("org.novasce.comecar", false), "comecar");
        assert_eq!(tela_da_classe("org.kde.dolphin", false), "arquivos");
        assert_eq!(tela_da_classe("md.obsidian.Obsidian", false), "brain");
        assert_eq!(tela_da_classe("google-chrome", false), "navegador");
        assert_eq!(tela_da_classe("org.kde.discover", false), "loja");
        assert_eq!(tela_da_classe("", false), "geral");
        assert_eq!(tela_da_classe("google-chrome", true), "sala");
    }

    #[test]
    fn script_devolve_por_dbus() {
        assert!(
            SCRIPT.contains("workspace.activeWindow")
                && SCRIPT
                    .contains(r#""org.novasce.Barra", "/janela", "org.novasce.Janela", "Ativa""#)
        );
    }
}
