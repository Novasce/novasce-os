//! Notas de versão públicas (`notas/versoes.json`), lidas pelo aviso de atualização do M3.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TipoDeNota {
    Importante,
    Opcional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotaDeVersao {
    pub versao: String,
    pub tipo: TipoDeNota,
    pub linha: String,
    pub custo: String,
}

pub fn ler_notas(json: &str) -> Result<Vec<NotaDeVersao>, String> {
    let notas: Vec<NotaDeVersao> = serde_json::from_str(json)
        .map_err(|e| format!("notas/versoes.json fora do formato: {e}"))?;
    for nota in &notas {
        if !versao_valida(&nota.versao) {
            return Err(format!(
                "a versão {} está fora do formato AAAA.MM.DD.N",
                nota.versao
            ));
        }
        for (campo, texto) in [("linha", &nota.linha), ("custo", &nota.custo)] {
            if texto.trim().is_empty() || texto.chars().count() > 140 {
                return Err(format!(
                    "{campo} da versão {} precisa ter de 1 a 140 caracteres",
                    nota.versao
                ));
            }
            if texto.contains('\u{2014}') {
                return Err(format!("{campo} da versão {} tem travessão", nota.versao));
            }
        }
    }
    Ok(notas)
}

/// AAAA.MM.DD.N, com N de um ou mais dígitos.
pub fn versao_valida(versao: &str) -> bool {
    let partes: Vec<&str> = versao.split('.').collect();
    partes.len() == 4
        && partes[0].len() == 4
        && partes[1].len() == 2
        && partes[2].len() == 2
        && !partes[3].is_empty()
        && partes.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_arquivo_do_repositorio_vale() {
        let json = include_str!("../../../../notas/versoes.json");
        assert!(ler_notas(json).is_ok());
    }

    #[test]
    fn le_uma_nota_valida() {
        let notas = ler_notas(
            r#"[{"versao":"2026.10.24.1","tipo":"opcional","linha":"Acha a impressora da sede sozinho.","custo":"Reiniciar leva uns 2 minutos."}]"#,
        )
        .unwrap();
        assert_eq!(notas[0].tipo, TipoDeNota::Opcional);
    }

    #[test]
    fn recusa_travessao() {
        let erro = ler_notas(
            "[{\"versao\":\"2026.10.24.1\",\"tipo\":\"importante\",\"linha\":\"Corrige o login \u{2014} urgente\",\"custo\":\"2 minutos\"}]",
        )
        .unwrap_err();
        assert!(erro.contains("travessão"));
    }

    #[test]
    fn recusa_versao_fora_do_formato() {
        assert!(
            ler_notas(r#"[{"versao":"1.0.4","tipo":"opcional","linha":"x","custo":"y"}]"#).is_err()
        );
    }

    #[test]
    fn recusa_tipo_desconhecido() {
        assert!(ler_notas(
            r#"[{"versao":"2026.10.24.1","tipo":"urgente","linha":"x","custo":"y"}]"#
        )
        .is_err());
    }

    #[test]
    fn formato_da_versao() {
        assert!(versao_valida("2026.10.24.1"));
        assert!(versao_valida("2026.10.24.12"));
        assert!(!versao_valida("2026.10.24"));
        assert!(!versao_valida("26.10.24.1"));
        assert!(!versao_valida("2026.10.24.x"));
    }
}
