//! Espaços (nós do brain dos brains) e o kit da frente.
use crate::brain::Git;
use novasce_casa::nucleo::Espaco;
use serde::Serialize;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EstadoEspaco {
    Trazer,
    Trazido,
    Fora,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EspacoVisao {
    pub slug: String,
    pub nome: String,
    pub descricao: String,
    pub estado: EstadoEspaco,
}

fn descricao(e: &Espaco) -> String {
    if !e.member {
        return "não é do seu setor no Tenancy".into();
    }
    match e.slug.as_str() {
        "studio" => "o institucional da Novasce · todo o time participa".into(),
        "food" => "a linha Food da Novasce · quem é do setor participa".into(),
        _ => "o espaço da sua área".into(),
    }
}

pub fn visoes(espacos: &[Espaco], raiz: &Path) -> Vec<EspacoVisao> {
    espacos
        .iter()
        .map(|e| EspacoVisao {
            slug: e.slug.clone(),
            nome: e.name.clone(),
            descricao: descricao(e),
            estado: match (
                e.member,
                raiz.join(&e.name).join(".git").exists() || raiz.join(&e.name).exists(),
            ) {
                (false, _) => EstadoEspaco::Fora,
                (true, true) => EstadoEspaco::Trazido,
                (true, false) => EstadoEspaco::Trazer,
            },
        })
        .collect()
}

fn url_do_repo(repo: &str) -> String {
    if repo.starts_with("https://") {
        if repo.ends_with(".git") {
            repo.into()
        } else {
            format!("{repo}.git")
        }
    } else {
        format!("https://github.com/{repo}.git")
    }
}

pub fn trazer(
    espacos: &[Espaco],
    slugs: &[String],
    raiz: &Path,
    git: &dyn Git,
) -> Vec<(String, Result<(), String>)> {
    espacos
        .iter()
        .filter(|e| e.member && slugs.contains(&e.slug))
        .map(|e| {
            let destino = raiz.join(&e.name);
            let r = if destino.exists() {
                Ok(())
            } else {
                git.clonar(&url_do_repo(&e.repo), &destino)
            };
            (e.slug.clone(), r)
        })
        .collect()
}

/// Baixa o zip do kit com o token da pessoa e extrai sem sobrescrever e sem sair da pasta.
pub fn baixar_kit(url: &str, token_pessoa: &str, destino: &Path) -> Result<usize, String> {
    let r = ureq::get(url)
        .set("Authorization", &format!("Bearer {token_pessoa}"))
        .timeout(std::time::Duration::from_secs(120))
        .call()
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    r.into_reader()
        .take(200 * 1024 * 1024)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let mut gravados = 0;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).map_err(|e| e.to_string())?;
        let Some(nome) = f.enclosed_name() else {
            continue;
        };
        let alvo = destino.join(nome);
        if f.is_dir() || alvo.exists() {
            continue;
        }
        if let Some(p) = alvo.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let mut saida = std::fs::File::create(&alvo).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, &mut saida).map_err(|e| e.to_string())?;
        gravados += 1;
    }
    Ok(gravados)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::brain::Git;
    use std::cell::RefCell;
    use std::io::Write;

    fn lista() -> Vec<Espaco> {
        serde_json::from_str(r#"[{"slug":"studio","name":"Studio","repo":"Novasce/studio","member":true},{"slug":"food","name":"Food","repo":"Novasce/food","member":false}]"#).unwrap()
    }

    #[derive(Default)]
    struct GitFalso(RefCell<Vec<String>>);
    impl Git for GitFalso {
        fn clonar(&self, url: &str, destino: &Path) -> Result<(), String> {
            self.0.borrow_mut().push(url.into());
            std::fs::create_dir_all(destino).map_err(|e| e.to_string())
        }
    }

    #[test]
    fn visoes_dizem_o_motivo() {
        let d = tempfile::tempdir().unwrap();
        let v = visoes(&lista(), d.path());
        assert_eq!(v[0].estado, EstadoEspaco::Trazer);
        assert_eq!(
            v[0].descricao,
            "o institucional da Novasce · todo o time participa"
        );
        assert_eq!(v[1].estado, EstadoEspaco::Fora);
        assert_eq!(v[1].descricao, "não é do seu setor no Tenancy");
    }

    #[test]
    fn traz_so_membro_e_nao_repete() {
        let d = tempfile::tempdir().unwrap();
        let git = GitFalso::default();
        let r = trazer(&lista(), &["studio".into(), "food".into()], d.path(), &git);
        assert_eq!(r.len(), 1);
        assert_eq!(git.0.borrow()[0], "https://github.com/Novasce/studio.git");
        trazer(&lista(), &["studio".into()], d.path(), &git);
        assert_eq!(git.0.borrow().len(), 1);
        assert_eq!(visoes(&lista(), d.path())[0].estado, EstadoEspaco::Trazido);
    }

    #[test]
    fn kit_nao_sobrescreve_e_nao_sai_da_pasta() {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let o = zip::write::SimpleFileOptions::default();
        zip.start_file("frente/leia.md", o).unwrap();
        zip.write_all(b"novo").unwrap();
        zip.start_file("ja.md", o).unwrap();
        zip.write_all(b"do kit").unwrap();
        zip.start_file("../fora.md", o).unwrap();
        zip.write_all(b"x").unwrap();
        let bytes = zip.finish().unwrap().into_inner();
        let mut s = mockito::Server::new();
        s.mock("GET", "/kit.zip")
            .match_header("authorization", "Bearer nvos_u_x")
            .with_body(bytes)
            .create();
        let d = tempfile::tempdir().unwrap();
        let destino = d.path().join("Brain");
        std::fs::create_dir_all(&destino).unwrap();
        std::fs::write(destino.join("ja.md"), "meu").unwrap();
        let n = baixar_kit(&format!("{}/kit.zip", s.url()), "nvos_u_x", &destino).unwrap();
        assert_eq!(n, 1);
        assert_eq!(
            std::fs::read_to_string(destino.join("ja.md")).unwrap(),
            "meu"
        );
        assert_eq!(
            std::fs::read_to_string(destino.join("frente/leia.md")).unwrap(),
            "novo"
        );
        assert!(!d.path().join("fora.md").exists());
    }
}
