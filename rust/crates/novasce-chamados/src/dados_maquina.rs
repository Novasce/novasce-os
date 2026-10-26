//! Os dados que vão junto com o chamado. Nunca arquivos, senhas ou o brain.

pub struct Fatos {
    pub versao_visivel: String,
    pub imagem: Option<String>,
    pub nome_maquina: Option<String>,
    pub papel: String,
    pub dias_desde_atualizacao: Option<i64>,
    pub cifrado: bool,
    pub app: String,
}

pub struct DadosMaquina {
    pub linha: String,
    pub json: serde_json::Value,
}

fn primeira_minuscula(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(p) => p.to_lowercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

pub fn montar(f: &Fatos) -> DadosMaquina {
    let mut partes = vec![format!("Novasce OS {}", f.versao_visivel)];
    if let Some(n) = &f.nome_maquina {
        partes.push(primeira_minuscula(n));
    }
    if let Some(d) = f.dias_desde_atualizacao {
        partes.push(match d {
            0 => "atualizado hoje".into(),
            1 => "atualizado ontem".into(),
            n => format!("atualizado há {n} dias"),
        });
    }
    partes.push(if f.cifrado {
        "disco cifrado".into()
    } else {
        "disco sem cifra".into()
    });
    DadosMaquina {
        linha: partes.join(" · "),
        json: serde_json::json!({
            "versao": f.versao_visivel,
            "imagem": f.imagem,
            "maquina": f.nome_maquina,
            "papel": f.papel,
            "diasDesdeAtualizacao": f.dias_desde_atualizacao,
            "discoCifrado": f.cifrado,
            "app": f.app,
        }),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn linha_igual_ao_canvas() {
        let f = Fatos {
            versao_visivel: "1.0.3".into(),
            imagem: Some("2026.10.23.1".into()),
            nome_maquina: Some("Notebook do Luan".into()),
            papel: "notebook".into(),
            dias_desde_atualizacao: Some(2),
            cifrado: true,
            app: "arquivos".into(),
        };
        let d = montar(&f);
        assert_eq!(
            d.linha,
            "Novasce OS 1.0.3 · notebook do Luan · atualizado há 2 dias · disco cifrado"
        );
        assert_eq!(d.json["discoCifrado"], true);
        assert_eq!(d.json["app"], "arquivos");
    }

    #[test]
    fn hoje_ontem_e_sem_nome() {
        let mut f = Fatos {
            versao_visivel: "1.0.3".into(),
            imagem: None,
            nome_maquina: None,
            papel: "sede".into(),
            dias_desde_atualizacao: Some(0),
            cifrado: false,
            app: "geral".into(),
        };
        assert_eq!(
            montar(&f).linha,
            "Novasce OS 1.0.3 · atualizado hoje · disco sem cifra"
        );
        f.dias_desde_atualizacao = Some(1);
        assert!(montar(&f).linha.contains("atualizado ontem"));
    }
}
