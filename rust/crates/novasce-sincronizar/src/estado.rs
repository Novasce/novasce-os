//! O que a nuvem diz na barra e no Começar.
use crate::brain::{ErroBrain, ResumoBrain};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Situacao {
    Salvo,
    Subindo,
    Problema,
    #[default]
    Desligado,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub estado: Situacao,
    pub quando: Option<String>,
    pub progresso: Option<u8>,
    pub mensagem: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EstadoSync {
    pub brain: Item,
    pub documentos: Item,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cor {
    Verde,
    Laranja,
    Neutra,
}

fn problema(m: impl Into<String>) -> Item {
    Item {
        estado: Situacao::Problema,
        quando: None,
        progresso: None,
        mensagem: Some(m.into()),
    }
}

pub fn do_resumo_brain(r: &ResumoBrain, agora: &str) -> Item {
    match &r.erro {
        Some(ErroBrain::SemBrain) => {
            return Item {
                mensagem: Some("O brain ainda não foi criado. Abra o Começar.".into()),
                ..Item::default()
            }
        }
        Some(ErroBrain::SemRede) => return problema(
            "Sem internet. As notas estão salvas neste computador e sobem quando a conexão voltar.",
        ),
        Some(ErroBrain::Varredura(_)) => {
            return problema("A varredura de segredo não rodou. Nada foi enviado.")
        }
        Some(ErroBrain::Git(m)) => {
            return problema(format!(
                "O brain não subiu. Nada foi apagado. {}",
                m.lines().next().unwrap_or("")
            ))
        }
        None => {}
    }
    match r.retidos.as_slice() {
        [] => {}
        [a] => {
            return problema(format!(
                "{} parece ter uma senha e ficou só neste computador.",
                a.arquivo
            ))
        }
        [a, resto @ ..] => {
            return problema(format!(
                "{} e mais {} arquivos parecem ter senha e ficaram só neste computador.",
                a.arquivo,
                resto.len()
            ))
        }
    }
    let mensagem = r.copias.first().map(|c| {
        format!("Uma nota mudou em dois lugares. Nada foi apagado: a sua versão ficou em {c}.")
    });
    Item {
        estado: Situacao::Salvo,
        quando: Some(agora.into()),
        progresso: None,
        mensagem,
    }
}

pub fn cor(e: &EstadoSync) -> Cor {
    let itens = [&e.brain, &e.documentos];
    if itens.iter().any(|i| i.estado == Situacao::Problema) {
        Cor::Laranja
    } else if itens.iter().all(|i| i.estado == Situacao::Desligado) {
        Cor::Neutra
    } else {
        Cor::Verde
    }
}

pub fn ha_quanto(quando: &str, agora: chrono::DateTime<chrono::Utc>) -> String {
    let Ok(q) = quando.parse::<chrono::DateTime<chrono::Utc>>() else {
        return String::new();
    };
    let min = (agora - q).num_minutes().max(0);
    match min {
        0 => "agora".into(),
        1..=59 => format!("há {min} min"),
        _ => format!("há {} h", min / 60),
    }
}

pub fn dica(e: &EstadoSync, agora: chrono::DateTime<chrono::Utc>) -> String {
    if let Some(m) = [&e.brain, &e.documentos]
        .iter()
        .find(|i| i.estado == Situacao::Problema)
        .and_then(|i| i.mensagem.clone())
    {
        return format!("Nuvem · {m}");
    }
    if e.documentos.estado == Situacao::Subindo {
        return match e.documentos.progresso {
            Some(p) => format!("Nuvem · subindo os Documentos ({p}%)"),
            None => "Nuvem · subindo os Documentos".into(),
        };
    }
    let salvo = |i: &Item| {
        (i.estado == Situacao::Salvo)
            .then(|| i.quando.clone())
            .flatten()
    };
    match (salvo(&e.documentos), salvo(&e.brain)) {
        (Some(d), Some(b)) => format!(
            "Nuvem · Documentos e brain salvos {}",
            ha_quanto(d.max(b).as_str(), agora)
        ),
        (Some(d), None) => format!("Nuvem · Documentos salvos {}", ha_quanto(&d, agora)),
        (None, Some(b)) => format!("Nuvem · brain salvo {}", ha_quanto(&b, agora)),
        (None, None) => "Nuvem · ainda não conectada. Abra o Começar para ligar.".into(),
    }
}

pub fn caminho(c: &novasce_casa::caminhos::Caminhos) -> std::path::PathBuf {
    c.estado_novasce().join("sincronizar.json")
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::brain::{Achado, ErroBrain, ResumoBrain};

    fn agora() -> chrono::DateTime<chrono::Utc> {
        "2026-10-23T14:21:00Z".parse().unwrap()
    }

    #[test]
    fn resumo_do_brain_vira_item() {
        let ok = do_resumo_brain(
            &ResumoBrain {
                enviou: true,
                ..Default::default()
            },
            "2026-10-23T14:20:00Z",
        );
        assert_eq!(ok.estado, Situacao::Salvo);
        let senha = do_resumo_brain(
            &ResumoBrain {
                retidos: vec![Achado {
                    arquivo: "notas/x.md".into(),
                    regra: "r".into(),
                }],
                ..Default::default()
            },
            "t",
        );
        assert_eq!(senha.estado, Situacao::Problema);
        assert_eq!(
            senha.mensagem.as_deref(),
            Some("notas/x.md parece ter uma senha e ficou só neste computador.")
        );
        let rede = do_resumo_brain(
            &ResumoBrain {
                erro: Some(ErroBrain::SemRede),
                ..Default::default()
            },
            "t",
        );
        assert_eq!(rede.mensagem.as_deref(), Some("Sem internet. As notas estão salvas neste computador e sobem quando a conexão voltar."));
    }

    #[test]
    fn cor_e_dica_da_barra() {
        let salvo = Item {
            estado: Situacao::Salvo,
            quando: Some("2026-10-23T14:20:00Z".into()),
            progresso: None,
            mensagem: None,
        };
        let e = EstadoSync {
            brain: salvo.clone(),
            documentos: salvo.clone(),
        };
        assert_eq!(cor(&e), Cor::Verde);
        assert_eq!(
            dica(&e, agora()),
            "Nuvem · Documentos e brain salvos há 1 min"
        );
        let p = EstadoSync {
            brain: salvo.clone(),
            documentos: Item {
                estado: Situacao::Problema,
                quando: None,
                progresso: None,
                mensagem: Some("Sem internet.".into()),
            },
        };
        assert_eq!(cor(&p), Cor::Laranja);
        assert_eq!(dica(&p, agora()), "Nuvem · Sem internet.");
        assert_eq!(cor(&EstadoSync::default()), Cor::Neutra);
        assert_eq!(
            dica(&EstadoSync::default(), agora()),
            "Nuvem · ainda não conectada. Abra o Começar para ligar."
        );
        let subindo = EstadoSync {
            brain: salvo,
            documentos: Item {
                estado: Situacao::Subindo,
                quando: None,
                progresso: Some(62),
                mensagem: None,
            },
        };
        assert_eq!(
            dica(&subindo, agora()),
            "Nuvem · subindo os Documentos (62%)"
        );
    }
}
