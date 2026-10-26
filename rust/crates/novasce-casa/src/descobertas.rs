//! Descobertas do fim do Começar: progresso sem placar, sem pontos e sem sequência de dias.
use crate::{caminhos::Caminhos, json_atomico};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Descobertas {
    pub falou: bool,
    pub pediu_ajuda: bool,
    pub achou_nota: bool,
    pub trocou_atalho: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Descoberta {
    Falou,
    PediuAjuda,
    AchouNota,
    TrocouAtalho,
}

fn caminho(c: &Caminhos) -> std::path::PathBuf {
    c.estado_novasce().join("descobertas.json")
}

pub fn ler(c: &Caminhos) -> Descobertas {
    json_atomico::ler(&caminho(c))
}

pub fn marcar(c: &Caminhos, d: Descoberta) -> std::io::Result<()> {
    let mut x = ler(c);
    match d {
        Descoberta::Falou => x.falou = true,
        Descoberta::PediuAjuda => x.pediu_ajuda = true,
        Descoberta::AchouNota => x.achou_nota = true,
        Descoberta::TrocouAtalho => x.trocou_atalho = true,
    }
    json_atomico::gravar(&caminho(c), &x)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn marca_uma_de_cada_vez() {
        let d = tempfile::tempdir().unwrap();
        let c = Caminhos::em(d.path());
        assert_eq!(ler(&c), Descobertas::default());
        marcar(&c, Descoberta::Falou).unwrap();
        marcar(&c, Descoberta::AchouNota).unwrap();
        let x = ler(&c);
        assert!(x.falou && x.achou_nota && !x.pediu_ajuda && !x.trocou_atalho);
    }
}
