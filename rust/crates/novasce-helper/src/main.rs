//! novasce-helper: roda como root pelo pkexec (política com.novasce.helper, só
//! para wheel). Cada evento sai numa linha JSON no stdout, para o Preparar ler.
use clap::{Parser, Subcommand, ValueEnum};
use novasce_core::config::Config;
use novasce_core::contrato::{Papel, PedidoDeRegistro};
use novasce_core::registro::{self, ErroDoRegistro, Espera, FimDoRegistro};
use novasce_core::sistema::{
    ler_papel, ler_variante, nome_da_maquina, versao_do_sistema, CAMINHO_PAPEL, CAMINHO_VARIANTE,
};
use novasce_core::tenancy::{ClienteTenancy, ErroTenancy};
use novasce_core::token::GuardaDoToken;
use novasce_helper::acoes::{self, Ambiente, ExecutorReal};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Ajudante do Novasce OS que roda como administrador (pelo pkexec).
#[derive(Parser)]
#[command(name = "novasce-helper", version)]
struct Cli {
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Clone, Copy, ValueEnum)]
enum PapelArg {
    Sede,
    Notebook,
}

#[derive(Subcommand)]
enum Comando {
    /// Grava o papel e o nome da máquina, e liga o que é só da sede.
    Papel { papel: PapelArg },
    /// Registra a máquina no Tenancy e espera a aprovação.
    Registrar,
    /// Lista o time do Tenancy para escolher o dono (só nas primeiras 24 h).
    Time,
    /// Cria a conta pessoal. Com --dono, também grava o dono do notebook no Tenancy.
    CriarConta {
        #[arg(long)]
        nome: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        dono: Option<String>,
    },
    /// Cria a conta Sede: entrada automática, sem carteira, sem terminal e sem sono.
    PrepararSede,
    /// Marca a máquina como preparada.
    Concluir,
}

fn emitir(valor: Value) {
    println!("{valor}");
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    // SAFETY: geteuid não tem efeito colateral.
    if unsafe { libc::geteuid() } != 0 {
        emitir(
            json!({"evento": "erro", "mensagem": "O ajudante precisa de administrador. Abra pelo Preparar esta máquina."}),
        );
        return ExitCode::from(1);
    }
    match executar(cli.comando) {
        Ok(()) => ExitCode::SUCCESS,
        Err(mensagem) => {
            emitir(json!({"evento": "erro", "mensagem": mensagem}));
            ExitCode::from(1)
        }
    }
}

fn executar(comando: Comando) -> Result<(), String> {
    let executor = ExecutorReal;
    let amb = Ambiente {
        raiz: PathBuf::from("/"),
        executor: &executor,
    };
    match comando {
        Comando::Papel { papel } => {
            let papel = match papel {
                PapelArg::Sede => Papel::Sede,
                PapelArg::Notebook => Papel::Notebook,
            };
            acoes::definir_papel(&amb, papel).map_err(|e| e.to_string())?;
            emitir(json!({"evento": "papel", "papel": papel.como_texto()}));
        }
        Comando::Registrar => registrar()?,
        Comando::Time => {
            let token = token_da_maquina()?;
            let pessoas = cliente()?.time(&token).map_err(texto_do_tenancy)?;
            emitir(json!({"evento": "time", "pessoas": pessoas}));
        }
        Comando::CriarConta { nome, email, dono } => {
            acoes::conferir_conta_nova(&amb, &nome, &email).map_err(|e| e.to_string())?;
            if let Some(id) = dono.as_deref() {
                let token = token_da_maquina()?;
                cliente()?
                    .definir_dono(&token, id)
                    .map_err(texto_do_tenancy)?;
            }
            let login =
                acoes::criar_conta_pessoal(&amb, &nome, &email).map_err(|e| e.to_string())?;
            emitir(json!({"evento": "conta", "login": login}));
        }
        Comando::PrepararSede => {
            acoes::preparar_sede(&amb).map_err(|e| e.to_string())?;
            emitir(json!({"evento": "sede"}));
        }
        Comando::Concluir => {
            acoes::concluir(&amb).map_err(|e| e.to_string())?;
            emitir(json!({"evento": "concluido"}));
        }
    }
    Ok(())
}

