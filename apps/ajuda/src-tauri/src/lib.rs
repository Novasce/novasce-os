mod argumentos;
mod comandos;

use argumentos::TelaInicial;
use novasce_casa::{caminhos::Caminhos, chaveiro::ChaveiroSistema, config_casa::ConfigCasa, nucleo::Config, sinal_local::SinalLocal};
use novasce_voz::gravacao::Gravador;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

pub struct Estado {
    pub caminhos: Caminhos,
    pub config: Config,
    pub casa: ConfigCasa,
    pub chaveiro: ChaveiroSistema,
    pub sinal: SinalLocal,
    pub inicial: Mutex<TelaInicial>,
    pub print: Mutex<Option<Vec<u8>>>,
    pub gravador: Mutex<Option<Gravador>>,
}

/// Tira o print da tela atual. A janela da Ajuda ainda está escondida, então o print mostra onde a pessoa travou.
pub fn tirar_print(c: &Caminhos) -> Option<Vec<u8>> {
    let destino = c.runtime_novasce().join("print-ajuda.png");
    let _ = std::fs::create_dir_all(c.runtime_novasce());
    let ok = std::process::Command::new("spectacle")
        .args(["--background", "--nonotify", "--fullscreen", "--output"])
        .arg(&destino)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let bytes = ok.then(|| std::fs::read(&destino).ok()).flatten();
    let _ = std::fs::remove_file(&destino);
    bytes.and_then(|b| comandos::reduzir_png(&b, 1920).ok())
}

fn mostrar(app: &tauri::AppHandle, t: &TelaInicial) {
    let alvo = if t.modo == "pedir" { "pedir" } else { "central" };
    if let Some(w) = app.get_webview_window(alvo) {
        let _ = w.show();
        let _ = w.set_focus();
    }
    let _ = app.emit("ajuda://tela", t.clone());
}

pub fn rodar() {
    // WebKitGTK com placa NVIDIA (a sede) abre em branco sem isto, como no Preparar do M1.
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    let caminhos = Caminhos::da_pessoa();
    let inicial = argumentos::ler(&std::env::args().collect::<Vec<_>>());
    let print = if argumentos::precisa_print(&inicial) { tirar_print(&caminhos) } else { None };
    let config = Config::carregar().expect("config.toml da máquina");
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            let t = argumentos::ler(&args);
            if argumentos::precisa_print(&t) {
                for rotulo in ["central", "pedir"] {
                    if let Some(w) = app.get_webview_window(rotulo) {
                        let _ = w.hide();
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(150));
                let s = app.state::<Estado>();
                *s.print.lock().unwrap() = tirar_print(&s.caminhos);
            }
            *app.state::<Estado>().inicial.lock().unwrap() = t.clone();
            mostrar(app, &t);
        }))
        .manage(Estado {
            chaveiro: ChaveiroSistema::da_pessoa(&caminhos),
            caminhos,
            config,
            casa: ConfigCasa::carregar(),
            sinal: SinalLocal::padrao(),
            inicial: Mutex::new(inicial.clone()),
            print: Mutex::new(print),
            gravador: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            comandos::tela_inicial,
            comandos::preparar_pedido,
            comandos::enviar_pedido,
            comandos::tentar_agora,
            comandos::ditar_iniciar,
            comandos::ditar_nivel,
            comandos::ditar_parar,
            comandos::recomendados_instalados,
            comandos::instalar_recomendado,
            comandos::esqueci,
            comandos::novidades,
            comandos::abrir_pedir,
            comandos::abrir_comecar,
            comandos::abrir_url,
            comandos::fechar_janela,
            comandos::movimento,
        ])
        .setup(move |app| {
            mostrar(app.handle(), &inicial);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Ajuda");
}
