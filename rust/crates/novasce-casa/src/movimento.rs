//! Menos movimento: segue a velocidade de animação do KDE (0 = sem animação).
use crate::caminhos::Caminhos;

pub fn fator_de_kdeglobals(texto: &str) -> f32 {
    let mut no_kde = false;
    for linha in texto.lines() {
        let l = linha.trim();
        if l.starts_with('[') {
            no_kde = l == "[KDE]";
            continue;
        }
        if no_kde {
            if let Some(v) = l.strip_prefix("AnimationDurationFactor=") {
                return v
                    .trim()
                    .parse::<f32>()
                    .map(|f| f.clamp(0.0, 4.0))
                    .unwrap_or(1.0);
            }
        }
    }
    1.0
}

pub fn fator_atual(c: &Caminhos) -> f32 {
    fator_de_kdeglobals(&std::fs::read_to_string(c.config.join("kdeglobals")).unwrap_or_default())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn le_o_fator_do_grupo_kde() {
        assert_eq!(
            fator_de_kdeglobals(
                "[General]\nAnimationDurationFactor=3\n[KDE]\nAnimationDurationFactor=0\n"
            ),
            0.0
        );
        assert_eq!(
            fator_de_kdeglobals("[KDE]\nLookAndFeelPackage=x\nAnimationDurationFactor=0.5\n"),
            0.5
        );
        assert_eq!(fator_de_kdeglobals("[KDE]\n"), 1.0);
        assert_eq!(
            fator_de_kdeglobals("[KDE]\nAnimationDurationFactor=lixo\n"),
            1.0
        );
        assert_eq!(
            fator_de_kdeglobals("[KDE]\nAnimationDurationFactor=9\n"),
            4.0
        );
    }
}
