//! Onde cada coisa da pessoa mora. Tudo do M3 fica em ~/.local/state/novasce e ~/.config/novasce.
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Caminhos {
    pub casa: PathBuf,
    pub estado: PathBuf,
    pub config: PathBuf,
    pub runtime: PathBuf,
}

impl Caminhos {
    pub fn da_pessoa() -> Caminhos {
        let casa = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        let xdg = |var: &str, padrao: &str| {
            std::env::var_os(var)
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .unwrap_or_else(|| casa.join(padrao))
        };
        let runtime = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        Caminhos {
            estado: xdg("XDG_STATE_HOME", ".local/state"),
            config: xdg("XDG_CONFIG_HOME", ".config"),
            runtime,
            casa,
        }
    }

    /// Para testes: tudo dentro de uma pasta temporária.
    pub fn em(raiz: &Path) -> Caminhos {
        Caminhos {
            casa: raiz.join("casa"),
            estado: raiz.join("casa/.local/state"),
            config: raiz.join("casa/.config"),
            runtime: raiz.join("run"),
        }
    }

    pub fn estado_novasce(&self) -> PathBuf {
        self.estado.join("novasce")
    }
    pub fn config_novasce(&self) -> PathBuf {
        self.config.join("novasce")
    }
    pub fn runtime_novasce(&self) -> PathBuf {
        self.runtime.join("novasce")
    }
    pub fn brain(&self) -> PathBuf {
        self.casa.join("Brain")
    }
    pub fn espacos(&self) -> PathBuf {
        self.casa.join("Espaços")
    }

    pub fn documentos(&self) -> PathBuf {
        let dirs = std::fs::read_to_string(self.config.join("user-dirs.dirs")).unwrap_or_default();
        pasta_de_documentos(&dirs, &self.casa)
    }
}

pub fn pasta_de_documentos(user_dirs: &str, casa: &Path) -> PathBuf {
    for linha in user_dirs.lines() {
        if let Some(valor) = linha.trim().strip_prefix("XDG_DOCUMENTS_DIR=") {
            let v = valor.trim().trim_matches('"');
            if let Some(resto) = v.strip_prefix("$HOME/") {
                return casa.join(resto);
            }
            if v.starts_with('/') {
                return PathBuf::from(v);
            }
        }
    }
    casa.join("Documentos")
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn documentos_vem_do_user_dirs() {
        let casa = Path::new("/home/luan");
        let dirs = "# gerado\nXDG_DESKTOP_DIR=\"$HOME/Área de trabalho\"\nXDG_DOCUMENTS_DIR=\"$HOME/Documentos\"\n";
        assert_eq!(
            pasta_de_documentos(dirs, casa),
            PathBuf::from("/home/luan/Documentos")
        );
        assert_eq!(
            pasta_de_documentos("", casa),
            PathBuf::from("/home/luan/Documentos")
        );
        assert_eq!(
            pasta_de_documentos("XDG_DOCUMENTS_DIR=\"/dados/docs\"", casa),
            PathBuf::from("/dados/docs")
        );
    }

    #[test]
    fn caminhos_de_teste_ficam_dentro_da_raiz() {
        let c = Caminhos::em(Path::new("/tmp/x"));
        assert_eq!(c.brain(), PathBuf::from("/tmp/x/casa/Brain"));
        assert_eq!(c.espacos(), PathBuf::from("/tmp/x/casa/Espaços"));
        assert_eq!(
            c.estado_novasce(),
            PathBuf::from("/tmp/x/casa/.local/state/novasce")
        );
        assert_eq!(
            c.config_novasce(),
            PathBuf::from("/tmp/x/casa/.config/novasce")
        );
    }
}
