use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelaInicial {
    pub modo: String,
    pub tela: String,
    pub detalhe: Option<String>,
}

pub fn ler(args: &[String]) -> TelaInicial {
    let valor = |nome: &str| args.iter().position(|a| a == nome).and_then(|i| args.get(i + 1)).cloned();
    let tela = valor("--tela").unwrap_or_else(|| "geral".into());
    let (modo, detalhe) = if args.iter().any(|a| a == "--pedir") {
        ("pedir", None)
    } else if args.iter().any(|a| a == "--recomendados") {
        ("recomendados", None)
    } else if args.iter().any(|a| a == "--esqueci") {
        ("esqueci", None)
    } else if let Some(f) = valor("--ficha") {
        ("detalhe", Some(format!("ficha:{f}")))
    } else if let Some(a) = valor("--artigo") {
        ("detalhe", Some(a))
    } else {
        ("central", None)
    };
    TelaInicial { modo: modo.into(), tela, detalhe }
}

pub fn precisa_print(t: &TelaInicial) -> bool {
    t.modo == "central" || t.modo == "pedir"
}

#[cfg(test)]
mod testes {
    use super::*;
    fn a(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn modos() {
        assert_eq!(ler(&a(&["novasce-ajuda", "--central", "--tela", "comecar"])), TelaInicial { modo: "central".into(), tela: "comecar".into(), detalhe: None });
        assert_eq!(ler(&a(&["x", "--pedir"])).modo, "pedir");
        assert_eq!(ler(&a(&["x", "--ficha", "drive"])).detalhe.as_deref(), Some("ficha:drive"));
        assert_eq!(ler(&a(&["x", "--artigo", "guia:primeiro-dia"])).detalhe.as_deref(), Some("guia:primeiro-dia"));
        assert_eq!(ler(&a(&["x"])).modo, "central");
        assert!(!precisa_print(&ler(&a(&["x", "--esqueci"]))));
    }
}
