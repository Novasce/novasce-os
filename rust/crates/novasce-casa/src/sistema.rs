//! Leituras do sistema que não pedem root: versão atual e pronta (rpm-ostree), disco cifrado, papel.
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub struct Implantacao {
    pub versao: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StatusOstree {
    pub atual: Option<Implantacao>,
    pub pronta: Option<Implantacao>,
}

pub fn ler_status_ostree(json: &str) -> Result<StatusOstree, String> {
    #[derive(Deserialize)]
    struct Raiz {
        deployments: Vec<Imp>,
    }
    #[derive(Deserialize)]
    struct Imp {
        #[serde(default)]
        booted: bool,
        #[serde(default)]
        staged: bool,
        version: Option<String>,
        #[serde(default)]
        timestamp: i64,
    }
    let raiz: Raiz =
        serde_json::from_str(json).map_err(|e| format!("rpm-ostree status ilegível: {e}"))?;
    let mut s = StatusOstree::default();
    for d in raiz.deployments {
        let i = Implantacao {
            versao: d.version,
            timestamp: d.timestamp,
        };
        if d.booted {
            s.atual = Some(i);
        } else if d.staged {
            s.pronta = Some(i);
        }
    }
    Ok(s)
}

pub fn status_ostree() -> Result<StatusOstree, String> {
    let saida = Command::new("rpm-ostree")
        .args(["status", "--json"])
        .output()
        .map_err(|e| e.to_string())?;
    if !saida.status.success() {
        return Err(String::from_utf8_lossy(&saida.stderr).into_owned());
    }
    ler_status_ostree(&String::from_utf8_lossy(&saida.stdout))
}

/// A versão que entra no próximo reinício, se for diferente da atual.
pub fn versao_pronta(s: &StatusOstree) -> Option<String> {
    let pronta = s.pronta.as_ref()?.versao.clone()?;
    let atual = s.atual.as_ref().and_then(|a| a.versao.clone());
    (Some(&pronta) != atual.as_ref()).then_some(pronta)
}

pub fn dias_desde(timestamp: i64, agora: i64) -> i64 {
    (agora - timestamp).max(0) / 86_400
}

pub fn agora_unix() -> i64 {
    chrono::Utc::now().timestamp()
}

pub fn disco_cifrado(lsblk_json: &str) -> bool {
    fn tem_crypt(v: &serde_json::Value) -> bool {
        match v {
            serde_json::Value::Object(m) => {
                m.get("type").and_then(|t| t.as_str()) == Some("crypt") || m.values().any(tem_crypt)
            }
            serde_json::Value::Array(a) => a.iter().any(tem_crypt),
            _ => false,
        }
    }
    serde_json::from_str::<serde_json::Value>(lsblk_json)
        .map(|v| tem_crypt(&v))
        .unwrap_or(false)
}

pub fn disco_cifrado_agora() -> bool {
    Command::new("lsblk")
        .args(["-J", "-o", "NAME,TYPE"])
        .output()
        .map(|s| disco_cifrado(&String::from_utf8_lossy(&s.stdout)))
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Papel {
    Sede,
    Notebook,
    Desconhecido,
}

pub fn papel_de(texto: &str) -> Papel {
    match texto.trim() {
        "sede" => Papel::Sede,
        "notebook" => Papel::Notebook,
        _ => Papel::Desconhecido,
    }
}

pub fn papel() -> Papel {
    papel_de(&std::fs::read_to_string("/etc/novasce/papel").unwrap_or_default())
}

pub fn conta_atual() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_default()
}

/// "Notebook do Luan" vira "notebook-do-luan": vai no nome da cópia de conflito.
pub fn nome_para_arquivo(nome: &str) -> String {
    let mut s = String::new();
    for c in nome.to_lowercase().chars() {
        let c = match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'ê' | 'è' => 'e',
            'í' | 'î' => 'i',
            'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ü' => 'u',
            'ç' => 'c',
            c => c,
        };
        if c.is_ascii_alphanumeric() {
            s.push(c);
        } else if !s.ends_with('-') {
            s.push('-');
        }
    }
    s.trim_matches('-').to_string()
}

#[cfg(test)]
mod testes {
    use super::*;

    const STATUS: &str = r#"{"deployments":[
        {"booted":false,"staged":true,"version":"2026.10.30.1","timestamp":1761800000,"container-image-reference":"ostree-image-signed:docker://ghcr.io/novasce/novasce-os:stable-padrao"},
        {"booted":true,"staged":false,"version":"2026.10.23.1","timestamp":1761200000},
        {"booted":false,"staged":false,"version":"2026.10.16.1","timestamp":1760600000}]}"#;

    #[test]
    fn le_atual_e_pronta() {
        let s = ler_status_ostree(STATUS).unwrap();
        assert_eq!(
            s.atual.as_ref().unwrap().versao.as_deref(),
            Some("2026.10.23.1")
        );
        assert_eq!(versao_pronta(&s).as_deref(), Some("2026.10.30.1"));
    }

    #[test]
    fn sem_pronta_quando_igual_ou_ausente() {
        let s = ler_status_ostree(
            r#"{"deployments":[{"booted":true,"version":"2026.10.23.1","timestamp":1}]}"#,
        )
        .unwrap();
        assert_eq!(versao_pronta(&s), None);
    }

    #[test]
    fn dias_desde_conta_dias_inteiros() {
        assert_eq!(dias_desde(1761200000, 1761200000 + 2 * 86400 + 5), 2);
        assert_eq!(dias_desde(10, 5), 0);
    }

    #[test]
    fn disco_cifrado_procura_crypt_na_arvore() {
        let sim = r#"{"blockdevices":[{"name":"nvme0n1","type":"disk","children":[{"name":"p3","type":"part","children":[{"name":"luks-1","type":"crypt"}]}]}]}"#;
        let nao = r#"{"blockdevices":[{"name":"sda","type":"disk","children":[{"name":"sda1","type":"part"}]}]}"#;
        assert!(disco_cifrado(sim));
        assert!(!disco_cifrado(nao));
        assert!(!disco_cifrado("lixo"));
    }

    #[test]
    fn papel_e_nome_para_arquivo() {
        assert_eq!(papel_de("sede\n"), Papel::Sede);
        assert_eq!(papel_de("notebook"), Papel::Notebook);
        assert_eq!(papel_de(""), Papel::Desconhecido);
        assert_eq!(nome_para_arquivo("Notebook do Luan"), "notebook-do-luan");
        assert_eq!(
            nome_para_arquivo("Sede · São Cristóvão"),
            "sede-sao-cristovao"
        );
    }
}
