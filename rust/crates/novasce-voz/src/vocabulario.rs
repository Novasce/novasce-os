//! Vocabulário da casa: um "prompt inicial" para o whisper e trocas depois da transcrição.
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Vocabulario {
    #[serde(default)]
    pub prompt: String,
    #[serde(default, rename = "troca")]
    pub trocas: Vec<Troca>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Troca {
    pub de: String,
    pub para: String,
}

pub fn ler(texto: &str) -> Result<Vocabulario, String> {
    toml::from_str(texto).map_err(|e| format!("vocabulário ilegível: {e}"))
}

pub fn juntar(casa: Vocabulario, pessoa: Vocabulario) -> Vocabulario {
    let prompt = [casa.prompt.trim(), pessoa.prompt.trim()]
        .iter()
        .filter(|s| !s.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    Vocabulario {
        prompt,
        trocas: casa.trocas.into_iter().chain(pessoa.trocas).collect(),
    }
}

pub fn aplicar(v: &Vocabulario, texto: &str) -> String {
    let mut s = texto.split_whitespace().collect::<Vec<_>>().join(" ");
    for t in &v.trocas {
        let padrao =
            t.de.split_whitespace()
                .map(regex::escape)
                .collect::<Vec<_>>()
                .join(r"\s+");
        if let Ok(re) = regex::Regex::new(&format!(r"(?i)\b{padrao}\b")) {
            s = re.replace_all(&s, t.para.as_str()).into_owned();
        }
    }
    s.trim().to_string()
}

pub fn carregar(arquivo_casa: &str, arquivo_pessoa: &std::path::Path) -> Vocabulario {
    let casa = std::fs::read_to_string(arquivo_casa)
        .ok()
        .and_then(|t| ler(&t).ok())
        .unwrap_or_default();
    let pessoa = std::fs::read_to_string(arquivo_pessoa)
        .ok()
        .and_then(|t| ler(&t).ok())
        .unwrap_or_default();
    juntar(casa, pessoa)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn troca_palavra_inteira_sem_maiuscula() {
        let v =
            ler("prompt = \"Novasce\"\n[[troca]]\nde = \"so pec\"\npara = \"SOPEC\"\n").unwrap();
        assert_eq!(aplicar(&v, "  abri o So Pec   hoje "), "abri o SOPEC hoje");
        assert_eq!(aplicar(&v, "sopecado"), "sopecado");
    }

    #[test]
    fn pessoa_soma_ao_da_casa() {
        let casa = ler("prompt = \"Novasce, Tenancy.\"").unwrap();
        let pessoa =
            ler("prompt = \"Mizoo.\"\n[[troca]]\nde = \"mi zoo\"\npara = \"Mizoo\"\n").unwrap();
        let j = juntar(casa, pessoa);
        assert_eq!(j.prompt, "Novasce, Tenancy. Mizoo.");
        assert_eq!(j.trocas.len(), 1);
    }

    #[test]
    fn o_arquivo_da_imagem_e_valido() {
        let v = ler(include_str!(
            "../../../../system_files/usr/share/novasce/voz/vocabulario.toml"
        ))
        .unwrap();
        assert!(v.prompt.contains("Tenancy"));
        assert_eq!(aplicar(&v, "abre o tenânsi"), "abre o Tenancy");
    }
}
