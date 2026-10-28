//! Conta pessoal na sede: 5 min sem uso, tranca e a TV volta à sessão Sede. Nenhum app fecha.
fn main() {
    #[cfg(target_os = "linux")]
    {
        use novasce_casa::config_casa::ConfigCasa;
        use novasce_casa::sistema::{conta_atual, papel, Papel};
        use novasce_sede::ociosidade::{observar, Evento};
        use novasce_sede::sessoes::{ativar, listar, sessao_da_sede, trancar};
        let casa = ConfigCasa::carregar();
        if papel() != Papel::Sede || conta_atual() == casa.sede.conta {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let ms = casa.sede.tranca_minutos * 60_000;
        std::thread::spawn(move || observar(ms, tx));
        for e in rx {
            if e == Evento::Ocioso {
                trancar();
                std::thread::sleep(std::time::Duration::from_secs(1));
                if let Some(s) = listar()
                    .ok()
                    .as_deref()
                    .and_then(|l| sessao_da_sede(l, &casa.sede.conta).cloned())
                {
                    let _ = ativar(&s.id);
                }
            }
        }
    }
}
