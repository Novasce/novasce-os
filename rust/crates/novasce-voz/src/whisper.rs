//! whisper.cpp na CPU, uma chamada por frase. Sem servidor residente (a sede tem várias contas).
use crate::gravacao::{wav, TAXA};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Whisper {
    pub binario: PathBuf,
    pub modelo: PathBuf,
    pub threads: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErroVoz {
    SemMicrofone,
    NadaOuvido,
    SemRecurso,
    Falhou(String),
}

impl ErroVoz {
    pub fn texto(&self) -> &str {
        match self {
            ErroVoz::SemMicrofone => {
                "Nenhum microfone encontrado. Confira o som nas Configurações."
            }
            ErroVoz::NadaOuvido => "Não deu para ouvir. Fale mais perto do microfone.",
            ErroVoz::SemRecurso => {
                "Este computador não tem o recurso que a voz precisa. Peça ajuda."
            }
            ErroVoz::Falhou(_) => "A voz não escreveu nada desta vez. Tente de novo.",
        }
    }
}

pub fn threads_padrao() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(1, 8)
}

pub fn args(w: &Whisper, arquivo: &Path, prompt: &str) -> Vec<String> {
    vec![
        "-m".into(),
        w.modelo.to_string_lossy().into(),
        "-f".into(),
        arquivo.to_string_lossy().into(),
        "-l".into(),
        "pt".into(),
        "-t".into(),
        w.threads.to_string(),
        "-nt".into(),
        "-np".into(),
        "--prompt".into(),
        prompt.into(),
    ]
}

pub fn limpar_saida(s: &str) -> String {
    s.lines()
        .map(str::trim)
        .filter(|l| {
            !l.is_empty()
                && !(l.starts_with('[') && l.ends_with(']'))
                && !(l.starts_with('(') && l.ends_with(')'))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn cpu_compativel(cpuinfo: &str) -> bool {
    cpuinfo
        .lines()
        .any(|l| l.starts_with("flags") && l.split_whitespace().any(|f| f == "avx2"))
}

pub fn transcrever(
    w: &Whisper,
    amostras: &[i16],
    prompt: &str,
    pasta: &Path,
) -> Result<String, ErroVoz> {
    if amostras.len() < (TAXA as usize * 3) / 10 {
        return Err(ErroVoz::NadaOuvido);
    }
    std::fs::create_dir_all(pasta).map_err(|e| ErroVoz::Falhou(e.to_string()))?;
    let arquivo = pasta.join(format!("fala-{}.wav", std::process::id()));
    std::fs::write(&arquivo, wav(amostras, TAXA)).map_err(|e| ErroVoz::Falhou(e.to_string()))?;
    let s = Command::new(&w.binario)
        .args(args(w, &arquivo, prompt))
        .output();
    let _ = std::fs::remove_file(&arquivo);
    let s = s.map_err(|e| ErroVoz::Falhou(e.to_string()))?;
    if !s.status.success() {
        return Err(ErroVoz::Falhou(
            String::from_utf8_lossy(&s.stderr)
                .lines()
                .last()
                .unwrap_or("")
                .into(),
        ));
    }
    let texto = limpar_saida(&String::from_utf8_lossy(&s.stdout));
    if texto.is_empty() {
        Err(ErroVoz::NadaOuvido)
    } else {
        Ok(texto)
    }
}

/// (p50, p95) em ms.
pub fn percentis(ms: &mut [u128]) -> (u128, u128) {
    ms.sort_unstable();
    let p = |q: f64| ms[(((ms.len() - 1) as f64) * q).round() as usize];
    (p(0.5), p(0.95))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn argumentos_em_portugues_sem_marcas_de_tempo() {
        let w = Whisper {
            binario: "/usr/libexec/novasce/whisper-cli".into(),
            modelo: "/m.bin".into(),
            threads: 4,
        };
        let a = args(&w, Path::new("/tmp/f.wav"), "Novasce, Tenancy.");
        assert_eq!(
            a,
            vec![
                "-m",
                "/m.bin",
                "-f",
                "/tmp/f.wav",
                "-l",
                "pt",
                "-t",
                "4",
                "-nt",
                "-np",
                "--prompt",
                "Novasce, Tenancy."
            ]
        );
    }

    #[test]
    fn limpa_a_saida() {
        assert_eq!(
            limpar_saida(" Abri o Tenancy.\n [BLANK_AUDIO]\n Agora o SOPEC.\n"),
            "Abri o Tenancy. Agora o SOPEC."
        );
        assert_eq!(limpar_saida("[Música]\n"), "");
    }

    #[test]
    fn cpu_precisa_de_avx2() {
        assert!(cpu_compativel(
            "processor : 0\nflags : fpu sse avx avx2 fma\n"
        ));
        assert!(!cpu_compativel("flags : fpu sse avx\n"));
    }

    #[test]
    fn frase_curta_demais_nem_vai_ao_whisper() {
        let w = Whisper {
            binario: "/nao/existe".into(),
            modelo: "/m.bin".into(),
            threads: 1,
        };
        let d = tempfile::tempdir().unwrap();
        assert_eq!(
            transcrever(&w, &[0i16; 1000], "", d.path()).unwrap_err(),
            ErroVoz::NadaOuvido
        );
    }

    #[test]
    fn percentis_do_medidor() {
        let mut v = vec![900u128, 1000, 1100, 3000, 1200];
        assert_eq!(percentis(&mut v), (1100, 3000));
    }
}
