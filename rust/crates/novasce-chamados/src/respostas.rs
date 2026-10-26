//! Respostas novas dos chamados: o que avisar e com quais botões ("Isso resolveu?").
use novasce_casa::chamado::MeuChamado;
use novasce_casa::nucleo::TipoResposta;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Acompanhamento {
    pub desde: Option<String>,
    pub vistas: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Novidade {
    pub ticket_id: String,
    pub numero: Option<String>,
    pub titulo_chamado: String,
    pub tipo: TipoResposta,
    pub corpo: String,
}

#[derive(Debug, PartialEq)]
pub struct Textos {
    pub titulo: String,
    pub corpo: String,
    pub acoes: Vec<(&'static str, &'static str)>,
}

pub fn novidades(chamados: &[MeuChamado], a: &mut Acompanhamento) -> Vec<Novidade> {
    let mut saida = Vec::new();
    for c in chamados {
        for r in &c.replies {
            if a.vistas.contains(&r.id) {
                continue;
            }
            a.vistas.push(r.id.clone());
            if a.desde.as_deref().is_none_or(|d| r.created_at.as_str() > d) {
                a.desde = Some(r.created_at.clone());
            }
            saida.push(Novidade {
                ticket_id: c.id.clone(),
                numero: c.numero(),
                titulo_chamado: c.title.clone(),
                tipo: r.tipo,
                corpo: r.body.clone(),
            });
        }
    }
    let excesso = a.vistas.len().saturating_sub(500);
    a.vistas.drain(..excesso);
    saida
}

pub fn textos(n: &Novidade) -> Textos {
    let num = n
        .numero
        .as_ref()
        .map(|x| format!(" · {x}"))
        .unwrap_or_default();
    match n.tipo {
        TipoResposta::Solucao => Textos {
            titulo: format!("A Novasce respondeu{num}"),
            corpo: format!("{} Isso resolveu?", n.corpo),
            acoes: vec![("resolveu", "Resolveu"), ("ainda-nao", "Ainda não")],
        },
        TipoResposta::Pergunta => Textos {
            titulo: format!("A Novasce perguntou{num}"),
            corpo: format!(
                "{} Responda no Tenancy ou pelo Pedir ajuda, citando o chamado.",
                n.corpo
            ),
            acoes: vec![],
        },
        TipoResposta::Andamento => Textos {
            titulo: format!("Andamento do chamado{num}"),
            corpo: n.corpo.clone(),
            acoes: vec![],
        },
    }
}

pub fn caminho(c: &novasce_casa::caminhos::Caminhos, via: &str) -> std::path::PathBuf {
    c.estado_novasce()
        .join(format!("chamados/acompanhamento-{via}.json"))
}

#[cfg(test)]
mod testes {
    use super::*;

    fn chamados() -> Vec<MeuChamado> {
        serde_json::from_str(r#"[{"id":"t-128","number":128,"title":"Impressora","status":"REVIEW","replies":[
            {"id":"r1","type":"ANDAMENTO","body":"Estamos vendo.","createdAt":"2026-10-23T13:00:00Z"},
            {"id":"r2","type":"SOLUCAO","body":"A impressora da sede foi adicionada. Tente de novo.","createdAt":"2026-10-23T14:00:00Z"}]}]"#).unwrap()
    }

    #[test]
    fn so_avisa_o_que_nao_viu_e_avanca_o_desde() {
        let mut a = Acompanhamento::default();
        let n = novidades(&chamados(), &mut a);
        assert_eq!(n.len(), 2);
        assert_eq!(a.desde.as_deref(), Some("2026-10-23T14:00:00Z"));
        assert!(novidades(&chamados(), &mut a).is_empty());
    }

    #[test]
    fn textos_da_solucao_perguntam_se_resolveu() {
        let mut a = Acompanhamento::default();
        let n = novidades(&chamados(), &mut a);
        let t = textos(&n[1]);
        assert_eq!(t.titulo, "A Novasce respondeu · #128");
        assert_eq!(
            t.corpo,
            "A impressora da sede foi adicionada. Tente de novo. Isso resolveu?"
        );
        assert_eq!(
            t.acoes,
            vec![("resolveu", "Resolveu"), ("ainda-nao", "Ainda não")]
        );
        assert_eq!(textos(&n[0]).acoes, vec![]);
    }
}
