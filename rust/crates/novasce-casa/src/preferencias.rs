//! Escolhas da pessoa que valem para vários apps.
use crate::{caminhos::Caminhos, json_atomico};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferencias {
    /// "Abrir o Meu dia ao chegar de manhã". Vem marcado.
    pub manha: bool,
}

impl Default for Preferencias {
    fn default() -> Self {
        Preferencias { manha: true }
    }
}

fn caminho(c: &Caminhos) -> std::path::PathBuf {
    c.config_novasce().join("preferencias.json")
}

pub fn ler(c: &Caminhos) -> Preferencias {
    json_atomico::ler(&caminho(c))
}

pub fn gravar(c: &Caminhos, p: &Preferencias) -> std::io::Result<()> {
    json_atomico::gravar(&caminho(c), p)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn manha_vem_marcada_e_grava() {
        let d = tempfile::tempdir().unwrap();
        let c = Caminhos::em(d.path());
        assert!(ler(&c).manha);
        gravar(&c, &Preferencias { manha: false }).unwrap();
        assert!(!ler(&c).manha);
    }
}
