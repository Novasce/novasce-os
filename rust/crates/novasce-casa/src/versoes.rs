//! Notas de versão públicas (notas/versoes.json do M1): aviso de atualização e Novidades da Central.
pub use novasce_core::notas::{ler_notas, NotaDeVersao, TipoDeNota};

pub type NotasVersao = Vec<NotaDeVersao>;

#[derive(Debug, Clone, PartialEq)]
pub struct Aviso {
    pub versao: String,
    pub titulo: String,
    pub corpo: String,
    pub importante: bool,
}

pub const CUSTO_PADRAO: &str = "Reiniciar leva uns 2 minutos, e nada fecha sem você mandar.";

pub fn ler_local(caminho: &str) -> NotasVersao {
    std::fs::read_to_string(caminho)
        .ok()
        .and_then(|t| ler_notas(&t).ok())
        .unwrap_or_default()
}

pub fn buscar_remota(url: &str) -> Result<NotasVersao, String> {
    let resposta = ureq::get(url)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| e.to_string())?;
    ler_notas(&resposta.into_string().map_err(|e| e.to_string())?)
}

pub fn procurar<'a>(n: &'a [NotaDeVersao], versao: &str) -> Option<&'a NotaDeVersao> {
    n.iter().find(|x| x.versao == versao)
}

pub fn aviso(n: &[NotaDeVersao], versao: &str) -> Aviso {
    match procurar(n, versao) {
        Some(x) => {
            let importante = x.tipo == TipoDeNota::Importante;
            Aviso {
                versao: versao.into(),
                titulo: format!(
                    "Novasce OS {versao} · {}",
                    if importante { "importante" } else { "opcional" }
                ),
                corpo: format!("{} {}", x.linha, x.custo),
                importante,
            }
        }
        None => Aviso {
            versao: versao.into(),
            titulo: format!("Novasce OS {versao} · opcional"),
            corpo: format!("Correções e melhorias. {CUSTO_PADRAO}"),
            importante: false,
        },
    }
}

fn numeros(versao: &str) -> Vec<u32> {
    versao.split('.').map(|p| p.parse().unwrap_or(0)).collect()
}

/// As últimas notas até a versão em uso, da mais nova para a mais velha.
pub fn recentes<'a>(n: &'a [NotaDeVersao], ate: &str, quantas: usize) -> Vec<&'a NotaDeVersao> {
    let limite = numeros(ate);
    let mut v: Vec<&NotaDeVersao> = n.iter().filter(|x| numeros(&x.versao) <= limite).collect();
    v.sort_by_key(|x| std::cmp::Reverse(numeros(&x.versao)));
    v.truncate(quantas);
    v
}

#[cfg(test)]
mod testes {
    use super::*;

    const NOTAS: &str = r#"[
      {"versao":"2026.10.30.1","tipo":"opcional","linha":"Deixa o Começar mais curto e acha a impressora da sede sozinho.","custo":"Reiniciar leva uns 2 minutos, e nada fecha sem você mandar."},
      {"versao":"2026.10.23.1","tipo":"importante","linha":"Corrige a volta à Sala.","custo":"Reiniciar leva uns 2 minutos, e nada fecha sem você mandar."}]"#;

    #[test]
    fn procura_pela_versao_da_imagem() {
        let n = ler_notas(NOTAS).unwrap();
        assert_eq!(
            procurar(&n, "2026.10.30.1").unwrap().linha,
            "Deixa o Começar mais curto e acha a impressora da sede sozinho."
        );
        assert!(procurar(&n, "2026.10.30.2").is_none());
    }

    #[test]
    fn aviso_com_e_sem_nota() {
        let n = ler_notas(NOTAS).unwrap();
        let a = aviso(&n, "2026.10.30.1");
        assert_eq!(a.titulo, "Novasce OS 2026.10.30.1 · opcional");
        assert_eq!(a.corpo, "Deixa o Começar mais curto e acha a impressora da sede sozinho. Reiniciar leva uns 2 minutos, e nada fecha sem você mandar.");
        assert!(!a.importante);
        assert!(aviso(&n, "2026.10.23.1").importante);
        let b = aviso(&[], "2026.11.06.1");
        assert_eq!(b.titulo, "Novasce OS 2026.11.06.1 · opcional");
        assert_eq!(b.corpo, format!("Correções e melhorias. {CUSTO_PADRAO}"));
    }

    #[test]
    fn recentes_ate_a_versao_atual_em_ordem() {
        let n = ler_notas(NOTAS).unwrap();
        let r = recentes(&n, "2026.10.23.1", 3);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].versao, "2026.10.23.1");
        assert_eq!(recentes(&n, "2026.11.01.1", 3)[0].versao, "2026.10.30.1");
    }

    #[test]
    fn arquivo_ausente_vira_lista_vazia() {
        assert!(ler_local("/nao/existe.json").is_empty());
    }
}
