//! Estado em JSON que nunca fica pela metade: grava num .tmp e troca de nome.
use serde::{de::DeserializeOwned, Serialize};
use std::{fs, io, path::Path};

pub fn ler<T: DeserializeOwned + Default>(caminho: &Path) -> T {
    fs::read(caminho)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

pub fn gravar<T: Serialize>(caminho: &Path, valor: &T) -> io::Result<()> {
    if let Some(pai) = caminho.parent() {
        fs::create_dir_all(pai)?;
    }
    let tmp = caminho.with_extension("tmp");
    fs::write(
        &tmp,
        serde_json::to_vec_pretty(valor).map_err(io::Error::other)?,
    )?;
    fs::rename(&tmp, caminho)
}

#[cfg(test)]
mod testes {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
    struct Coisa {
        n: u32,
    }

    #[test]
    fn grava_e_le_criando_pastas() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a/b/coisa.json");
        gravar(&p, &Coisa { n: 7 }).unwrap();
        assert_eq!(ler::<Coisa>(&p), Coisa { n: 7 });
        assert!(!p.with_extension("tmp").exists());
    }

    #[test]
    fn arquivo_ausente_ou_quebrado_vira_padrao() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(ler::<Coisa>(&d.path().join("nada.json")), Coisa::default());
        std::fs::write(d.path().join("ruim.json"), "{").unwrap();
        assert_eq!(ler::<Coisa>(&d.path().join("ruim.json")), Coisa::default());
    }
}
