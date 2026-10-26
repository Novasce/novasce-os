//! Conversa com o novasce-sinal-local (/run/novasce/sinal.sock): o que precisa do token da máquina.
//! Uma linha JSON de pedido, uma linha JSON de resposta. O serviço sabe quem pede pelo SO_PEERCRED.
use crate::chamado::{criado_de, Criado, MeuChamado};
use crate::nucleo::{DeviceInicio, DeviceToken, Maquina};
use base64::Engine;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

pub const SOCKET: &str = "/run/novasce/sinal.sock";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum PedidoLocal {
    Maquina,
    Entrar,
    EntrarToken {
        #[serde(rename = "deviceCode")]
        device_code: String,
    },
    Chamado {
        texto: String,
        #[serde(rename = "machineInfo")]
        machine_info: serde_json::Value,
        /// PNG em base64, até 5 MB depois de decodificado.
        print: Option<String>,
    },
    Respostas {
        since: Option<String>,
    },
    Retorno {
        #[serde(rename = "ticketId")]
        ticket_id: String,
        resolved: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "r", rename_all = "snake_case")]
pub enum RespostaLocal {
    Ok {
        dados: serde_json::Value,
    },
    Pendente,
    /// codigo 0 = sem rede; 412 = máquina não registrada; os outros vêm do Tenancy.
    Erro {
        codigo: u16,
        mensagem: String,
    },
}

#[derive(Debug)]
pub enum ErroLocal {
    SemServico(String),
    SemRede(String),
    Recusado { codigo: u16, mensagem: String },
    Formato(String),
}

impl std::fmt::Display for ErroLocal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroLocal::SemServico(m) => write!(f, "o serviço da máquina não respondeu: {m}"),
            ErroLocal::SemRede(m) => write!(f, "sem internet: {m}"),
            ErroLocal::Recusado { codigo, mensagem } => {
                write!(f, "o Tenancy recusou ({codigo}): {mensagem}")
            }
            ErroLocal::Formato(m) => write!(f, "resposta inesperada: {m}"),
        }
    }
}

pub enum Saida<T> {
    Pronto(T),
    Pendente,
}

pub struct SinalLocal {
    caminho: PathBuf,
}

impl SinalLocal {
    pub fn padrao() -> Self {
        SinalLocal {
            caminho: SOCKET.into(),
        }
    }
    pub fn em(caminho: PathBuf) -> Self {
        SinalLocal { caminho }
    }

    pub fn pedir(&self, p: &PedidoLocal) -> Result<RespostaLocal, ErroLocal> {
        let mut s =
            UnixStream::connect(&self.caminho).map_err(|e| ErroLocal::SemServico(e.to_string()))?;
        s.set_read_timeout(Some(Duration::from_secs(40))).ok();
        let mut linha = serde_json::to_string(p).map_err(|e| ErroLocal::Formato(e.to_string()))?;
        linha.push('\n');
        s.write_all(linha.as_bytes())
            .map_err(|e| ErroLocal::SemServico(e.to_string()))?;
        let mut resposta = String::new();
        BufReader::new(&s)
            .read_line(&mut resposta)
            .map_err(|e| ErroLocal::SemServico(e.to_string()))?;
        serde_json::from_str(&resposta).map_err(|e| ErroLocal::Formato(e.to_string()))
    }

    fn pedir_tipado<T: DeserializeOwned>(&self, p: &PedidoLocal) -> Result<Saida<T>, ErroLocal> {
        match self.pedir(p)? {
            RespostaLocal::Ok { dados } => serde_json::from_value(dados)
                .map(Saida::Pronto)
                .map_err(|e| ErroLocal::Formato(e.to_string())),
            RespostaLocal::Pendente => Ok(Saida::Pendente),
            RespostaLocal::Erro {
                codigo: 0,
                mensagem,
            } => Err(ErroLocal::SemRede(mensagem)),
            RespostaLocal::Erro { codigo, mensagem } => {
                Err(ErroLocal::Recusado { codigo, mensagem })
            }
        }
    }

