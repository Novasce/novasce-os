use clap::Parser;

/// Sinal da máquina Novasce para o Tenancy.
#[derive(Parser)]
#[command(name = "novasce-sinal", version)]
struct Cli {}

fn main() {
    let _ = Cli::parse();
}
