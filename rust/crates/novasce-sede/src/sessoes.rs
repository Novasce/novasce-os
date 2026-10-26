//! Sessões do logind na sede: a da conta Sede, e se dá para reiniciar sem fechar o app de alguém.
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub struct Sessao {
    pub id: String,
    pub usuario: String,
    pub assento: String,
    /// "online", "active" ou "closing" (do logind). Sessão que fechou com processo
    /// sobrando fica em "closing" e ainda aparece na lista.
    pub estado: String,
}

const TELAS_DE_LOGIN: [&str; 4] = ["plasmalogin", "sddm", "gdm", "lightdm"];

pub fn sessao_da_sede<'a>(l: &'a [Sessao], conta_sede: &str) -> Option<&'a Sessao> {
    l.iter()
        .find(|s| s.usuario == conta_sede && s.assento == "seat0")
}

#[derive(Debug, Clone, PartialEq)]
pub enum PodeReiniciar {
    Sim,
    AgentesEmRodada,
    OutraConta(String),
}

/// `agentes` é a saída de `systemctl is-active novasce-assistente.service` (oneshot: "activating" durante a rodada).
pub fn pode_reiniciar(agentes: &str, l: &[Sessao], conta_sede: &str) -> PodeReiniciar {
    if matches!(agentes.trim(), "active" | "activating" | "reloading") {
        return PodeReiniciar::AgentesEmRodada;
    }
    match l.iter().find(|s| {
        s.usuario != conta_sede
            && !s.usuario.is_empty()
            && !TELAS_DE_LOGIN.contains(&s.usuario.as_str())
    }) {
        Some(s) => PodeReiniciar::OutraConta(s.usuario.clone()),
        None => PodeReiniciar::Sim,
    }
}

/// A TV está só na tela de login: há uma tela de login no seat0 e nenhuma conta dentro.
/// Sessão fechando não conta; terminal (Ctrl+Alt+F3) no seat0 conta como conta dentro.
pub fn so_a_tela_de_login(l: &[Sessao]) -> bool {
    let mut tela_de_login = false;
    for s in l
        .iter()
        .filter(|s| s.assento == "seat0" && s.estado != "closing")
    {
        if TELAS_DE_LOGIN.contains(&s.usuario.as_str()) {
            tela_de_login = true;
        } else if !s.usuario.is_empty() {
            return false;
        }
    }
    tela_de_login
}

/// Conta as conferências seguidas com a TV só no login. Na `vezes`-ésima seguida,
/// diz que é hora de reiniciar a tela de login, e a conta recomeça.
pub struct Guarda {
    seguidas: u32,
    vezes: u32,
}

impl Guarda {
    pub fn nova(vezes: u32) -> Self {
        Self { seguidas: 0, vezes }
    }

    pub fn conferir(&mut self, so_login: bool) -> bool {
        if !so_login {
            self.seguidas = 0;
            return false;
        }
        self.seguidas += 1;
        if self.seguidas >= self.vezes {
            self.seguidas = 0;
            return true;
        }
        false
    }
}

pub fn motivo(p: &PodeReiniciar) -> Option<String> {
    match p {
        PodeReiniciar::Sim => None,
        PodeReiniciar::AgentesEmRodada => {
            Some("Os agentes estão numa rodada. O botão volta quando ela terminar.".into())
        }
        PodeReiniciar::OutraConta(u) => Some(format!(
            "A conta de {u} está aberta por trás. Reiniciar fecharia os apps dessa pessoa."
        )),
    }
}

pub fn agentes_agora() -> String {
    Command::new("systemctl")
        .args(["is-active", "novasce-assistente.service"])
        .output()
        .map(|s| String::from_utf8_lossy(&s.stdout).into_owned())
        .unwrap_or_default()
}

