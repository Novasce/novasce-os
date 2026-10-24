//! O Meu dia abre sozinho na primeira entrada do dia. O Novasce OS só abre a tela; o Meu dia é do Tenancy.
use chrono::{NaiveDate, NaiveDateTime, Timelike};
use serde::{Deserialize, Serialize};

pub const AVISO_FORA: &str = "Ajuda · o Tenancy não respondeu. O Meu dia abre quando ele voltar.";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EstadoManha {
    pub ultimo_dia: Option<String>,
}

pub fn dia_de_trabalho(agora: NaiveDateTime) -> NaiveDate {
    if agora.hour() < 5 {
        agora.date().pred_opt().unwrap_or(agora.date())
    } else {
        agora.date()
    }
}

pub fn deve_abrir(ligada: bool, conta_sede: bool, e: &EstadoManha, hoje: NaiveDate) -> bool {
    ligada && !conta_sede && e.ultimo_dia.as_deref() != Some(hoje.to_string().as_str())
}

pub fn tenancy_responde(web: &str) -> bool {
    let url = novasce_casa::abrir::juntar(web, "/meu-dia");
    match ureq::get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .call()
    {
        Ok(_) => true,
        Err(ureq::Error::Status(codigo, _)) => codigo < 500,
        Err(_) => false,
    }
}

pub fn caminho(c: &novasce_casa::caminhos::Caminhos) -> std::path::PathBuf {
    c.estado_novasce().join("manha.json")
}

#[cfg(test)]
mod testes {
    use super::*;
    use chrono::NaiveDate;

    fn d(s: &str) -> chrono::NaiveDateTime {
        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap()
    }

    #[test]
    fn o_dia_vira_as_cinco() {
        assert_eq!(
            dia_de_trabalho(d("2026-10-23 04:59")),
            NaiveDate::from_ymd_opt(2026, 10, 22).unwrap()
        );
        assert_eq!(
            dia_de_trabalho(d("2026-10-23 08:41")),
            NaiveDate::from_ymd_opt(2026, 10, 23).unwrap()
        );
    }

    #[test]
    fn abre_uma_vez_por_dia_so_para_a_pessoa() {
        let hoje = NaiveDate::from_ymd_opt(2026, 10, 23).unwrap();
        let novo = EstadoManha::default();
        assert!(deve_abrir(true, false, &novo, hoje));
        assert!(!deve_abrir(false, false, &novo, hoje));
        assert!(!deve_abrir(true, true, &novo, hoje));
        let ja = EstadoManha {
            ultimo_dia: Some("2026-10-23".into()),
        };
        assert!(!deve_abrir(true, false, &ja, hoje));
    }

    #[test]
    fn aviso_discreto_quando_o_tenancy_nao_responde() {
        assert_eq!(
            AVISO_FORA,
            "Ajuda · o Tenancy não respondeu. O Meu dia abre quando ele voltar."
        );
        assert!(!tenancy_responde("http://127.0.0.1:9"));
    }
}
