//! Microfone pelo PipeWire (parecord, via pipewire-pulse), 16 kHz mono s16le.
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

pub const TAXA: u32 = 16_000;

pub fn nivel_rms(a: &[i16]) -> f32 {
    if a.is_empty() {
        return 0.0;
    }
    let soma: f64 = a.iter().map(|&x| (x as f64) * (x as f64)).sum();
    (((soma / a.len() as f64).sqrt() / 32768.0) * 4.0).min(1.0) as f32
}

pub fn silencio_no_fim(a: &[i16], taxa: u32, ms: u32, limiar: f32) -> bool {
    let n = (taxa as usize * ms as usize) / 1000;
    a.len() >= n && nivel_rms(&a[a.len() - n..]) < limiar
}

pub fn amostras_de_bytes(b: &[u8]) -> Vec<i16> {
    b.as_chunks::<2>()
        .0
        .iter()
        .map(|c| i16::from_le_bytes(*c))
        .collect()
}

pub fn wav(a: &[i16], taxa: u32) -> Vec<u8> {
    let dados = (a.len() * 2) as u32;
    let mut w = Vec::with_capacity(44 + dados as usize);
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + dados).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&taxa.to_le_bytes());
    w.extend_from_slice(&(taxa * 2).to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&dados.to_le_bytes());
    for x in a {
        w.extend_from_slice(&x.to_le_bytes());
    }
    w
}

pub struct Gravador {
    filho: Child,
    amostras: Arc<Mutex<Vec<i16>>>,
    nivel: Arc<AtomicU32>,
    leitor: Option<std::thread::JoinHandle<()>>,
}

impl Gravador {
    pub fn iniciar() -> std::io::Result<Gravador> {
        let mut filho = Command::new("parecord")
            .args([
                "--raw",
                "--format=s16le",
                "--rate=16000",
                "--channels=1",
                "--latency-msec=50",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut saida = filho
            .stdout
            .take()
            .ok_or_else(|| std::io::Error::other("sem saída do parecord"))?;
        let amostras = Arc::new(Mutex::new(Vec::new()));
        let nivel = Arc::new(AtomicU32::new(0));
        let (a, n) = (amostras.clone(), nivel.clone());
        let leitor = std::thread::spawn(move || {
            let mut buf = [0u8; 1600];
            while let Ok(lidos) = saida.read(&mut buf) {
                if lidos == 0 {
                    break;
                }
                let pedaco = amostras_de_bytes(&buf[..lidos - lidos % 2]);
                n.store(nivel_rms(&pedaco).to_bits(), Ordering::Relaxed);
                a.lock().unwrap().extend(pedaco);
            }
        });
        Ok(Gravador {
            filho,
            amostras,
            nivel,
            leitor: Some(leitor),
        })
    }

    pub fn nivel(&self) -> f32 {
        f32::from_bits(self.nivel.load(Ordering::Relaxed))
    }

    pub fn ate_agora(&self) -> Vec<i16> {
        self.amostras.lock().unwrap().clone()
    }

    pub fn parar(mut self) -> Vec<i16> {
        let _ = self.filho.kill();
        let _ = self.filho.wait();
        if let Some(l) = self.leitor.take() {
            let _ = l.join();
        }
        std::mem::take(&mut *self.amostras.lock().unwrap())
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn nivel_e_silencio() {
        assert_eq!(nivel_rms(&[]), 0.0);
        assert!(nivel_rms(&[8000; 800]) > 0.5);
        let mut a = vec![8000i16; 16_000];
        a.extend(vec![0i16; 16_000]);
        assert!(silencio_no_fim(&a, 16_000, 1000, 0.02));
        assert!(!silencio_no_fim(&vec![8000i16; 32_000], 16_000, 1200, 0.02));
    }

    #[test]
    fn wav_tem_cabecalho_certo() {
        let w = wav(&[1, -1], 16_000);
        assert_eq!(&w[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(w[4..8].try_into().unwrap()), 36 + 4);
        assert_eq!(&w[8..16], b"WAVEfmt ");
        assert_eq!(u32::from_le_bytes(w[24..28].try_into().unwrap()), 16_000);
        assert_eq!(&w[36..40], b"data");
        assert_eq!(w.len(), 44 + 4);
        assert_eq!(amostras_de_bytes(&w[44..]), vec![1, -1]);
    }
}
