//! /usr/libexec/novasce/novasce-sede-guarda: a TV da sede que fica só na tela de login volta à Sede.
//! Sem o `Relogin=true` (que quebrava o Trocar usuário), a Sede que saiu ou caiu não entra de novo
//! sozinha. A guarda confere a cada 30 s; com 2 min seguidos só no login e ninguém dentro, reinicia
//! a tela de login, e a entrada automática da Sede vale de novo. Conta pessoal aberta segura tudo.
#[cfg(target_os = "linux")]
const INTERVALO: std::time::Duration = std::time::Duration::from_secs(30);
#[cfg(target_os = "linux")]
const SEGUIDAS: u32 = 4;
#[cfg(target_os = "linux")]
const DEPOIS_DE_REINICIAR: std::time::Duration = std::time::Duration::from_secs(300);

fn main() {
    #[cfg(target_os = "linux")]
    {
        use novasce_casa::sistema::{papel, Papel};
        use novasce_sede::sessoes::{listar, so_a_tela_de_login, Guarda};
        if papel() != Papel::Sede {
            return;
        }
        let mut guarda = Guarda::nova(SEGUIDAS);
        loop {
            std::thread::sleep(INTERVALO);
            let so_login = listar().is_ok_and(|l| so_a_tela_de_login(&l));
            if guarda.conferir(so_login) {
                eprintln!("A TV ficou 2 minutos só no login. A tela de login reinicia para a Sede entrar.");
                let _ = std::process::Command::new("systemctl")
                    .args(["restart", "display-manager.service"])
                    .status();
                std::thread::sleep(DEPOIS_DE_REINICIAR);
            }
        }
    }
}