    fn pronto<T: DeserializeOwned>(&self, p: &PedidoLocal) -> Result<T, ErroLocal> {
        match self.pedir_tipado(p)? {
            Saida::Pronto(t) => Ok(t),
            Saida::Pendente => Err(ErroLocal::Formato("pendente fora de hora".into())),
        }
    }

    pub fn maquina(&self) -> Result<Maquina, ErroLocal> {
        self.pronto(&PedidoLocal::Maquina)
    }
    pub fn entrar(&self) -> Result<DeviceInicio, ErroLocal> {
        self.pronto(&PedidoLocal::Entrar)
    }
    pub fn entrar_token(&self, device_code: &str) -> Result<Saida<DeviceToken>, ErroLocal> {
        self.pedir_tipado(&PedidoLocal::EntrarToken {
            device_code: device_code.into(),
        })
    }
    pub fn chamado(
        &self,
        texto: &str,
        info: serde_json::Value,
        print: Option<&[u8]>,
    ) -> Result<Criado, ErroLocal> {
        let print = print.map(|b| base64::engine::general_purpose::STANDARD.encode(b));
        let v: serde_json::Value = self.pronto(&PedidoLocal::Chamado {
            texto: texto.into(),
            machine_info: info,
            print,
        })?;
        Ok(criado_de(&v))
    }
    pub fn respostas(&self, since: Option<&str>) -> Result<Vec<MeuChamado>, ErroLocal> {
        self.pronto(&PedidoLocal::Respostas {
            since: since.map(String::from),
        })
    }
    pub fn retorno(&self, ticket_id: &str, resolvido: bool) -> Result<(), ErroLocal> {
        self.pronto::<serde_json::Value>(&PedidoLocal::Retorno {
            ticket_id: ticket_id.into(),
            resolved: resolvido,
        })
        .map(|_| ())
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;

    fn servidor_falso(
        resposta: &'static str,
    ) -> (
        tempfile::TempDir,
        SinalLocal,
        std::thread::JoinHandle<String>,
    ) {
        let d = tempfile::tempdir().unwrap();
        let caminho = d.path().join("s.sock");
        let ouvinte = UnixListener::bind(&caminho).unwrap();
        let t = std::thread::spawn(move || {
            let (s, _) = ouvinte.accept().unwrap();
            let mut linha = String::new();
            BufReader::new(&s).read_line(&mut linha).unwrap();
            (&s).write_all(resposta.as_bytes()).unwrap();
            linha
        });
        (d, SinalLocal::em(caminho), t)
    }

    #[test]
    fn pede_codigo_de_entrar() {
        let (_d, s, t) = servidor_falso("{\"r\":\"ok\",\"dados\":{\"userCode\":\"ABCD-EFGH\",\"verificationUri\":\"https://t/os/entrar\",\"deviceCode\":\"dc\",\"interval\":5,\"expiresIn\":600}}\n");
        let i = s.entrar().unwrap();
        assert_eq!(i.user_code, "ABCD-EFGH");
        assert_eq!(t.join().unwrap().trim(), "{\"op\":\"entrar\"}");
    }

    #[test]
    fn pendente_vira_saida_pendente() {
        let (_d, s, _t) = servidor_falso("{\"r\":\"pendente\"}\n");
        assert!(matches!(s.entrar_token("dc").unwrap(), Saida::Pendente));
    }

    #[test]
    fn erro_zero_e_sem_rede() {
        let (_d, s, _t) =
            servidor_falso("{\"r\":\"erro\",\"codigo\":0,\"mensagem\":\"sem conexão\"}\n");
        assert!(matches!(s.maquina(), Err(ErroLocal::SemRede(_))));
    }

    #[test]
    fn sem_servico_quando_socket_nao_existe() {
        let s = SinalLocal::em("/nao/existe.sock".into());
        assert!(matches!(s.maquina(), Err(ErroLocal::SemServico(_))));
    }
}
