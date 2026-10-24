use clap::Parser;

/// Ajudante do Novasce OS que roda como administrador (pelo pkexec).
#[derive(Parser)]
#[command(name = "novasce-helper", version)]
struct Cli {}

fn main() {
    let _ = Cli::parse();
}
