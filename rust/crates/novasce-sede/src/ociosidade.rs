//! "Ninguém mexe há N minutos" no Wayland, pelo protocolo ext-idle-notify-v1 do KWin.
//! Respeita inibidores: vídeo de reunião tocando segura a contagem.
use std::sync::mpsc::Sender;
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::{wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1, ext_idle_notifier_v1,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Evento {
    Ocioso,
    Voltou,
}

struct Estado {
    tx: Sender<Evento>,
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Estado {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<wl_seat::WlSeat, ()> for Estado {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<ext_idle_notifier_v1::ExtIdleNotifierV1, ()> for Estado {
    fn event(
        _: &mut Self,
        _: &ext_idle_notifier_v1::ExtIdleNotifierV1,
        _: ext_idle_notifier_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<ext_idle_notification_v1::ExtIdleNotificationV1, ()> for Estado {
    fn event(
        e: &mut Self,
        _: &ext_idle_notification_v1::ExtIdleNotificationV1,
        ev: ext_idle_notification_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let _ = match ev {
            ext_idle_notification_v1::Event::Idled => e.tx.send(Evento::Ocioso),
            ext_idle_notification_v1::Event::Resumed => e.tx.send(Evento::Voltou),
            _ => Ok(()),
        };
    }
}

/// Bloqueia para sempre, mandando Ocioso depois de `ms` sem uso e Voltou quando alguém mexe.
pub fn observar(ms: u32, tx: Sender<Evento>) -> Result<(), String> {
    let con = Connection::connect_to_env().map_err(|e| e.to_string())?;
    let (globais, mut fila) = registry_queue_init::<Estado>(&con).map_err(|e| e.to_string())?;
    let qh = fila.handle();
    let assento: wl_seat::WlSeat = globais.bind(&qh, 1..=8, ()).map_err(|e| e.to_string())?;
    let avisador: ext_idle_notifier_v1::ExtIdleNotifierV1 = globais
        .bind(&qh, 1..=1, ())
        .map_err(|_| "o compositor não oferece ext-idle-notify-v1".to_string())?;
    let _n = avisador.get_idle_notification(ms, &assento, &qh, ());
    let mut e = Estado { tx };
    loop {
        fila.blocking_dispatch(&mut e).map_err(|x| x.to_string())?;
    }
}
