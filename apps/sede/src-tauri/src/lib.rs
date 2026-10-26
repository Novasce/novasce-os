use novasce_casa::{caminhos::Caminhos, config_casa::ConfigCasa, movimento as mov, sistema};
use novasce_sede::{kwin, navegador, sessoes};
use serde::Serialize;
use tauri::{Emitter, Manager, State};

struct Estado {
    c: Caminhos,
    casa: ConfigCasa,
}

#[derive(Clone, Serialize)]
struct CantoDados {
    versao: String,
    motivo: Option<String>,
}

fn trazer_sala(e: &Estado) {
    #[cfg(target_os = "linux")]
    {
        let _ = kwin::rodar_script(
            &e.c.runtime_novasce(),
            "novasce-trazer-sala",
            &kwin::script_trazer_sala(&e.casa.sede.titulo_sala),
        );
    }
    let _ = e;
}

/// Esconde o véu e zera a contagem na tela: sem isso o relógio da tela continuaria em
/// segundo plano e, ao zerar, esqueceria o navegador de quem já voltou a usar.
fn esconder_volta(app: &tauri::AppHandle) {
    let _ = app.emit("sede://volta", None::<u32>);
    if let Some(w) = app.get_webview_window("volta") {
        let _ = w.hide();
    }
}

#[tauri::command(async)]
fn voltar_agora(app: tauri::AppHandle, s: State<Estado>) {
    let _ = navegador::esquecer(&navegador::perfil(&s.c.casa, &s.casa.sede));
    trazer_sala(&s);
    esconder_volta(&app);
}

#[tauri::command]
fn continuar_usando(app: tauri::AppHandle) {
    esconder_volta(&app);
}

#[tauri::command(async)]
fn reiniciar(s: State<Estado>) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let p = sessoes::pode_reiniciar(
            &sessoes::agentes_agora(),
            &sessoes::listar().unwrap_or_default(),
            &s.casa.sede.conta,
        );
        if let Some(m) = sessoes::motivo(&p) {
            return Err(m);
        }
        std::process::Command::new("busctl")
            .args([
                "--user",
                "call",
                "org.kde.Shutdown",
                "/Shutdown",
                "org.kde.Shutdown",
                "logoutAndReboot",
            ])
            .status()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = s;
        Ok(())
    }
}

#[tauri::command]
fn movimento(s: State<Estado>) -> f32 {
    mov::fator_atual(&s.c)
}

pub fn rodar() {
    // WebKitGTK com placa NVIDIA (a sede) abre em branco sem isto, como no Preparar do M1.
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    let casa = ConfigCasa::carregar();
    if sistema::conta_atual() != casa.sede.conta {
        return;
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
        .manage(Estado {
            c: Caminhos::da_pessoa(),
            casa: casa.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            voltar_agora,
            continuar_usando,
            reiniciar,
            movimento
        ])
        .setup(move |app| {
            #[cfg(target_os = "linux")]
            {
                use gtk::prelude::*;
                use novasce_camada::{ancorar, Edge, KeyboardMode};
                let volta = app.get_webview_window("volta").expect("volta");
                ancorar(
                    volta.gtk_window()?.upcast_ref(),
                    &[
                        (Edge::Top, 0),
                        (Edge::Bottom, 0),
                        (Edge::Left, 0),
                        (Edge::Right, 0),
                    ],
                    KeyboardMode::OnDemand,
                );
                let canto = app.get_webview_window("canto").expect("canto");
                ancorar(
                    canto.gtk_window()?.upcast_ref(),
                    &[(Edge::Bottom, 24), (Edge::Right, 24)],
                    KeyboardMode::None,
                );

                let alca = app.handle().clone();
                let casa2 = casa.clone();
                std::thread::spawn(move || {
                    use novasce_sede::ociosidade::{observar, Evento};
                    let (tx, rx) = std::sync::mpsc::channel();
                    let ms = casa2.sede.volta_minutos * 60_000;
                    std::thread::spawn(move || observar(ms, tx));
                    let c = Caminhos::da_pessoa();
                    for ev in rx {
                        let w = alca.get_webview_window("volta");
                        match ev {
                            Evento::Ocioso
                                if navegador::aberto(&navegador::perfil(&c.casa, &casa2.sede)) =>
                            {
                                let _ =
                                    alca.emit("sede://volta", Some(casa2.sede.contagem_segundos));
                                if let Some(w) = w {
                                    let _ = w.show();
                                }
                            }
                            Evento::Ocioso => {
                                let e = alca.state::<Estado>();
                                trazer_sala(&e);
                            }
                            Evento::Voltou => esconder_volta(&alca),
                        }
                    }
                });

                let alca2 = app.handle().clone();
                let casa3 = casa.clone();
                std::thread::spawn(move || loop {
                    let status = sistema::status_ostree().unwrap_or_default();
                    let dados = sistema::versao_pronta(&status).map(|imagem| {
                        let p = sessoes::pode_reiniciar(
                            &sessoes::agentes_agora(),
                            &sessoes::listar().unwrap_or_default(),
                            &casa3.sede.conta,
                        );
                        CantoDados {
                            versao: imagem,
                            motivo: sessoes::motivo(&p),
                        }
                    });
                    let _ = alca2.emit("sede://canto", dados.clone());
                    if let Some(w) = alca2.get_webview_window("canto") {
                        let _ = if dados.is_some() { w.show() } else { w.hide() };
                    }
                    std::thread::sleep(std::time::Duration::from_secs(60));
                });
            }
            let _ = app;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Sede");
}
