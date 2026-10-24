//! novasce-sinal: o serviço da máquina. Manda o sinal do dia ao Tenancy, registra
//! a máquina pelo terminal e responde as URLs da casa para os atalhos e a Sala.
use clap::{Parser, Subcommand};
use novasce_core::config::Config;
use novasce_core::contrato::{Papel, PedidoDeRegistro, Sinal};
use novasce_core::registro::{self, formatar_codigo, ErroDoRegistro, Espera, FimDoRegistro};
use novasce_core::sistema::{
    agentes_ociosos, disco_cifrado, ler_papel, ler_variante, nome_da_maquina, secure_boot_ligado,
    versao_do_sistema, versao_preparada, CAMINHO_PAPEL, CAMINHO_VARIANTE, TRAVA_DOS_AGENTES,
};
use novasce_core::tenancy::{responde, ClienteTenancy, ErroTenancy};
use novasce_core::token::GuardaDoToken;
use std::path::Path;
use std::process::{Command, ExitCode};
use std::time::Duration;

/// Sinal da máquina Novasce para o Tenancy.
#[derive(Parser)]
#[command(name = "novasce-sinal", version)]
struct Cli {
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Subcommand)]
enum Comando {
    /// Envia o sinal do dia: versão, atualização pronta, disco, Secure Boot e agentes.
    Heartbeat,
    /// Registra esta máquina no Tenancy pelo terminal (precisa de sudo).
    Pair,
    /// Imprime uma URL da casa: tenancy, material, sala, claude, chatgpt, gemini ou whatsapp.
    Url { nome: String },
    /// Sai com 0 se a Sala responde e com 1 se não.
    SalaPronta,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let config = match Config::carregar() {
        Ok(config) => config,
        Err(erro) => {
            eprintln!("{erro}");
            return ExitCode::from(2);
        }
    };
    match cli.comando {
        Comando::Url { nome } => match config.url(&nome) {
            Some(url) => {
                println!("{url}");
                ExitCode::SUCCESS
            }
            None => {
                eprintln!("Não conheço a URL {nome}.");
                ExitCode::from(2)
            }
        },
        Comando::SalaPronta => {
            if responde(&config.sala_url, Duration::from_secs(5)) {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Comando::Heartbeat => heartbeat(&config),
        Comando::Pair => pair(&config),
    }
}

fn heartbeat(config: &Config) -> ExitCode {
    let maquina = match GuardaDoToken::padrao().ler() {
        Ok(Some(maquina)) => maquina,
        Ok(None) => {
            println!("Máquina sem registro no Tenancy. Nada a enviar.");
            return ExitCode::SUCCESS;
        }
        Err(erro) => {
            eprintln!("{erro}");
            return ExitCode::from(1);
        }
    };
    let sinal = coletar_sinal();
    match ClienteTenancy::novo(&config.tenancy_api).enviar_sinal(&maquina.machine_token, &sinal) {
        Ok(()) => {
            println!(
                "Sinal enviado: versão {}, preparada {}, disco cifrado {}, Secure Boot {}.",
                sinal.os_version,
                sinal.staged_version.as_deref().unwrap_or("nenhuma"),
                sinal.disk_encrypted,
                sinal.secure_boot
            );
            ExitCode::SUCCESS
        }
        Err(ErroTenancy::SemModulo) => {
            println!("O Tenancy ainda não recebe o sinal. Fica para o próximo.");
            ExitCode::SUCCESS
        }
        Err(ErroTenancy::NaoAutorizado) => {
            eprintln!("O token desta máquina não vale mais. Registre de novo com sudo novasce-sinal pair.");
            ExitCode::from(1)
        }
        Err(erro) => {
            eprintln!("{erro}");
            ExitCode::from(1)
        }
    }
}

fn coletar_sinal() -> Sinal {
    let preparada = saida("/usr/bin/rpm-ostree", &["status", "--json"])
        .and_then(|json| versao_preparada(&json).ok().flatten());
    let cifrado = saida("/usr/bin/lsblk", &["--json", "--tree", "-o", "NAME,TYPE,MOUNTPOINTS"])
        .and_then(|json| disco_cifrado(&json).ok())
        .unwrap_or(false);
    let secure_boot = saida("/usr/bin/mokutil", &["--sb-state"])
        .map(|texto| secure_boot_ligado(&texto))
        .unwrap_or(false);
    let na_sede = ler_papel(Path::new(CAMINHO_PAPEL)) == Some(Papel::Sede);
    Sinal {
        os_version: versao_do_sistema(),
        staged_version: preparada,
        disk_encrypted: cifrado,
        secure_boot,
        agents_idle: na_sede.then(|| agentes_ociosos(Path::new(TRAVA_DOS_AGENTES))),
    }
}

fn saida(programa: &str, args: &[&str]) -> Option<String> {
    let resultado = Command::new(programa).args(args).output().ok()?;
    resultado
        .status
        .success()
        .then(|| String::from_utf8_lossy(&resultado.stdout).into_owned())
}

fn pair(config: &Config) -> ExitCode {
    // SAFETY: geteuid não tem efeito colateral.
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("Rode com sudo: sudo novasce-sinal pair");
        return ExitCode::from(1);
    }
    let Some(papel) = ler_papel(Path::new(CAMINHO_PAPEL)) else {
        eprintln!("Defina o papel da máquina antes: abra Preparar esta máquina.");
        return ExitCode::from(1);
    };
    let pedido = PedidoDeRegistro {
        role: papel,
        variant: ler_variante(Path::new(CAMINHO_VARIANTE)),
        hostname: nome_da_maquina(),
        os_version: versao_do_sistema(),
    };
    let fim = registro::registrar(
        &ClienteTenancy::novo(&config.tenancy_api),
        &GuardaDoToken::padrao(),
        &pedido,
        &Espera::default(),
        &std::thread::sleep,
        &mut |r| {
            println!(
                "Código: {}. No Tenancy, abra Máquinas e telas e aprove. Vale 10 minutos.",
                formatar_codigo(&r.pairing_code)
            )
        },
    );
    match fim {
        Ok(FimDoRegistro::Aprovado { .. }) => {
            println!("Aprovada. Para mandar o primeiro sinal agora: sudo systemctl start novasce-sinal");
            ExitCode::SUCCESS
        }
        Ok(FimDoRegistro::Negado) => {
            eprintln!("O Tenancy recusou este código. Rode de novo para gerar outro.");
            ExitCode::from(1)
        }
        Ok(FimDoRegistro::Expirado) => {
            eprintln!("O código venceu ou já foi usado. Rode de novo para gerar outro.");
            ExitCode::from(1)
        }
        Err(ErroDoRegistro::Tenancy(ErroTenancy::SemModulo)) => {
            eprintln!("O Tenancy ainda não recebe máquinas.");
            ExitCode::from(1)
        }
        Err(erro) => {
            eprintln!("{erro}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn comandos_conhecidos() {
        assert!(Cli::try_parse_from(["novasce-sinal", "heartbeat"]).is_ok());
        assert!(Cli::try_parse_from(["novasce-sinal", "pair"]).is_ok());
        assert!(Cli::try_parse_from(["novasce-sinal", "sala-pronta"]).is_ok());
        assert!(matches!(
            Cli::try_parse_from(["novasce-sinal", "url", "sala"]).unwrap().comando,
            Comando::Url { nome } if nome == "sala"
        ));
        assert!(Cli::try_parse_from(["novasce-sinal", "apagar"]).is_err());
    }
}
