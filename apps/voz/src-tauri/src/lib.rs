mod motor;

use novasce_casa::{caminhos::Caminhos, config_casa::ConfigCasa, descobertas, movimento};
use novasce_voz::gesto::Entrada;
use novasce_voz::vocabulario::carregar;
use novasce_voz::whisper::{cpu_compativel, threads_padrao, Whisper};
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{Emitter, Manager};

struct Canal(Mutex<Sender<Entrada>>, Instant);

#[tauri::command]
fn movimento() -> f32 {
    movimento::fator_atual(&Caminhos::da_pessoa())
}

fn agora(c: &Canal) -> u64 {
    c.1.elapsed().as_millis() as u64
}

#[cfg(target_os = "linux")]
struct Falar {
    tx: Sender<Entrada>,
    inicio: Instant,
    qualquer_campo: bool,
}

#[cfg(target_os = "linux")]
impl ksni::Tray for Falar {
    fn id(&self) -> String {
        "novasce-1-falar".into()
    }
    fn title(&self) -> String {
        "Falar".into()
    }
    fn icon_name(&self) -> String {
        "novasce-falar".into()
    }
    fn tool_tip(&self) -> ksni::ToolTip {
        let t = if self.qualquer_campo { "Falar · segure Super + Espaço e fale" } else { "Falar · use o botão Falar nos apps da casa" };
        ksni::ToolTip { title: t.into(), ..Default::default() }
    }
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(Entrada::Clique(self.inicio.elapsed().as_millis() as u64));
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::StandardItem;
        vec![
            StandardItem { label: "Testar agora".into(), activate: Box::new(|t: &mut Self| { let _ = t.tx.send(Entrada::Clique(t.inicio.elapsed().as_millis() as u64)); }), ..Default::default() }.into(),
            StandardItem { label: "Trocar o atalho".into(), activate: Box::new(|_| { let _ = novasce_casa::abrir::rodar_solto(&["kcmshell6", "kcm_keys"]); }), ..Default::default() }.into(),
        ]
    }
}

/// Portal GlobalShortcuts: Activated = apertou, Deactivated = soltou (xdg-desktop-portal-kde emite os dois).
#[cfg(target_os = "linux")]
async fn atalho(tx: Sender<Entrada>, inicio: Instant) -> ashpd::Result<()> {
    use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
    use futures_util::StreamExt;
    let gs = GlobalShortcuts::new().await?;
    let sessao = gs.create_session().await?;
    let pedido = [NewShortcut::new("falar", "Falar em vez de digitar").preferred_trigger("LOGO+space")];
    gs.bind_shortcuts(&sessao, &pedido, &ashpd::WindowIdentifier::default()).await?.response()?;
    let mut apertou = gs.receive_activated().await?;
    let mut soltou = gs.receive_deactivated().await?;
    let mut mudou = gs.receive_shortcuts_changed().await?;
    let ms = || inicio.elapsed().as_millis() as u64;
    loop {
        tokio::select! {
            Some(_) = apertou.next() => { let _ = tx.send(Entrada::Apertou(ms())); }
            Some(_) = soltou.next() => { let _ = tx.send(Entrada::Soltou(ms())); }
            Some(_) = mudou.next() => { let _ = descobertas::marcar(&Caminhos::da_pessoa(), descobertas::Descoberta::TrocouAtalho); }
            else => break,
        }
    }
    Ok(())
}

pub fn rodar() {
    // WebKitGTK com placa NVIDIA (a sede) abre em branco sem isto, como no Preparar do M1.
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    let c = Caminhos::da_pessoa();
    let casa = ConfigCasa::carregar();
    if novasce_casa::sistema::conta_atual() == casa.sede.conta {
        return;
    }
    let (tx, rx) = channel::<Entrada>();
    let inicio = Instant::now();
    let modelo = if std::path::Path::new(&casa.voz.modelo).exists() { casa.voz.modelo.clone() } else { casa.voz.modelo_reserva.clone() };
    let m = motor::Motor {
        whisper: Whisper { binario: casa.voz.whisper.clone().into(), modelo: modelo.into(), threads: threads_padrao() },
        vocabulario: carregar(&casa.voz.vocabulario, &c.config_novasce().join("voz-vocabulario.toml")),
        qualquer_campo: casa.voz.qualquer_campo,
        cpu_ok: cpu_compativel(&std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default()),
        pasta_tmp: c.runtime_novasce(),
        limite_toque_ms: casa.voz.limite_toque_ms,
    };
    let qualquer_campo = casa.voz.qualquer_campo;
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            if args.iter().any(|a| a == "--abrir") {
                let c = app.state::<Canal>();
                let _ = c.0.lock().unwrap().send(Entrada::Clique(agora(&c)));
            }
        }))
        .manage(Canal(Mutex::new(tx.clone()), inicio))
        .invoke_handler(tauri::generate_handler![movimento])
        .setup(move |app| {
            let painel = app.get_webview_window("painel").expect("janela painel");
            #[cfg(target_os = "linux")]
            {
                use gtk::prelude::*;
                use novasce_camada::{ancorar, Edge, KeyboardMode};
                ancorar(painel.gtk_window()?.upcast_ref(), &[(Edge::Bottom, 28)], KeyboardMode::None);
                let s = ksni::TrayService::new(Falar { tx: tx.clone(), inicio, qualquer_campo });
                s.spawn();
                if qualquer_campo {
                    let tx_atalho = tx.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = atalho(tx_atalho, inicio).await;
                    });
                }
            }
            let alca = app.handle().clone();
            std::thread::spawn(move || {
                let caminhos = Caminhos::da_pessoa();
                motor::rodar(
                    m,
                    rx,
                    inicio,
                    |estado| {
                        if let Some(w) = alca.get_webview_window("painel") {
                            let _ = if estado.fase == "oculto" { w.hide() } else { w.show() };
                        }
                        let _ = alca.emit("voz://estado", estado);
                    },
                    || {
                        let _ = descobertas::marcar(&caminhos, descobertas::Descoberta::Falou);
                    },
                );
            });
            if std::env::args().any(|a| a == "--abrir") {
                let _ = tx.send(Entrada::Clique(0));
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Voz");
}
