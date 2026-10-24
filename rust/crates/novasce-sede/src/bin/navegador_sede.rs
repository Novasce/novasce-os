//! /usr/bin/novasce-navegador-sede: o navegador da conta Sede, com perfil descartável.
use std::os::unix::process::CommandExt;

fn main() {
    let casa = novasce_casa::config_casa::ConfigCasa::carregar();
    let c = novasce_casa::caminhos::Caminhos::da_pessoa();
    let perfil = novasce_sede::navegador::perfil(&c.casa, &casa.sede);
    let _ = std::fs::create_dir_all(&perfil);
    let extra: Vec<String> = std::env::args().skip(1).collect();
    let a = novasce_sede::navegador::args_navegador(&casa.sede, &perfil, &extra);
    let erro = std::process::Command::new(&a[0]).args(&a[1..]).exec();
    eprintln!("não deu para abrir o navegador: {erro}");
}
