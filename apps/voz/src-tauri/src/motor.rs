use novasce_voz::escrever::{escrever_no_cursor, SistemaReal};
use novasce_voz::gesto::{Acao, Entrada, Gesto};
use novasce_voz::gravacao::{silencio_no_fim, Gravador, TAXA};
use novasce_voz::vocabulario::{aplicar, Vocabulario};
use novasce_voz::whisper::{transcrever, ErroVoz, Whisper};
use serde::Serialize;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

#[derive(Clone, Serialize)]
pub struct EstadoVoz {
    pub fase: &'static str,
    pub nivel: f32,
    pub mensagem: Option<String>,
}

pub struct Motor {
    pub whisper: Whisper,
    pub vocabulario: Vocabulario,
    pub qualquer_campo: bool,
    pub cpu_ok: bool,
    pub pasta_tmp: std::path::PathBuf,
    pub limite_toque_ms: u64,
}

/// `mostrar(estado)` atualiza o painel; `falou()` marca a descoberta.
pub fn rodar(m: Motor, entradas: Receiver<Entrada>, inicio: Instant, mostrar: impl Fn(EstadoVoz), falou: impl Fn()) {
    let mut gesto = Gesto::novo(m.limite_toque_ms);
    let mut gravador: Option<Gravador> = None;
    let ms = || inicio.elapsed().as_millis() as u64;
    loop {
        let e = entradas.recv_timeout(Duration::from_millis(50)).unwrap_or(Entrada::Tique(ms()));
        let mut acao = gesto.passo(e);
        if let (Some(g), true) = (&gravador, gesto.maos_livres()) {
            let a = g.ate_agora();
            if a.len() > TAXA as usize * 3 / 2 && silencio_no_fim(&a, TAXA, 1200, 0.02) {
                acao = gesto.passo(Entrada::Clique(ms()));
            }
        }
        if let Some(g) = &gravador {
            let fase = if gesto.maos_livres() { "maos-livres" } else { "segurando" };
            mostrar(EstadoVoz { fase, nivel: g.nivel(), mensagem: None });
        }
        match acao {
            Acao::Nada => {}
            Acao::OuvirSegurando | Acao::AbrirEOuvir => {
                if !m.cpu_ok {
                    mostrar(EstadoVoz { fase: "erro", nivel: 0.0, mensagem: Some(ErroVoz::SemRecurso.texto().into()) });
                    let _ = gesto.passo(Entrada::Cancelar);
                    continue;
                }
                match Gravador::iniciar() {
                    Ok(g) => gravador = Some(g),
                    Err(_) => {
                        mostrar(EstadoVoz { fase: "erro", nivel: 0.0, mensagem: Some(ErroVoz::SemMicrofone.texto().into()) });
                        let _ = gesto.passo(Entrada::Cancelar);
                    }
                }
            }
            Acao::Cancelar => {
                if let Some(g) = gravador.take() {
                    let _ = g.parar();
                }
                mostrar(EstadoVoz { fase: "oculto", nivel: 0.0, mensagem: None });
            }
            Acao::Escrever => {
                let Some(g) = gravador.take() else { continue };
                mostrar(EstadoVoz { fase: "escrevendo", nivel: 0.0, mensagem: None });
                let amostras = g.parar();
                let r = transcrever(&m.whisper, &amostras, &m.vocabulario.prompt, &m.pasta_tmp).map(|t| aplicar(&m.vocabulario, &t));
                let fim = match r {
                    Ok(texto) if m.qualquer_campo => match escrever_no_cursor(&SistemaReal, &texto) {
                        Ok(()) => {
                            falou();
                            None
                        }
                        Err(e) => Some(e.texto().to_string()),
                    },
                    Ok(texto) => {
                        let _ = copiar_texto(&texto);
                        falou();
                        Some("O texto está copiado. Cole com Ctrl+V.".to_string())
                    }
                    Err(e) => Some(e.texto().to_string()),
                };
                match fim {
                    None => mostrar(EstadoVoz { fase: "oculto", nivel: 0.0, mensagem: None }),
                    Some(msg) => {
                        mostrar(EstadoVoz { fase: "erro", nivel: 0.0, mensagem: Some(msg) });
                        std::thread::sleep(Duration::from_secs(4));
                        mostrar(EstadoVoz { fase: "oculto", nivel: 0.0, mensagem: None });
                    }
                }
            }
        }
    }
}

fn copiar_texto(texto: &str) -> Result<Vec<u8>, String> {
    use novasce_voz::escrever::Sistema;
    SistemaReal.rodar(&["wl-copy", "--type", "text/plain;charset=utf-8"], Some(texto.as_bytes()))
}