fn cliente() -> Result<ClienteTenancy, String> {
    let config = Config::carregar().map_err(|e| e.to_string())?;
    Ok(ClienteTenancy::novo(&config.tenancy_api))
}

fn token_da_maquina() -> Result<String, String> {
    match GuardaDoToken::padrao().ler() {
        Ok(Some(maquina)) => Ok(maquina.machine_token),
        Ok(None) => Err("Esta máquina ainda não foi registrada no Tenancy.".to_string()),
        Err(erro) => Err(erro.to_string()),
    }
}

fn texto_do_tenancy(erro: ErroTenancy) -> String {
    match erro {
        ErroTenancy::Proibido(mensagem) => format!(
            "O Tenancy não deixou: {mensagem}. A lista do time só abre nas primeiras 24 horas depois do registro, e só enquanto a máquina não tem dono."
        ),
        outro => outro.to_string(),
    }
}

fn registrar() -> Result<(), String> {
    let papel = ler_papel(Path::new(CAMINHO_PAPEL))
        .ok_or("Defina o papel da máquina antes de registrar.")?;
    let pedido = PedidoDeRegistro {
        role: papel,
        variant: ler_variante(Path::new(CAMINHO_VARIANTE)),
        hostname: nome_da_maquina(),
        os_version: versao_do_sistema(),
    };
    let fim = registro::registrar(
        &cliente()?,
        &GuardaDoToken::padrao(),
        &pedido,
        &Espera::default(),
        &std::thread::sleep,
        &mut |r| {
            emitir(json!({"evento": "codigo", "codigo": r.pairing_code, "expiraEm": r.expires_at}))
        },
    );
    match fim {
        Ok(FimDoRegistro::Aprovado { machine_id }) => {
            emitir(json!({"evento": "aprovado", "maquina": machine_id}))
        }
        Ok(FimDoRegistro::Negado) => emitir(json!({"evento": "negado"})),
        Ok(FimDoRegistro::Expirado) => emitir(json!({"evento": "expirado"})),
        Err(ErroDoRegistro::Tenancy(ErroTenancy::SemModulo)) => {
            emitir(json!({"evento": "sem-modulo"}))
        }
        Err(erro) => return Err(erro.to_string()),
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn criar_conta_com_e_sem_dono() {
        let com = Cli::try_parse_from([
            "novasce-helper",
            "criar-conta",
            "--nome",
            "Luan Silva",
            "--email",
            "luan@novasce.com",
            "--dono",
            "u1",
        ])
        .unwrap();
        assert!(matches!(com.comando, Comando::CriarConta { dono: Some(ref d), .. } if d == "u1"));
        let sem = Cli::try_parse_from([
            "novasce-helper",
            "criar-conta",
            "--nome",
            "Luan",
            "--email",
            "luan@novasce.com",
        ])
        .unwrap();
        assert!(matches!(
            sem.comando,
            Comando::CriarConta { dono: None, .. }
        ));
    }

    #[test]
    fn papel_so_aceita_sede_ou_notebook() {
        assert!(Cli::try_parse_from(["novasce-helper", "papel", "sede"]).is_ok());
        assert!(Cli::try_parse_from(["novasce-helper", "papel", "notebook"]).is_ok());
        assert!(Cli::try_parse_from(["novasce-helper", "papel", "servidor"]).is_err());
    }

    #[test]
    fn os_outros_comandos_existem() {
        for comando in ["registrar", "time", "preparar-sede", "concluir"] {
            assert!(
                Cli::try_parse_from(["novasce-helper", comando]).is_ok(),
                "{comando}"
            );
        }
    }
}
