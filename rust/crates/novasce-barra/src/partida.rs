//! Partida do serviço: a barra só sobe com a config da máquina.
//!
//! Máquina que ainda não foi preparada (ou config com erro de digitação) não é motivo para o serviço
//! cair: a unidade tem `Restart=on-failure` e `RestartSec=10`, então cair e voltar encheria o journal a
//! cada 10 s. A barra avisa uma vez, espera com calma e sobe sozinha quando a config passa a valer.
use std::fmt::Display;
use std::time::Duration;

/// Entre uma tentativa e outra. É só ler um arquivo pequeno, mas ninguém precisa disso a cada segundo.
pub const INTERVALO_SEM_CONFIG: Duration = Duration::from_secs(60);

/// Repete `carregar` até dar certo. `registrar` recebe uma linha de log por motivo novo (o mesmo erro
/// repetido não vira mil linhas) e outra quando a config finalmente vale.
pub fn esperar_config<T, E: Display>(
    mut carregar: impl FnMut() -> Result<T, E>,
    mut dormir: impl FnMut(Duration),
    mut registrar: impl FnMut(&str),
) -> T {
    let mut motivo = String::new();
    loop {
        match carregar() {
            Ok(valor) => {
                if !motivo.is_empty() {
                    registrar("a config da máquina passou a valer, a barra sobe");
                }
                return valor;
            }
            Err(erro) => {
                let texto = erro.to_string();
                if texto != motivo {
                    registrar(&format!(
                        "sem config da máquina ({texto}); espero e tento de novo a cada {} s",
                        INTERVALO_SEM_CONFIG.as_secs()
                    ));
                    motivo = texto;
                }
                dormir(INTERVALO_SEM_CONFIG);
            }
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn config_que_ja_vale_sobe_sem_dormir_nem_avisar() {
        let mut dormiu = 0;
        let mut linhas: Vec<String> = vec![];
        let v = esperar_config(
            || Ok::<_, String>(7),
            |_| dormiu += 1,
            |l| linhas.push(l.into()),
        );
        assert_eq!((v, dormiu, linhas.len()), (7, 0, 0));
    }

    #[test]
    fn espera_em_silencio_avisa_uma_vez_e_sobe_quando_a_config_chega() {
        let mut tentativas = 0;
        let mut pausas: Vec<Duration> = vec![];
        let mut linhas: Vec<String> = vec![];
        let v = esperar_config(
            || {
                tentativas += 1;
                if tentativas < 5 {
                    Err("não deu para ler /usr/share/novasce/config.toml".to_string())
                } else {
                    Ok("pronta")
                }
            },
            |p| pausas.push(p),
            |l| linhas.push(l.into()),
        );
        assert_eq!(v, "pronta");
        assert_eq!(pausas.len(), 4);
        assert!(
            pausas.iter().all(|p| *p >= Duration::from_secs(30)),
            "sem laço apertado"
        );
        assert_eq!(
            linhas.len(),
            2,
            "um aviso e o aviso de que subiu: {linhas:?}"
        );
        assert!(linhas[0].contains("sem config da máquina") && linhas[0].contains("config.toml"));
        assert!(linhas[1].contains("passou a valer"));
    }

    #[test]
    fn motivo_novo_ganha_linha_nova() {
        let erros = [
            "falta o arquivo",
            "falta o arquivo",
            "chave desconhecida",
            "chave desconhecida",
        ];
        let mut i = 0;
        let mut linhas: Vec<String> = vec![];
        esperar_config(
            || {
                i += 1;
                erros
                    .get(i - 1)
                    .map(|e| Err(e.to_string()))
                    .unwrap_or(Ok(()))
            },
            |_| {},
            |l| linhas.push(l.into()),
        );
        assert_eq!(linhas.len(), 3);
        assert!(linhas[0].contains("falta o arquivo") && linhas[1].contains("chave desconhecida"));
    }
}
