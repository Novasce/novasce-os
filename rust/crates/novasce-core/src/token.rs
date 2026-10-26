//! Guarda do token da máquina: `/var/lib/novasce/maquina.json`, 0600, dono root.
//! O token só escreve o estado da própria máquina (contrato, seção 3).
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

pub const CAMINHO_DA_MAQUINA: &str = "/var/lib/novasce/maquina.json";
pub const PREFIXO_DA_MAQUINA: &str = "nvos_m_";

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaquinaSalva {
    pub machine_id: String,
    pub machine_token: String,
}

impl fmt::Debug for MaquinaSalva {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaquinaSalva")
            .field("machine_id", &self.machine_id)
            .field("machine_token", &"(oculto)")
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ErroToken {
    #[error("não deu para ler ou gravar {caminho}: {fonte}")]
    Arquivo {
        caminho: String,
        fonte: std::io::Error,
    },
    #[error("{caminho} está aberto para outros usuários (modo {modo:o}); precisa ser 0600")]
    PermissaoAberta { caminho: String, modo: u32 },
    #[error("{0} não tem um token de máquina válido")]
    Formato(String),
}

pub struct GuardaDoToken {
    caminho: PathBuf,
}

impl GuardaDoToken {
    pub fn padrao() -> Self {
        Self::em(CAMINHO_DA_MAQUINA)
    }

    pub fn em(caminho: impl Into<PathBuf>) -> Self {
        Self {
            caminho: caminho.into(),
        }
    }

    pub fn caminho(&self) -> &Path {
        &self.caminho
    }

    pub fn salvar(&self, maquina: &MaquinaSalva) -> Result<(), ErroToken> {
        if !maquina.machine_token.starts_with(PREFIXO_DA_MAQUINA) {
            return Err(ErroToken::Formato(self.texto()));
        }
        let erro = |fonte| ErroToken::Arquivo {
            caminho: self.texto(),
            fonte,
        };
        if let Some(pasta) = self.caminho.parent() {
            DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(pasta)
                .map_err(erro)?;
        }
        let novo = self.caminho.with_extension("json.novo");
        let mut arquivo = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&novo)
            .map_err(erro)?;
        fs::set_permissions(&novo, fs::Permissions::from_mode(0o600)).map_err(erro)?;
        let json = serde_json::to_vec(maquina).expect("MaquinaSalva sempre vira JSON");
        arquivo.write_all(&json).map_err(erro)?;
        arquivo.sync_all().map_err(erro)?;
        fs::rename(&novo, &self.caminho).map_err(erro)?;
        Ok(())
    }

    pub fn ler(&self) -> Result<Option<MaquinaSalva>, ErroToken> {
        let meta = match fs::metadata(&self.caminho) {
            Ok(meta) => meta,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(fonte) => {
                return Err(ErroToken::Arquivo {
                    caminho: self.texto(),
                    fonte,
                })
            }
        };
        let modo = meta.permissions().mode() & 0o777;
        if modo & 0o077 != 0 {
            return Err(ErroToken::PermissaoAberta {
                caminho: self.texto(),
                modo,
            });
        }
        let texto = fs::read_to_string(&self.caminho).map_err(|fonte| ErroToken::Arquivo {
            caminho: self.texto(),
            fonte,
        })?;
        let maquina: MaquinaSalva =
            serde_json::from_str(&texto).map_err(|_| ErroToken::Formato(self.texto()))?;
        if !maquina.machine_token.starts_with(PREFIXO_DA_MAQUINA) {
            return Err(ErroToken::Formato(self.texto()));
        }
        Ok(Some(maquina))
    }

    fn texto(&self) -> String {
        self.caminho.display().to_string()
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn maquina() -> MaquinaSalva {
        MaquinaSalva {
            machine_id: "m1".into(),
            machine_token: "nvos_m_abc123".into(),
        }
    }

    #[test]
    fn salva_em_0600_numa_pasta_0700_e_le_de_volta() {
        let raiz = tempfile::tempdir().unwrap();
        let guarda = GuardaDoToken::em(raiz.path().join("novasce/maquina.json"));
        guarda.salvar(&maquina()).unwrap();
        let modo = fs::metadata(guarda.caminho()).unwrap().permissions().mode() & 0o777;
        assert_eq!(modo, 0o600);
        let pasta = fs::metadata(raiz.path().join("novasce"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(pasta, 0o700);
        assert_eq!(guarda.ler().unwrap(), Some(maquina()));
    }

    #[test]
    fn sem_arquivo_e_maquina_sem_registro() {
        let raiz = tempfile::tempdir().unwrap();
        assert_eq!(
            GuardaDoToken::em(raiz.path().join("maquina.json"))
                .ler()
                .unwrap(),
            None
        );
    }

    #[test]
    fn arquivo_aberto_para_outros_e_recusado() {
        let raiz = tempfile::tempdir().unwrap();
        let guarda = GuardaDoToken::em(raiz.path().join("maquina.json"));
        guarda.salvar(&maquina()).unwrap();
        fs::set_permissions(guarda.caminho(), fs::Permissions::from_mode(0o644)).unwrap();
        assert!(matches!(
            guarda.ler(),
            Err(ErroToken::PermissaoAberta { modo: 0o644, .. })
        ));
    }

    #[test]
    fn token_sem_prefixo_nao_e_salvo() {
        let raiz = tempfile::tempdir().unwrap();
        let guarda = GuardaDoToken::em(raiz.path().join("maquina.json"));
        let errado = MaquinaSalva {
            machine_id: "m1".into(),
            machine_token: "abc".into(),
        };
        assert!(matches!(guarda.salvar(&errado), Err(ErroToken::Formato(_))));
        assert!(!guarda.caminho().exists());
    }

    #[test]
    fn debug_nunca_mostra_o_token() {
        assert!(!format!("{:?}", maquina()).contains("abc123"));
    }
}
