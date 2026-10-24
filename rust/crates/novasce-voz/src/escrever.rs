//! Escrever no campo do cursor no Plasma Wayland: área de transferência + Shift+Insert pelo ydotool.
use std::io::Write;
use std::process::{Command, Stdio};

pub trait Sistema {
    fn rodar(&self, argv: &[&str], entrada: Option<&[u8]>) -> Result<Vec<u8>, String>;
    fn esperar(&self, ms: u64);
}

pub struct SistemaReal;

impl Sistema for SistemaReal {
    fn rodar(&self, argv: &[&str], entrada: Option<&[u8]>) -> Result<Vec<u8>, String> {
        let mut c = Command::new(argv[0]);
        c.args(&argv[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        c.stdin(if entrada.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        let mut filho = c.spawn().map_err(|e| format!("{}: {e}", argv[0]))?;
        if let (Some(b), Some(mut i)) = (entrada, filho.stdin.take()) {
            i.write_all(b).map_err(|e| e.to_string())?;
        }
        let s = filho.wait_with_output().map_err(|e| e.to_string())?;
        if s.status.success() {
            Ok(s.stdout)
        } else {
            Err(String::from_utf8_lossy(&s.stderr).trim().to_string())
        }
    }
    fn esperar(&self, ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

#[derive(Debug, PartialEq)]
pub enum ErroEscrever {
    /// O ydotool não colou; o texto está na área de transferência.
    FicouCopiado,
    NaoCopiou(String),
}

impl ErroEscrever {
    pub fn texto(&self) -> &str {
        match self {
            ErroEscrever::FicouCopiado => "O texto está copiado. Cole com Ctrl+V.",
            ErroEscrever::NaoCopiou(_) => "A voz não escreveu nada desta vez. Tente de novo.",
        }
    }
}

const TIPO: &str = "text/plain;charset=utf-8";

pub fn escrever_no_cursor(s: &dyn Sistema, texto: &str) -> Result<(), ErroEscrever> {
    let anterior = s
        .rodar(&["wl-paste", "--no-newline", "--type", "text/plain"], None)
        .ok();
    s.rodar(&["wl-copy", "--type", TIPO], Some(texto.as_bytes()))
        .map_err(ErroEscrever::NaoCopiou)?;
    s.rodar(
        &["wl-copy", "--primary", "--type", TIPO],
        Some(texto.as_bytes()),
    )
    .map_err(ErroEscrever::NaoCopiou)?;
    s.esperar(80);
    // KEY_LEFTSHIFT = 42, KEY_INSERT = 110 (linux/input-event-codes.h)
    s.rodar(&["ydotool", "key", "42:1", "110:1", "110:0", "42:0"], None)
        .map_err(|_| ErroEscrever::FicouCopiado)?;
    s.esperar(400);
    if let Some(a) = anterior.filter(|a| !a.is_empty()) {
        let _ = s.rodar(&["wl-copy", "--type", TIPO], Some(&a));
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::cell::RefCell;

    struct Falso {
        chamadas: RefCell<Vec<(String, Option<String>)>>,
        anterior: Option<&'static str>,
        ydotool_falha: bool,
    }
    impl Sistema for Falso {
        fn rodar(&self, argv: &[&str], entrada: Option<&[u8]>) -> Result<Vec<u8>, String> {
            self.chamadas.borrow_mut().push((
                argv.join(" "),
                entrada.map(|b| String::from_utf8_lossy(b).into()),
            ));
            match argv[0] {
                "wl-paste" => self
                    .anterior
                    .map(|a| a.as_bytes().to_vec())
                    .ok_or_else(|| "vazio".into()),
                "ydotool" if self.ydotool_falha => Err("sem uinput".into()),
                _ => Ok(vec![]),
            }
        }
        fn esperar(&self, _: u64) {}
    }

    #[test]
    fn copia_cola_com_shift_insert_e_devolve_a_area() {
        let s = Falso {
            chamadas: RefCell::new(vec![]),
            anterior: Some("antes"),
            ydotool_falha: false,
        };
        escrever_no_cursor(&s, "Conferir os insumos até sexta.").unwrap();
        let c = s.chamadas.borrow();
        assert_eq!(
            c[1],
            (
                "wl-copy --type text/plain;charset=utf-8".into(),
                Some("Conferir os insumos até sexta.".into())
            )
        );
        assert_eq!(c[2].0, "wl-copy --primary --type text/plain;charset=utf-8");
        assert_eq!(c[3].0, "ydotool key 42:1 110:1 110:0 42:0");
        assert_eq!(
            c[4],
            (
                "wl-copy --type text/plain;charset=utf-8".into(),
                Some("antes".into())
            )
        );
    }

    #[test]
    fn sem_ydotool_o_texto_fica_copiado() {
        let s = Falso {
            chamadas: RefCell::new(vec![]),
            anterior: Some("antes"),
            ydotool_falha: true,
        };
        assert_eq!(
            escrever_no_cursor(&s, "oi").unwrap_err(),
            ErroEscrever::FicouCopiado
        );
        assert_eq!(s.chamadas.borrow().len(), 4);
    }
}
