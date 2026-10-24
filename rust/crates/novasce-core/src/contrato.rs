//! Tipos do contrato máquina ↔ Tenancy (índice dos marcos, seção 3). JSON em camelCase.
//! Todo tipo que carrega token tem `Debug` escrito à mão, para o token nunca ir para log.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Papel {
    Sede,
    Notebook,
}

impl Papel {
    pub fn como_texto(self) -> &'static str {
        match self {
            Papel::Sede => "sede",
            Papel::Notebook => "notebook",
        }
    }
}

impl FromStr for Papel {
    type Err = String;
    fn from_str(texto: &str) -> Result<Self, Self::Err> {
        match texto.trim() {
            "sede" => Ok(Papel::Sede),
            "notebook" => Ok(Papel::Notebook),
            outro => Err(format!("papel desconhecido: {outro}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Variante {
    Nvidia,
    Padrao,
}

// ---------- Máquina ----------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PedidoDeRegistro {
    pub role: Papel,
    pub variant: Variante,
    pub hostname: String,
    pub os_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RespostaDeRegistro {
    pub pairing_code: String,
    pub poll_token: String,
    pub expires_at: String,
}

/// Contrato v2: o `200` do poll só traz estes dois. Recusado vem como `403`
/// e expirado ou já entregue como `410` (o cliente traduz).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EstadoDoRegistro {
    Pending,
    Approved,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RespostaDaEspera {
    pub status: EstadoDoRegistro,
    #[serde(default)]
    pub machine_id: Option<String>,
    #[serde(default)]
    pub machine_token: Option<String>,
}

impl fmt::Debug for RespostaDaEspera {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RespostaDaEspera")
            .field("status", &self.status)
            .field("machine_id", &self.machine_id)
            .field(
                "machine_token",
                &self.machine_token.as_ref().map(|_| "(oculto)"),
            )
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pessoa {
    pub id: String,
    pub name: String,
    pub first_name: String,
    pub email: String,
    /// Até 14 dias depois da admissão (o M3 mostra as boas-vindas).
    pub is_new: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Maquina {
    pub id: String,
    pub name: String,
    pub role: Papel,
    pub owner: Option<Pessoa>,
    pub patrimony_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembroDoTime {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PedidoDeDono {
    pub user_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sinal {
    pub os_version: String,
    pub staged_version: Option<String>,
    pub disk_encrypted: bool,
    pub secure_boot: bool,
    pub agents_idle: Option<bool>,
}

// ---------- Pessoa (Começar e chamados, usados pelo M3) ----------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispositivoIniciado {
    pub user_code: String,
    pub verification_uri: String,
    pub device_code: String,
    pub interval: u64,
    pub expires_in: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PedidoDeToken {
    pub device_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PessoaComPapel {
    pub id: String,
    pub name: String,
    pub first_name: String,
    pub email: String,
    pub role: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenDaPessoa {
    pub user_token: String,
    pub user: PessoaComPapel,
}

impl fmt::Debug for TokenDaPessoa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenDaPessoa")
            .field("user_token", &"(oculto)")
            .field("user", &self.user)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Area {
    pub name: String,
    pub owner_name: String,
    pub doc_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimeiraTarefa {
    pub id: String,
    pub title: String,
    pub due_at: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Kit {
    pub available: bool,
    /// Caminho na API (`/os/me/kit`), baixado com o token da pessoa.
    #[serde(default)]
    pub download_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Espaco {
    pub slug: String,
    pub name: String,
    pub repo: String,
    pub member: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comecar {
    pub first_name: String,
    pub is_new: bool,
    /// A pessoa já teve outra máquina (troca de notebook).
    pub returning: bool,
    pub area: Option<Area>,
    pub first_task: Option<PrimeiraTarefa>,
    pub kit: Kit,
    pub spaces: Vec<Espaco>,
    pub email_signature_html: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChamadoAberto {
    pub ticket_id: String,
    /// O número que a pessoa vê ("#128").
    pub number: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TipoDeResposta {
    Pergunta,
    Solucao,
    Andamento,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RespostaDoChamado {
    pub id: String,
    #[serde(rename = "type")]
    pub tipo: TipoDeResposta,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chamado {
    pub id: String,
    pub number: u64,
    pub title: String,
    pub status: String,
    pub replies: Vec<RespostaDoChamado>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Retorno {
    pub resolved: bool,
}

// A tela (nvos_t_) não tem tipo aqui de propósito: o pareamento e o token da
// tela vivem na página /sala do Tenancy. O sistema nunca guarda token de tela.

// ---------- Erro no padrão Nest ----------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErroNest {
    pub status_code: Option<u16>,
    pub message: Value,
}

impl ErroNest {
    pub fn mensagem(&self) -> String {
        match &self.message {
            Value::String(texto) => texto.clone(),
            Value::Array(lista) => lista
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join("; "),
            outro => outro.to_string(),
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use serde_json::json;

    #[test]
    fn pedido_de_registro_vai_em_camel_case() {
        let pedido = PedidoDeRegistro {
            role: Papel::Sede,
            variant: Variante::Nvidia,
            hostname: "novasce-sede".into(),
            os_version: "2026.10.24.1".into(),
        };
        assert_eq!(
            serde_json::to_value(&pedido).unwrap(),
            json!({"role": "sede", "variant": "nvidia", "hostname": "novasce-sede", "osVersion": "2026.10.24.1"})
        );
    }

    #[test]
    fn sinal_manda_null_quando_nao_sabe() {
        let sinal = Sinal {
            os_version: "2026.10.24.1".into(),
            staged_version: None,
            disk_encrypted: true,
            secure_boot: false,
            agents_idle: None,
        };
        assert_eq!(
            serde_json::to_value(&sinal).unwrap(),
            json!({"osVersion": "2026.10.24.1", "stagedVersion": null, "diskEncrypted": true, "secureBoot": false, "agentsIdle": null})
        );
    }

    #[test]
    fn espera_aprovada_traz_o_token_e_o_debug_esconde() {
        let r: RespostaDaEspera = serde_json::from_value(
            json!({"status": "approved", "machineId": "m1", "machineToken": "nvos_m_segredo"}),
        )
        .unwrap();
        assert_eq!(r.status, EstadoDoRegistro::Approved);
        assert_eq!(r.machine_token.as_deref(), Some("nvos_m_segredo"));
        assert!(!format!("{r:?}").contains("segredo"));
    }

    #[test]
    fn espera_pendente_vem_sem_token() {
        let r: RespostaDaEspera = serde_json::from_value(json!({"status": "pending"})).unwrap();
        assert_eq!(r.status, EstadoDoRegistro::Pending);
        assert_eq!(r.machine_id, None);
    }

    #[test]
    fn espera_so_conhece_pendente_e_aprovado() {
        assert!(serde_json::from_value::<RespostaDaEspera>(json!({"status": "denied"})).is_err());
    }

    #[test]
    fn maquina_sem_dono() {
        let m: Maquina = serde_json::from_value(
            json!({"id": "m1", "name": "Sede", "role": "sede", "owner": null, "patrimonyCode": null}),
        )
        .unwrap();
        assert_eq!(m.role, Papel::Sede);
        assert!(m.owner.is_none());
    }

    #[test]
    fn dono_novo_vem_marcado() {
        let m: Maquina = serde_json::from_value(json!({
            "id": "m2", "name": "Notebook do Luan", "role": "notebook",
            "owner": {"id": "u1", "name": "Luan Silva", "firstName": "Luan", "email": "luan@novasce.com", "isNew": true},
            "patrimonyCode": "NB-007"
        }))
        .unwrap();
        let dono = m.owner.unwrap();
        assert!(dono.is_new);
        assert_eq!(dono.first_name, "Luan");
    }

    #[test]
    fn papel_vem_do_texto() {
        assert_eq!("sede".parse::<Papel>(), Ok(Papel::Sede));
        assert_eq!(" notebook\n".parse::<Papel>(), Ok(Papel::Notebook));
        assert!("servidor".parse::<Papel>().is_err());
        assert_eq!(Papel::Notebook.como_texto(), "notebook");
    }

    #[test]
    fn comecar_do_contrato() {
        let c: Comecar = serde_json::from_value(json!({
            "firstName": "Luan", "isNew": true, "returning": false,
            "area": {"name": "Dev", "ownerName": "Eduardo", "docUrl": "https://t/doc"},
            "firstTask": {"id": "t1", "title": "Ler o Manual", "dueAt": null, "url": "https://t/t1"},
            "kit": {"available": true, "downloadPath": "/os/me/kit"},
            "spaces": [{"slug": "studio", "name": "Studio", "repo": "Novasce/studio", "member": true}],
            "emailSignatureHtml": "<p>Luan</p>"
        }))
        .unwrap();
        assert!(c.is_new);
        assert!(!c.returning);
        assert_eq!(c.spaces[0].slug, "studio");
        assert_eq!(c.kit.download_path.as_deref(), Some("/os/me/kit"));
    }

    #[test]
    fn chamados_com_numero_e_resposta_tipada() {
        let aberto: ChamadoAberto =
            serde_json::from_value(json!({"ticketId": "c1", "number": 128})).unwrap();
        assert_eq!(aberto.number, 128);
        let lista: Vec<Chamado> = serde_json::from_value(json!([{
            "id": "c1", "number": 128, "title": "Impressora", "status": "REVIEW",
            "replies": [{"id": "r1", "type": "SOLUCAO", "body": "Reinstalei", "createdAt": "2026-10-24T12:00:00Z"}]
        }]))
        .unwrap();
        assert_eq!(lista[0].number, 128);
        assert_eq!(lista[0].replies[0].tipo, TipoDeResposta::Solucao);
    }

    #[test]
    fn token_da_pessoa_nao_aparece_no_debug() {
        let t: TokenDaPessoa = serde_json::from_value(json!({
            "userToken": "nvos_u_segredo",
            "user": {"id": "u1", "name": "Luan Silva", "firstName": "Luan", "email": "luan@novasce.com", "role": "DEV"}
        }))
        .unwrap();
        assert!(!format!("{t:?}").contains("segredo"));
    }

    #[test]
    fn erro_do_nest_com_texto_ou_lista() {
        let um: ErroNest =
            serde_json::from_value(json!({"statusCode": 403, "message": "Prazo encerrado"}))
                .unwrap();
        assert_eq!(um.mensagem(), "Prazo encerrado");
        let lista: ErroNest = serde_json::from_value(
            json!({"statusCode": 400, "message": ["userId vazio", "role inválido"]}),
        )
        .unwrap();
        assert_eq!(lista.mensagem(), "userId vazio; role inválido");
    }
}
