//! Executor D-Bus do KRunner: notas do brain pelo título e artigos da ajuda.
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ItemIndice {
    pub tipo: String,
    pub id: String,
    pub titulo: String,
    pub trecho: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Alvo {
    Nota(PathBuf),
    Ajuda(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Achado {
    pub id: String,
    pub texto: String,
    pub sub: String,
    pub relevancia: f64,
}

pub fn normalizar(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'ê' | 'è' => 'e',
            'í' | 'î' => 'i',
            'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ü' => 'u',
            'ç' => 'c',
            c => c,
        })
        .collect()
}

fn titulo(arquivo: &Path) -> String {
    let texto = std::fs::read_to_string(arquivo).unwrap_or_default();
    texto
        .lines()
        .take(30)
        .find_map(|l| l.strip_prefix("# ").map(|t| t.trim().to_string()))
        .unwrap_or_else(|| {
            arquivo
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        })
}

/// (caminho relativo ao brain, título). Pastas que começam com ponto ficam de fora.
pub fn notas(brain: &Path, limite: usize) -> Vec<(PathBuf, String)> {
    let mut saida = Vec::new();
    let mut pilha = vec![brain.to_path_buf()];
    while let Some(dir) = pilha.pop() {
        let Ok(it) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in it.flatten() {
            let p = e.path();
            if e.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            if p.is_dir() {
                pilha.push(p);
            } else if p.extension().is_some_and(|x| x == "md") {
                let rel = p.strip_prefix(brain).unwrap_or(&p).to_path_buf();
                saida.push((rel, titulo(&p)));
                if saida.len() >= limite {
                    return saida;
                }
            }
        }
    }
    saida
}

pub fn procurar(
    consulta: &str,
    notas: &[(PathBuf, String)],
    _brain: &Path,
    ajuda: &[ItemIndice],
    limite: usize,
) -> Vec<Achado> {
    let n = normalizar(consulta);
    let palavras: Vec<&str> = n.split_whitespace().collect();
    if n.trim().chars().count() < 3 {
        return vec![];
    }
    let casa = |texto: &str| {
        let t = normalizar(texto);
        palavras.iter().all(|p| t.contains(p))
    };
    let mut r: Vec<Achado> = notas
        .iter()
        .filter(|(rel, t)| casa(&format!("{t} {}", rel.to_string_lossy())))
        .map(|(rel, t)| Achado {
            id: format!("nota:{}", rel.to_string_lossy()),
            texto: t.clone(),
            sub: format!("Nota do brain · {}", rel.to_string_lossy()),
            relevancia: 0.8,
        })
        .collect();
    r.extend(
        ajuda
            .iter()
            .filter(|i| casa(&format!("{} {}", i.titulo, i.trecho)))
            .map(|i| Achado {
                id: format!("ajuda:{}:{}", i.tipo, i.id),
                texto: i.titulo.clone(),
                sub: "Ajuda da Novasce".into(),
                relevancia: 0.6,
            }),
    );
    r.truncate(limite);
    r
}

fn codificar(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

pub fn url_obsidian(rel: &Path) -> String {
    let sem_ext = rel.with_extension("");
    format!(
        "obsidian://open?vault=Brain&file={}",
        codificar(&sem_ext.to_string_lossy())
    )
}

pub fn alvo_de_id(id: &str) -> Option<Alvo> {
    if let Some(n) = id.strip_prefix("nota:") {
        Some(Alvo::Nota(n.into()))
    } else {
        id.strip_prefix("ajuda:").map(|a| Alvo::Ajuda(a.into()))
    }
}

pub fn ler_indice(caminho: &str) -> Vec<ItemIndice> {
    std::fs::read_to_string(caminho)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod testes {
    use super::*;

    fn brain() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("reunioes")).unwrap();
        std::fs::create_dir_all(d.path().join(".git")).unwrap();
        std::fs::write(
            d.path().join("reunioes/2026-10-23.md"),
            "---\ntags: x\n---\n# Reunião com a Yakimassa\n\nInsumos.",
        )
        .unwrap();
        std::fs::write(d.path().join("ideias.md"), "sem título").unwrap();
        std::fs::write(d.path().join(".git/config.md"), "# não entra").unwrap();
        d
    }

    #[test]
    fn titulo_vem_do_cabecalho_e_git_fica_de_fora() {
        let d = brain();
        let n = notas(d.path(), 100);
        assert_eq!(n.len(), 2);
        assert!(n.iter().any(|(_, t)| t == "Reunião com a Yakimassa"));
        assert!(n.iter().any(|(_, t)| t == "ideias"));
    }

    #[test]
    fn acha_sem_acento_e_mistura_ajuda() {
        let d = brain();
        let n = notas(d.path(), 100);
        let ajuda = vec![ItemIndice {
            tipo: "guia".into(),
            id: "brain-e-espacos".into(),
            titulo: "Brain e espaços".into(),
            trecho: "O brain é seu.".into(),
        }];
        let r = procurar("reuniao yakimassa", &n, d.path(), &ajuda, 8);
        assert_eq!(r[0].texto, "Reunião com a Yakimassa");
        assert_eq!(r[0].sub, "Nota do brain · reunioes/2026-10-23.md");
        let r = procurar("espacos", &n, d.path(), &ajuda, 8);
        assert_eq!(r[0].id, "ajuda:guia:brain-e-espacos");
        assert!(procurar("ab", &n, d.path(), &ajuda, 8).is_empty());
    }

    #[test]
    fn abre_no_obsidian() {
        assert_eq!(
            url_obsidian(Path::new("reunioes/Reunião com a Yakimassa.md")),
            "obsidian://open?vault=Brain&file=reunioes%2FReuni%C3%A3o%20com%20a%20Yakimassa"
        );
        assert_eq!(
            alvo_de_id("nota:reunioes/a.md"),
            Some(Alvo::Nota("reunioes/a.md".into()))
        );
        assert_eq!(
            alvo_de_id("ajuda:ficha:drive"),
            Some(Alvo::Ajuda("ficha:drive".into()))
        );
        assert_eq!(alvo_de_id("x"), None);
    }
}
