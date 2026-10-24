//! Chamado no contrato v2: `number` é o "#128" que a pessoa vê. Tenancy antigo sem `number`: nada de número.
use crate::nucleo::RespostaChamado;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeuChamado {
    pub id: String,
    #[serde(default)]
    pub number: Option<serde_json::Value>,
    pub title: String,
    pub status: String,
    pub replies: Vec<RespostaChamado>,
}

impl MeuChamado {
    pub fn numero(&self) -> Option<String> {
        numero_visivel(self.number.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Criado {
    pub ticket_id: String,
    pub numero: Option<String>,
}

pub fn numero_visivel(number: Option<&serde_json::Value>) -> Option<String> {
    match number? {
        serde_json::Value::Number(n) => Some(format!("#{n}")),
        serde_json::Value::String(t) if !t.trim().is_empty() => {
            Some(format!("#{}", t.trim().trim_start_matches('#')))
        }
        _ => None,
    }
}

/// A resposta de `POST /os/tickets`: `{ ticketId, number }`.
pub fn criado_de(v: &serde_json::Value) -> Criado {
    let ticket_id = match v.get("ticketId") {
        Some(serde_json::Value::String(t)) => t.clone(),
        Some(outro) => outro.to_string(),
        None => String::new(),
    };
    Criado {
        ticket_id,
        numero: numero_visivel(v.get("number")),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn criado_le_o_number_do_contrato() {
        assert_eq!(
            criado_de(&serde_json::json!({"ticketId":"t-9","number":128})),
            Criado {
                ticket_id: "t-9".into(),
                numero: Some("#128".into())
            }
        );
        assert_eq!(
            criado_de(&serde_json::json!({"ticketId":"t-9","number":"128"}))
                .numero
                .as_deref(),
            Some("#128")
        );
    }

    #[test]
    fn sem_number_nao_inventa_numero() {
        assert_eq!(
            criado_de(&serde_json::json!({"ticketId":"c9828ec6"})).numero,
            None
        );
    }

    #[test]
    fn meu_chamado_com_e_sem_number() {
        let l: Vec<MeuChamado> = serde_json::from_str(r#"[{"id":"a","number":128,"title":"x","status":"REVIEW","replies":[]},{"id":"b","title":"y","status":"BACKLOG","replies":[]}]"#).unwrap();
        assert_eq!(l[0].numero().as_deref(), Some("#128"));
        assert_eq!(l[1].numero(), None);
    }
}
