//! novasce-voz-medir --arquivo frase.wav [--modelo …] [--vezes 5]
//! Mede o tempo de soltar a tecla até o texto, com o vocabulário da casa. Usado na Tarefa 35.
use novasce_voz::gravacao::amostras_de_bytes;
use novasce_voz::vocabulario::{aplicar, carregar};
use novasce_voz::whisper::{percentis, threads_padrao, transcrever, Whisper};
use std::time::Instant;

fn arg(nome: &str, padrao: &str) -> String {
    let a: Vec<String> = std::env::args().collect();
    a.iter()
        .position(|x| x == nome)
        .and_then(|i| a.get(i + 1).cloned())
        .unwrap_or_else(|| padrao.into())
}

fn main() {
    let arquivo = arg("--arquivo", "frase.wav");
    let modelo = arg("--modelo", "/usr/share/novasce/voz/ggml-small-q5_1.bin");
    let vezes: usize = arg("--vezes", "5").parse().unwrap_or(5);
    let bytes = std::fs::read(&arquivo).expect("arquivo WAV 16 kHz mono s16le");
    let amostras = amostras_de_bytes(&bytes[44..]);
    let voc = carregar(
        "/usr/share/novasce/voz/vocabulario.toml",
        std::path::Path::new(""),
    );
    let w = Whisper {
        binario: "/usr/libexec/novasce/whisper-cli".into(),
        modelo: modelo.clone().into(),
        threads: threads_padrao(),
    };
    let pasta = std::env::temp_dir();
    let mut tempos = Vec::new();
    for i in 0..vezes {
        let t = Instant::now();
        let texto = transcrever(&w, &amostras, &voc.prompt, &pasta).map(|t| aplicar(&voc, &t));
        let ms = t.elapsed().as_millis();
        println!("rodada {}: {} ms · {:?}", i + 1, ms, texto);
        tempos.push(ms);
    }
    let segundos = amostras.len() as f64 / 16_000.0;
    let (p50, p95) = percentis(&mut tempos);
    println!(
        "modelo {modelo} · fala de {segundos:.1} s · {} threads · p50 {p50} ms · p95 {p95} ms",
        w.threads
    );
}
