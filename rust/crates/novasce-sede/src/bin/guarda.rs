//! /usr/libexec/novasce/novasce-sede-guarda: a TV da sede que fica só na tela de login volta à Sede.
//! Sem o `Relogin=true` (que quebrava o Trocar usuário), a Sede que saiu ou caiu não entra de novo
//! sozinha. A guarda confere a cada 30 s; com 2 min seguidos só no login e nenhuma conta pessoal
//! aberta, ela devolve a Sede: ativa a sessão da Sede que ficou por trás (decisão de 27/10) ou,
//! sem ela, reinicia a tela de login, e a entrada automática da Sede vale de novo.
#[cfg(target_os = "linux")]
const INTERVALO: std::time::Duration = std::time::Duration::from_secs(30);
#[cfg(target_os = "linux")]
const SEGUIDAS: u32 = 4;
#[cfg(target_os = "linux")]
const DEPOIS_DE_REINICIAR: std::time::Duration = std::time::Duration::from_secs(300);

fn main() {
    #[cfg(target_os = "linux")]
    {
        use novasce_casa::config_casa::ConfigCasa;
        use novasce_casa::sistema::{papel, Papel};
        use novasce_sede::sessoes::{ativar, listar, o_que_a_tv_mostra, Guarda, Tela};
        if papel() != Papel::Sede {
            return;
        }
        let conta_sede = ConfigCasa::carregar().sede.conta;
        let mut guarda = Guarda::nova(SEGUIDAS);
        loop {
            std::thread::sleep(INTERVALO);
            let tela = listar()
                .map(|l| o_que_a_tv_mostra(&l, &conta_sede))
                .unwrap_or(Tela::Ocupada);
            if !guarda.conferir(tela != Tela::Ocupada) {
                continue;
            }
            match tela {
                Tela::SedePorTras(id) => {
                    eprintln!("A TV ficou 2 minutos no login com a Sede aberta por trás. A Sede volta para a frente.");
                    if let Err(e) = ativar(&id) {
                        eprintln!("Não deu para ativar a Sede: {e}");
                    }
                }
                Tela::SoLogin => {
                    eprintln!("A TV ficou 2 minutos só no login. A tela de login reinicia para a Sede entrar.");
                    let _ = std::process::Command::new("systemctl")
                        .args(["restart", "display-manager.service"])
                        .status();
                    std::thread::sleep(DEPOIS_DE_REINICIAR);
                }
                Tela::Ocupada => {}
            }
        }
    }
}
