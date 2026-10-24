//! /usr/libexec/novasce/novasce-login-qr (root): gera o fundo do login com o QR desta máquina.
use novasce_casa::nucleo::{ler_maquina, Config, Maquina, Tenancy, CAMINHO_MAQUINA};
use novasce_chamados::esqueci::{textos, url_esqueci};
use novasce_chamados::login::{conf_greeter, svg_fundo};

const PAPEL: &str = "/usr/share/wallpapers/Novasce/contents/images/3840x2160.png";
// Fora de /var/lib/novasce (0700, só root): o greeter roda sem ser root e precisa ler o fundo.
const PASTA: &str = "/var/lib/novasce-login";
const CONF: &str = "/etc/plasmalogin.conf.d/90-novasce-qr.conf";

fn main() {
    let Ok(token) = ler_maquina(std::path::Path::new(CAMINHO_MAQUINA)) else {
        eprintln!("máquina ainda não registrada; o login fica sem QR");
        let _ = std::fs::remove_file(CONF);
        return;
    };
    let config = Config::carregar().expect("config.toml da máquina");
    let maquina: Option<Maquina> = Tenancy::new(&config.tenancy_api, Some(token.machine_token))
        .get("/os/machine")
        .ok();
    let nome = maquina
        .as_ref()
        .map(|m| m.name.clone())
        .unwrap_or_else(|| "computador".into());
    let dono = maquina
        .as_ref()
        .and_then(|m| m.owner.as_ref())
        .map(|o| o.first_name.clone());
    let (titulo, linha) = textos(&nome, dono.as_deref());
    let svg = svg_fundo(
        PAPEL,
        &url_esqueci(&config.tenancy_web, &token.machine_id),
        &titulo,
        &linha,
        3840,
        2160,
    )
    .expect("QR");
    std::fs::create_dir_all(PASTA).expect("pasta do login");
    let arquivo_svg = format!("{PASTA}/fundo.svg");
    let arquivo_png = format!("{PASTA}/fundo.png");
    std::fs::write(&arquivo_svg, svg).expect("svg");
    let ok = std::process::Command::new("rsvg-convert")
        .args(["-o", &arquivo_png, &arquivo_svg])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        eprintln!("rsvg-convert falhou; o login fica com o papel de parede da casa");
        return;
    }
    std::fs::create_dir_all("/etc/plasmalogin.conf.d").expect("conf.d");
    std::fs::write(CONF, conf_greeter(&arquivo_png)).expect("conf do greeter");
}
