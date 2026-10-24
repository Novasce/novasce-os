//! Tokens da pessoa: no chaveiro do sistema (Secret Service do KDE). Se ele não responder,
//! um arquivo 0600 na pasta de estado da pessoa, para nunca travar o Começar.
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

pub const CONTA_TENANCY: &str = "tenancy-pessoa";
pub const CONTA_GITHUB: &str = "github";
const SERVICO: &str = "novasce";

pub trait Chaveiro: Send + Sync {
    fn ler(&self, conta: &str) -> Option<String>;
    fn salvar(&self, conta: &str, valor: &str) -> Result<(), String>;
    fn apagar(&self, conta: &str);
}

#[derive(Default)]
pub struct ChaveiroMemoria(Mutex<HashMap<String, String>>);

impl Chaveiro for ChaveiroMemoria {
    fn ler(&self, conta: &str) -> Option<String> {
        self.0.lock().unwrap().get(conta).cloned()
    }
    fn salvar(&self, conta: &str, valor: &str) -> Result<(), String> {
        self.0.lock().unwrap().insert(conta.into(), valor.into());
        Ok(())
    }
    fn apagar(&self, conta: &str) {
        self.0.lock().unwrap().remove(conta);
    }
}

pub struct ChaveiroSistema {
    reserva: PathBuf,
    /// Só lido no Linux (Secret Service); fora dele só o arquivo vale.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    usar_sistema: bool,
}

impl ChaveiroSistema {
    pub fn da_pessoa(c: &crate::caminhos::Caminhos) -> Self {
        ChaveiroSistema {
            reserva: c.estado_novasce().join("segredos"),
            usar_sistema: true,
        }
    }
    pub fn so_arquivo(reserva: PathBuf) -> Self {
        ChaveiroSistema {
            reserva,
            usar_sistema: false,
        }
    }

    fn ler_arquivo(&self, conta: &str) -> Option<String> {
        std::fs::read_to_string(self.reserva.join(conta))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn salvar_arquivo(&self, conta: &str, valor: &str) -> Result<(), String> {
        use std::io::Write;
        use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&self.reserva)
            .map_err(|e| e.to_string())?;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(self.reserva.join(conta))
            .map_err(|e| e.to_string())?;
        f.write_all(valor.as_bytes()).map_err(|e| e.to_string())
    }
}

#[cfg(target_os = "linux")]
fn entrada(conta: &str) -> Option<keyring::Entry> {
    keyring::Entry::new(SERVICO, conta).ok()
}

impl Chaveiro for ChaveiroSistema {
    fn ler(&self, conta: &str) -> Option<String> {
        #[cfg(target_os = "linux")]
        if self.usar_sistema {
            if let Some(v) = entrada(conta).and_then(|e| e.get_password().ok()) {
                return Some(v);
            }
        }
        let _ = SERVICO;
        self.ler_arquivo(conta)
    }

    fn salvar(&self, conta: &str, valor: &str) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        if self.usar_sistema {
            if let Some(e) = entrada(conta) {
                if e.set_password(valor).is_ok() {
                    let _ = std::fs::remove_file(self.reserva.join(conta));
                    return Ok(());
                }
            }
        }
        self.salvar_arquivo(conta, valor)
    }

    fn apagar(&self, conta: &str) {
        #[cfg(target_os = "linux")]
        if self.usar_sistema {
            if let Some(e) = entrada(conta) {
                let _ = e.delete_credential();
            }
        }
        let _ = std::fs::remove_file(self.reserva.join(conta));
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn memoria_guarda_e_apaga() {
        let c = ChaveiroMemoria::default();
        assert_eq!(c.ler(CONTA_TENANCY), None);
        c.salvar(CONTA_TENANCY, "nvos_u_x").unwrap();
        assert_eq!(c.ler(CONTA_TENANCY).as_deref(), Some("nvos_u_x"));
        c.apagar(CONTA_TENANCY);
        assert_eq!(c.ler(CONTA_TENANCY), None);
    }

    #[test]
    fn reserva_em_arquivo_e_so_da_pessoa() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        let c = ChaveiroSistema::so_arquivo(d.path().join("segredos"));
        c.salvar(CONTA_GITHUB, "gho_x").unwrap();
        assert_eq!(c.ler(CONTA_GITHUB).as_deref(), Some("gho_x"));
        let modo = std::fs::metadata(d.path().join("segredos/github"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(modo, 0o600);
        c.apagar(CONTA_GITHUB);
        assert_eq!(c.ler(CONTA_GITHUB), None);
    }
}
