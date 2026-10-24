//! Laço do serviço: aceita conexões no socket, uma por fio, uma linha de pedido e uma de resposta.
//! Só roda no Linux (SO_PEERCRED e ativação por socket do systemd). No Mac o binário só avisa;
//! a lógica de verdade está na lib e é testada em qualquer sistema.

#[cfg(target_os = "linux")]
mod servico {
    use nix::sys::socket::{getsockopt, sockopt::PeerCredentials};
    use nix::unistd::{Uid, User};
    use novasce_casa::nucleo::{Config, CAMINHO_MAQUINA};
    use novasce_sinal_local::{responder, Contexto};
    use std::io::{BufRead, BufReader, Read, Write};
    use std::os::fd::FromRawFd;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::sync::Arc;

    fn ouvinte() -> std::io::Result<UnixListener> {
        let ativado = std::env::var("LISTEN_FDS").ok().as_deref() == Some("1")
            && std::env::var("LISTEN_PID").ok() == Some(std::process::id().to_string());
        if ativado {
            // SAFETY: o systemd entrega o socket no descritor 3 quando LISTEN_FDS=1 e LISTEN_PID é este processo.
            return Ok(unsafe { UnixListener::from_raw_fd(3) });
        }
        let caminho = novasce_casa::sinal_local::SOCKET;
        std::fs::create_dir_all("/run/novasce")?;
        let _ = std::fs::remove_file(caminho);
        let l = UnixListener::bind(caminho)?;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(caminho, std::fs::Permissions::from_mode(0o666))?;
        Ok(l)
    }

    fn tratar(ctx: &Contexto, s: UnixStream) {
        let Ok(cred) = getsockopt(&s, PeerCredentials) else {
            return;
        };
        let uid = cred.uid();
        let conta = User::from_uid(Uid::from_raw(uid))
            .ok()
            .flatten()
            .map(|u| u.name)
            .unwrap_or_else(|| uid.to_string());
        let mut linha = String::new();
        if BufReader::new((&s).take(8 * 1024 * 1024))
            .read_line(&mut linha)
            .is_err()
        {
            return;
        }
        let saida = responder(ctx, uid, &conta, &linha);
        let _ = (&s).write_all(saida.as_bytes());
    }

    pub fn rodar() {
        let config = Config::carregar().expect("config.toml da máquina");
        let ctx = Arc::new(Contexto::novo(
            config.tenancy_api.clone(),
            CAMINHO_MAQUINA.into(),
        ));
        let l = ouvinte().expect("socket do sinal local");
        for conexao in l.incoming().flatten() {
            let ctx = ctx.clone();
            std::thread::spawn(move || tratar(&ctx, conexao));
        }
    }
}

#[cfg(target_os = "linux")]
fn main() {
    servico::rodar();
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("novasce-sinal-local só roda no Linux (na sede).");
    std::process::exit(1);
}
