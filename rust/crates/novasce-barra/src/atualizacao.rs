//! Aviso de atualização: aparece uma vez; depois fica o sinal "x.y.z pronta". Nada reinicia à força.
use novasce_casa::sistema::{versao_pronta, StatusOstree};
pub use novasce_casa::versoes::NotasVersao;
use novasce_casa::versoes::{aviso, Aviso};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EstadoAviso {
    pub avisada: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum Passo {
    Nada,
    Esconder,
    Avisar(Aviso, String),
    Sinal { titulo: String },
}

pub fn decidir(
    status: &StatusOstree,
    notas: &NotasVersao,
    e: &EstadoAviso,
    conta_sede: bool,
) -> Passo {
    if conta_sede {
        return Passo::Nada;
    }
    let Some(imagem) = versao_pronta(status) else {
        return Passo::Esconder;
    };
    let a = aviso(notas, &imagem);
    if e.avisada.as_deref() == Some(imagem.as_str()) {
        Passo::Sinal {
            titulo: format!("{} pronta", a.versao),
        }
    } else {
        Passo::Avisar(a, imagem)
    }
}

pub fn args_reiniciar() -> Vec<&'static str> {
    vec![
        "busctl",
        "--user",
        "call",
        "org.kde.Shutdown",
        "/Shutdown",
        "org.kde.Shutdown",
        "logoutAndReboot",
    ]
}

pub fn caminho(c: &novasce_casa::caminhos::Caminhos) -> std::path::PathBuf {
    c.estado_novasce().join("atualizacao.json")
}

#[cfg(test)]
mod testes {
    use super::*;
    use novasce_casa::sistema::ler_status_ostree;
    use novasce_casa::versoes::ler_notas;

    fn notas() -> NotasVersao {
        ler_notas(r#"[{"versao":"2026.10.30.1","tipo":"opcional","linha":"Deixa o Começar mais curto e acha a impressora da sede sozinho.","custo":"Reiniciar leva uns 2 minutos, e nada fecha sem você mandar."}]"#).unwrap()
    }
    fn com_pronta() -> StatusOstree {
        ler_status_ostree(r#"{"deployments":[{"staged":true,"version":"2026.10.30.1","timestamp":2},{"booted":true,"version":"2026.10.23.1","timestamp":1}]}"#).unwrap()
    }

    #[test]
    fn sem_versao_pronta_esconde() {
        let s = ler_status_ostree(
            r#"{"deployments":[{"booted":true,"version":"2026.10.23.1","timestamp":1}]}"#,
        )
        .unwrap();
        assert_eq!(
            decidir(&s, &notas(), &EstadoAviso::default(), false),
            Passo::Esconder
        );
    }

    #[test]
    fn avisa_uma_vez_e_depois_fica_o_sinal() {
        match decidir(&com_pronta(), &notas(), &EstadoAviso::default(), false) {
            Passo::Avisar(a, imagem) => {
                assert_eq!(a.titulo, "Novasce OS 2026.10.30.1 · opcional");
                assert_eq!(imagem, "2026.10.30.1");
            }
            outro => panic!("{outro:?}"),
        }
        let visto = EstadoAviso {
            avisada: Some("2026.10.30.1".into()),
        };
        assert_eq!(
            decidir(&com_pronta(), &notas(), &visto, false),
            Passo::Sinal {
                titulo: "2026.10.30.1 pronta".into()
            }
        );
    }

    #[test]
    fn na_conta_sede_quem_avisa_e_a_sala() {
        assert_eq!(
            decidir(&com_pronta(), &notas(), &EstadoAviso::default(), true),
            Passo::Nada
        );
    }

    #[test]
    fn reiniciar_pelo_plasma() {
        assert_eq!(
            args_reiniciar(),
            vec![
                "busctl",
                "--user",
                "call",
                "org.kde.Shutdown",
                "/Shutdown",
                "org.kde.Shutdown",
                "logoutAndReboot"
            ]
        );
    }
}
