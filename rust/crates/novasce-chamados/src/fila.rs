//! Pedidos guardados sem internet: uma pasta por pedido, com pedido.json e print.png.
use crate::pedido::Pedido;
use std::path::PathBuf;

pub struct Fila {
    pasta: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Guardado {
    texto: String,
    dados_maquina: serde_json::Value,
}

impl Fila {
    pub fn em(pasta: PathBuf) -> Fila {
        Fila { pasta }
    }

    pub fn da_pessoa(c: &novasce_casa::caminhos::Caminhos) -> Fila {
        Fila::em(c.estado_novasce().join("chamados/fila"))
    }

    pub fn guardar(&self, p: &Pedido) -> std::io::Result<String> {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let id = format!("{n:032x}");
        let dir = self.pasta.join(&id);
        std::fs::create_dir_all(&dir)?;
        let g = Guardado {
            texto: p.texto.clone(),
            dados_maquina: p.dados_maquina.clone(),
        };
        novasce_casa::json_atomico::gravar(&dir.join("pedido.json"), &g)?;
        if let Some(b) = &p.print {
            std::fs::write(dir.join("print.png"), b)?;
        }
        Ok(id)
    }

    pub fn pendentes(&self) -> Vec<(String, Pedido)> {
        let mut ids: Vec<String> = std::fs::read_dir(&self.pasta)
            .map(|it| {
                it.flatten()
                    .filter(|e| e.path().is_dir())
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        ids.sort();
        ids.into_iter()
            .filter_map(|id| {
                let dir = self.pasta.join(&id);
                let g: Guardado =
                    serde_json::from_slice(&std::fs::read(dir.join("pedido.json")).ok()?).ok()?;
                let print = std::fs::read(dir.join("print.png")).ok();
                Some((
                    id,
                    Pedido {
                        texto: g.texto,
                        print,
                        dados_maquina: g.dados_maquina,
                    },
                ))
            })
            .collect()
    }

    pub fn tem_pendentes(&self) -> bool {
        !self.pendentes().is_empty()
    }

    pub fn remover(&self, id: &str) {
        let _ = std::fs::remove_dir_all(self.pasta.join(id));
    }

    /// Recusado pelo Tenancy: sai da fila, mas não é apagado (fica para a Ajuda mostrar).
    pub fn descartar(&self, id: &str) {
        let destino = self.pasta.with_file_name("recusados");
        let _ = std::fs::create_dir_all(&destino);
        let _ = std::fs::rename(self.pasta.join(id), destino.join(id));
    }
}