#[cfg(target_os = "linux")]
pub fn listar() -> Result<Vec<Sessao>, String> {
    let con = zbus::blocking::Connection::system().map_err(|e| e.to_string())?;
    let p = zbus::blocking::Proxy::new(
        &con,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .map_err(|e| e.to_string())?;
    let l: Vec<(String, u32, String, String, zbus::zvariant::OwnedObjectPath)> =
        p.call("ListSessions", &()).map_err(|e| e.to_string())?;
    Ok(l.into_iter()
        .map(|(id, _, usuario, assento, caminho)| Sessao {
            estado: estado(&con, &caminho).unwrap_or_default(),
            id,
            usuario,
            assento,
        })
        .collect())
}

#[cfg(target_os = "linux")]
fn estado(
    con: &zbus::blocking::Connection,
    caminho: &zbus::zvariant::OwnedObjectPath,
) -> Option<String> {
    zbus::blocking::Proxy::new(
        con,
        "org.freedesktop.login1",
        caminho.as_str(),
        "org.freedesktop.login1.Session",
    )
    .ok()?
    .get_property::<String>("State")
    .ok()
}

#[cfg(target_os = "linux")]
pub fn ativar(id: &str) -> Result<(), String> {
    let con = zbus::blocking::Connection::system().map_err(|e| e.to_string())?;
    let p = zbus::blocking::Proxy::new(
        &con,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .map_err(|e| e.to_string())?;
    p.call::<_, _, ()>("ActivateSession", &(id,))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod testes {
    use super::*;

    fn s(id: &str, u: &str) -> Sessao {
        Sessao {
            id: id.into(),
            usuario: u.into(),
            assento: "seat0".into(),
            estado: "active".into(),
        }
    }

    #[test]
    fn acha_a_sessao_da_sede() {
        let l = vec![s("3", "luan"), s("2", "sede")];
        assert_eq!(sessao_da_sede(&l, "sede").unwrap().id, "2");
        assert!(sessao_da_sede(&[s("3", "luan")], "sede").is_none());
    }

    #[test]
    fn reiniciar_so_quando_ninguem_perde_nada() {
        assert_eq!(
            pode_reiniciar(
                "inactive\n",
                &[s("2", "sede"), s("9", "plasmalogin")],
                "sede"
            ),
            PodeReiniciar::Sim
        );
        assert_eq!(
            pode_reiniciar("activating\n", &[s("2", "sede")], "sede"),
            PodeReiniciar::AgentesEmRodada
        );
        assert_eq!(
            pode_reiniciar("inactive", &[s("2", "sede"), s("3", "luan")], "sede"),
            PodeReiniciar::OutraConta("luan".into())
        );
    }

    #[test]
    fn so_a_tela_de_login_quando_ninguem_esta_dentro() {
        let fechando = Sessao {
            estado: "closing".into(),
            ..s("2", "sede")
        };
        let por_ssh = Sessao {
            assento: String::new(),
            ..s("7", "eduardo")
        };
        assert!(so_a_tela_de_login(&[s("9", "plasmalogin")]));
        assert!(so_a_tela_de_login(&[fechando, s("9", "plasmalogin")]));
        assert!(so_a_tela_de_login(&[por_ssh, s("9", "plasmalogin")]));
        // A Sede ou uma conta pessoal dentro: a guarda não mexe.
        assert!(!so_a_tela_de_login(&[
            s("2", "sede"),
            s("9", "plasmalogin")
        ]));
        assert!(!so_a_tela_de_login(&[
            s("3", "luan"),
            s("9", "plasmalogin")
        ]));
        // Sem tela de login (o boot ainda não terminou), também não.
        assert!(!so_a_tela_de_login(&[]));
    }

    #[test]
    fn a_guarda_espera_as_conferencias_seguidas() {
        let mut g = Guarda::nova(4);
        assert!(!g.conferir(true));
        assert!(!g.conferir(true));
        assert!(!g.conferir(false));
        for _ in 0..3 {
            assert!(!g.conferir(true));
        }
        assert!(g.conferir(true));
        // Depois de mandar reiniciar, a conta recomeça.
        assert!(!g.conferir(true));
    }

    #[test]
    fn o_motivo_vira_texto() {
        assert_eq!(motivo(&PodeReiniciar::Sim), None);
        assert_eq!(
            motivo(&PodeReiniciar::AgentesEmRodada).unwrap(),
            "Os agentes estão numa rodada. O botão volta quando ela terminar."
        );
        assert_eq!(
            motivo(&PodeReiniciar::OutraConta("luan".into())).unwrap(),
            "A conta de luan está aberta por trás. Reiniciar fecharia os apps dessa pessoa."
        );
    }
}
