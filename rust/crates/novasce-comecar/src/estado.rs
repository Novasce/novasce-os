use novasce_casa::{caminhos::Caminhos, json_atomico};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Modo {
    #[default]
    Normal,
    DeVolta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tela {
    BoasVindas,
    Passo1,
    Passo2,
    Passo3,
    Espacos,
    DeVolta,
    Fim,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Pessoa {
    pub first_name: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Extras {
    pub nuvem: bool,
    pub senhas: bool,
    pub voz: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaVisao {
    pub nome: String,
    pub dono: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TarefaVisao {
    pub titulo: String,
    pub prazo: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoasVindas {
    pub first_name: String,
    pub area: Option<AreaVisao>,
    pub primeira_tarefa: Option<TarefaVisao>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct EstadoComecar {
    pub tenancy: bool,
    pub ia: bool,
    pub github: bool,
    pub espacos_vistos: bool,
    pub pessoa_nova: bool,
    pub boas_vindas_vista: bool,
    pub boas_vindas: Option<BoasVindas>,
    pub fim_visto: bool,
    pub modo: Modo,
    pub pessoa: Option<Pessoa>,
    pub github_login: Option<String>,
    pub brain_repo: Option<String>,
    pub ias: Vec<String>,
    pub extras: Extras,
}

impl EstadoComecar {
    pub fn passos_feitos(&self) -> u8 {
        [self.tenancy, self.ia, self.github && self.espacos_vistos]
            .iter()
            .filter(|x| **x)
            .count() as u8
    }

    pub fn tudo_feito(&self) -> bool {
        self.passos_feitos() == 3
    }

    /// Abre sozinho no login até os três passos estarem feitos e o fim ter sido visto.
    pub fn deve_abrir_sozinho(&self) -> bool {
        !(self.tudo_feito() && self.fim_visto)
    }

    pub fn tela(&self) -> Tela {
        if self.pessoa_nova && !self.boas_vindas_vista {
            return Tela::BoasVindas;
        }
        if !self.tenancy {
            return Tela::Passo1;
        }
        if self.modo == Modo::DeVolta && !self.fim_visto {
            return Tela::DeVolta;
        }
        if !self.ia {
            return Tela::Passo2;
        }
        if !self.github {
            return Tela::Passo3;
        }
        if !self.espacos_vistos {
            return Tela::Espacos;
        }
        Tela::Fim
    }
}

/// Lê as boas-vindas de `GET /os/me/comecar` ou de `owner` em `GET /os/machine` (contrato v2, `isNew`).
/// Só devolve algo quando `isNew` é verdadeiro.
pub fn boas_vindas_de(v: &serde_json::Value) -> Option<BoasVindas> {
    if v.get("isNew").and_then(|x| x.as_bool()) != Some(true) {
        return None;
    }
    let texto =
        |o: &serde_json::Value, k: &str| o.get(k).and_then(|x| x.as_str()).map(String::from);
    let area = v.get("area").filter(|a| a.is_object()).map(|a| AreaVisao {
        nome: texto(a, "name").unwrap_or_default(),
        dono: texto(a, "ownerName").unwrap_or_default(),
        url: texto(a, "docUrl").unwrap_or_default(),
    });
    let primeira_tarefa = v
        .get("firstTask")
        .filter(|t| t.is_object())
        .map(|t| TarefaVisao {
            titulo: texto(t, "title").unwrap_or_default(),
            prazo: texto(t, "dueAt"),
            url: texto(t, "url").unwrap_or_default(),
        });
    Some(BoasVindas {
        first_name: texto(v, "firstName").unwrap_or_default(),
        area,
        primeira_tarefa,
    })
}

fn caminho(c: &Caminhos) -> std::path::PathBuf {
    c.estado_novasce().join("comecar.json")
}

pub fn ler(c: &Caminhos) -> EstadoComecar {
    json_atomico::ler(&caminho(c))
}

pub fn gravar(c: &Caminhos, e: &EstadoComecar) -> std::io::Result<()> {
    json_atomico::gravar(&caminho(c), e)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn ordem_dos_passos() {
        let mut e = EstadoComecar::default();
        assert_eq!(e.tela(), Tela::Passo1);
        e.tenancy = true;
        assert_eq!(e.tela(), Tela::Passo2);
        e.ia = true;
        assert_eq!(e.tela(), Tela::Passo3);
        e.github = true;
        assert_eq!(e.tela(), Tela::Espacos);
        assert_eq!(e.passos_feitos(), 2);
        e.espacos_vistos = true;
        assert_eq!(e.tela(), Tela::Fim);
        assert_eq!(e.passos_feitos(), 3);
    }

    #[test]
    fn pessoa_nova_ve_boas_vindas_antes_e_uma_vez() {
        let mut e = EstadoComecar {
            pessoa_nova: true,
            ..Default::default()
        };
        assert_eq!(e.tela(), Tela::BoasVindas);
        e.boas_vindas_vista = true;
        assert_eq!(e.tela(), Tela::Passo1);
    }

    #[test]
    fn de_volta_depois_do_tenancy() {
        let e = EstadoComecar {
            tenancy: true,
            modo: Modo::DeVolta,
            ..Default::default()
        };
        assert_eq!(e.tela(), Tela::DeVolta);
    }

    #[test]
    fn abre_sozinho_ate_ver_o_fim() {
        let mut e = EstadoComecar {
            tenancy: true,
            ia: true,
            github: true,
            espacos_vistos: true,
            ..Default::default()
        };
        assert!(e.deve_abrir_sozinho());
        e.fim_visto = true;
        assert!(!e.deve_abrir_sozinho());
    }

    #[test]
    fn grava_e_le() {
        let d = tempfile::tempdir().unwrap();
        let c = Caminhos::em(d.path());
        let e = EstadoComecar {
            tenancy: true,
            ias: vec!["claude".into()],
            ..Default::default()
        };
        gravar(&c, &e).unwrap();
        assert_eq!(ler(&c), e);
    }

    #[test]
    fn boas_vindas_do_comecar_e_da_maquina() {
        let comecar = serde_json::json!({"firstName":"Ana","isNew":true,
            "area":{"name":"Comercial","ownerName":"Eduardo","docUrl":"https://t/a"},
            "firstTask":{"id":"t","title":"Conhecer o SOPEC por dentro","dueAt":"2026-10-24T21:00:00Z","url":"https://t/t"}});
        let b = boas_vindas_de(&comecar).unwrap();
        assert_eq!(b.first_name, "Ana");
        assert_eq!(b.area.unwrap().dono, "Eduardo");
        assert_eq!(
            b.primeira_tarefa.unwrap().titulo,
            "Conhecer o SOPEC por dentro"
        );
        assert!(boas_vindas_de(&serde_json::json!({"firstName":"Ana","isNew":false})).is_none());
        let maquina = serde_json::json!({"id":"m","owner":{"firstName":"Ana","isNew":true,"area":null,"firstTask":null}});
        assert_eq!(boas_vindas_de(&maquina["owner"]).unwrap().first_name, "Ana");
        assert!(boas_vindas_de(
            &serde_json::json!({"id":"m","owner":{"firstName":"Ana"}})["owner"]
        )
        .is_none());
    }
}
