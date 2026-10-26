#[cfg(target_os = "linux")]
mod servico {
    use novasce_barra::atualizacao::{self, args_reiniciar, decidir, EstadoAviso, Passo};
    use novasce_barra::busca::{self, alvo_de_id, Alvo, ItemIndice};
    use novasce_barra::janela::{tela_da_classe, SCRIPT};
    use novasce_barra::manha::{
        self, deve_abrir, dia_de_trabalho, tenancy_responde, EstadoManha, AVISO_FORA,
    };
    use novasce_barra::partida::esperar_config;
    use novasce_casa::abrir::{abrir_tenancy, rodar_solto};
    use novasce_casa::caminhos::Caminhos;
    use novasce_casa::chaveiro::{Chaveiro, ChaveiroSistema, CONTA_TENANCY};
    use novasce_casa::config_casa::ConfigCasa;
    use novasce_casa::nucleo::Config;
    use novasce_casa::sistema::{conta_atual, status_ostree};
    use novasce_casa::{descobertas, json_atomico, preferencias, versoes};
    use novasce_chamados::fila::Fila;
    use novasce_chamados::pedido::{envio_para, esvaziar_fila, Envio};
    use novasce_chamados::respostas::{self, novidades, textos, Acompanhamento};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    use zbus::zvariant::{OwnedValue, Value};

    #[derive(Clone)]
    struct Ctx {
        c: Caminhos,
        casa: ConfigCasa,
        web: String,
        api: String,
        conta_sede: bool,
    }

    enum Pedido {
        Central,
        Pedir,
        MostrarAviso,
    }

    // ---------- sinais ----------
    struct Senhas {
        web: String,
    }
    impl ksni::Tray for Senhas {
        // ksni 0.2 não tem `MENU_ON_ACTIVATE` (é da 0.3): o clique esquerdo abre o Bitwarden
        // e o menu, com o Cofre do time, fica no clique direito.
        fn activate(&mut self, _x: i32, _y: i32) {
            let _ = rodar_solto(&["flatpak", "run", "com.bitwarden.desktop"]);
        }
        fn id(&self) -> String {
            "novasce-3-senhas".into()
        }
        fn title(&self) -> String {
            "Senhas".into()
        }
        fn icon_name(&self) -> String {
            "novasce-senhas".into()
        }
        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip {
                title: "Senhas · as suas e o Cofre do time".into(),
                ..Default::default()
            }
        }
        fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
            use ksni::menu::StandardItem;
            vec![
                StandardItem {
                    label: "Abrir o Bitwarden".into(),
                    activate: Box::new(|_| {
                        let _ = rodar_solto(&["flatpak", "run", "com.bitwarden.desktop"]);
                    }),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: "Abrir o Cofre do time".into(),
                    activate: Box::new(|t: &mut Self| {
                        let _ = abrir_tenancy(&t.web, "/cofre");
                    }),
                    ..Default::default()
                }
                .into(),
            ]
        }
    }

    struct AjudaSinal {
        tx: Sender<Pedido>,
        web: String,
        fora: bool,
    }
    impl ksni::Tray for AjudaSinal {
        fn id(&self) -> String {
            "novasce-4-ajuda".into()
        }
        fn title(&self) -> String {
            "Ajuda".into()
        }
        fn icon_name(&self) -> String {
            "novasce-ajuda".into()
        }
        fn status(&self) -> ksni::Status {
            if self.fora {
                ksni::Status::NeedsAttention
            } else {
                ksni::Status::Active
            }
        }
        fn tool_tip(&self) -> ksni::ToolTip {
            let t = if self.fora {
                AVISO_FORA
            } else {
                "Ajuda · abre um chamado com a Novasce"
            };
            ksni::ToolTip {
                title: t.into(),
                ..Default::default()
            }
        }
        fn activate(&mut self, _x: i32, _y: i32) {
            let _ = self.tx.send(Pedido::Central);
        }
        fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
            use ksni::menu::StandardItem;
            vec![
                StandardItem {
                    label: "Pedir ajuda".into(),
                    activate: Box::new(|t: &mut Self| {
                        let _ = t.tx.send(Pedido::Pedir);
                    }),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: "Abrir a Central de ajuda".into(),
                    activate: Box::new(|t: &mut Self| {
                        let _ = t.tx.send(Pedido::Central);
                    }),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: "Guia do Novasce OS".into(),
                    activate: Box::new(|t: &mut Self| {
                        let _ = abrir_tenancy(&t.web, "/time/material");
                    }),
                    ..Default::default()
                }
                .into(),
            ]
        }
    }

    struct Pronta {
        titulo: String,
        tx: Sender<Pedido>,
    }
    impl ksni::Tray for Pronta {
        fn id(&self) -> String {
            "novasce-0-atualizacao".into()
        }
        fn title(&self) -> String {
            self.titulo.clone()
        }
        fn icon_name(&self) -> String {
            "novasce-atualizacao".into()
        }
        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip {
                title: format!(
                    "{} · reinicie quando quiser. Nada fecha sem você mandar.",
                    self.titulo
                ),
                ..Default::default()
            }
        }
        fn activate(&mut self, _x: i32, _y: i32) {
            let _ = self.tx.send(Pedido::MostrarAviso);
        }
    }

    // ---------- D-Bus: janela ativa e KRunner ----------
    struct Janela {
        tx: Mutex<Sender<String>>,
    }
    #[zbus::interface(name = "org.novasce.Janela")]
    impl Janela {
        fn ativa(&self, classe: String) {
            let _ = self.tx.lock().unwrap().send(classe);
        }
    }

    type NotasEmCache = (Option<Instant>, Vec<(PathBuf, String)>);
    struct Busca {
        c: Caminhos,
        indice: Vec<ItemIndice>,
        cache: Mutex<NotasEmCache>,
    }
    type Resultado = (
        String,
        String,
        String,
        i32,
        f64,
        HashMap<String, OwnedValue>,
    );
    #[zbus::interface(name = "org.kde.krunner1")]
    impl Busca {
        fn actions(&self) -> Vec<(String, String, String)> {
            vec![]
        }
        #[zbus(name = "Match")]
        fn procurar(&self, consulta: String) -> Vec<Resultado> {
            let mut cache = self.cache.lock().unwrap();
            if cache
                .0
                .is_none_or(|q| q.elapsed() > Duration::from_secs(60))
            {
                *cache = (Some(Instant::now()), busca::notas(&self.c.brain(), 5000));
            }
            busca::procurar(&consulta, &cache.1, &self.c.brain(), &self.indice, 8)
                .into_iter()
                .map(|a| {
                    let mut p = HashMap::new();
                    if let Ok(v) = Value::from(a.sub.clone()).try_to_owned() {
                        p.insert("subtext".to_string(), v);
                    }
                    (
                        a.id,
                        a.texto,
                        "novasce-logo".to_string(),
                        50,
                        a.relevancia,
                        p,
                    )
                })
                .collect()
        }
        fn run(&self, id: String, _acao: String) {
            match alvo_de_id(&id) {
                Some(Alvo::Nota(rel)) => {
                    let _ = rodar_solto(&["xdg-open", &busca::url_obsidian(&rel)]);
                    let _ = descobertas::marcar(&self.c, descobertas::Descoberta::AchouNota);
                }
                Some(Alvo::Ajuda(alvo)) => {
                    let _ = rodar_solto(&["novasce-ajuda", "--artigo", &alvo]);
                }
                None => {}
            }
        }
    }

    fn tela_ativa(ctx: &Ctx, rx: &Receiver<String>) -> String {
        while rx.try_recv().is_ok() {}
        let tentar = || -> zbus::Result<String> {
            let arq = ctx.c.runtime_novasce().join("janela-ativa.js");
            let _ = std::fs::create_dir_all(ctx.c.runtime_novasce());
            std::fs::write(&arq, SCRIPT).map_err(|e| zbus::Error::Failure(e.to_string()))?;
            let con = zbus::blocking::Connection::session()?;
            let kwin = zbus::blocking::Proxy::new(
                &con,
                "org.kde.KWin",
                "/Scripting",
                "org.kde.kwin.Scripting",
            )?;
            let _: bool = kwin
                .call("unloadScript", &("novasce-janela-ativa",))
                .unwrap_or(false);
            let id: i32 = kwin.call(
                "loadScript",
                &(arq.to_string_lossy().to_string(), "novasce-janela-ativa"),
            )?;
            let s = zbus::blocking::Proxy::new(
                &con,
                "org.kde.KWin",
                format!("/Scripting/Script{id}"),
                "org.kde.kwin.Script",
            )?;
            s.call::<_, _, ()>("run", &())?;
            let classe = rx
                .recv_timeout(Duration::from_millis(600))
                .unwrap_or_default();
            let _: bool = kwin
                .call("unloadScript", &("novasce-janela-ativa",))
                .unwrap_or(false);
            Ok(classe)
        };
        tela_da_classe(&tentar().unwrap_or_default(), ctx.conta_sede).to_string()
    }

    fn envio(ctx: &Ctx) -> Box<dyn Envio> {
        envio_para(
            &conta_atual(),
            &ctx.casa.sede.conta,
            ChaveiroSistema::da_pessoa(&ctx.c).ler(CONTA_TENANCY),
            &ctx.api,
        )
    }

    fn avisar(titulo: &str, corpo: &str) {
        let _ = notify_rust::Notification::new()
            .appname("Novasce OS")
            .icon("novasce-ajuda")
            .summary(titulo)
            .body(corpo)
            .show();
    }

    // ---------- aviso de atualização ----------
    /// Devolve `false` quando o serviço de notificações não respondeu (nada foi mostrado).
    fn mostrar_aviso(ctx: &Ctx, a: &versoes::Aviso, imagem: &str) -> bool {
        let r = notify_rust::Notification::new()
            .appname("Novasce OS")
            .icon("novasce-atualizacao")
            .summary(&a.titulo)
            .body(&a.corpo)
            .action("reiniciar", "Reiniciar agora")
            .action("depois", "Depois")
            .timeout(notify_rust::Timeout::Never)
            .show();
        let Ok(h) = r else { return false };
        let mut reiniciar = false;
        h.wait_for_action(|acao| reiniciar = acao == "reiniciar");
        // Avisou uma vez: dali em diante fica só o sinal, mesmo que a pessoa desista de reiniciar.
        let _ = json_atomico::gravar(
            &atualizacao::caminho(&ctx.c),
            &EstadoAviso {
                avisada: Some(imagem.into()),
            },
        );
        if reiniciar {
            let a = args_reiniciar();
            let _ = rodar_solto(&a);
        }
        true
    }

    fn ciclo_atualizacao(ctx: Ctx, tx: Sender<Pedido>, pedidos_aviso: Receiver<()>) {
        let mut sinal: Option<ksni::Handle<Pronta>> = None;
        loop {
            let status = status_ostree().unwrap_or_default();
            let notas = versoes::buscar_remota(&ctx.casa.atualizacao.notas_url)
                .unwrap_or_else(|_| versoes::ler_local(&ctx.casa.atualizacao.notas_local));
            let estado: EstadoAviso = json_atomico::ler(&atualizacao::caminho(&ctx.c));
            match decidir(&status, &notas, &estado, ctx.conta_sede) {
                Passo::Avisar(a, imagem) => {
                    // Sem serviço de notificações, tenta de novo na próxima volta (30 min), sem girar.
                    if mostrar_aviso(&ctx, &a, &imagem) {
                        continue;
                    }
                }
                Passo::Sinal { titulo } => {
                    if let Some(h) = &sinal {
                        let t = titulo.clone();
                        h.update(move |p: &mut Pronta| p.titulo = t);
                    } else {
                        let s = ksni::TrayService::new(Pronta {
                            titulo,
                            tx: tx.clone(),
                        });
                        sinal = Some(s.handle());
                        s.spawn();
                    }
                }
                Passo::Esconder => {
                    if let Some(h) = sinal.take() {
                        h.shutdown();
                    }
                }
                Passo::Nada => {}
            }
            if pedidos_aviso
                .recv_timeout(Duration::from_secs(1800))
                .is_ok()
            {
                if let Some(imagem) = novasce_casa::sistema::versao_pronta(&status) {
                    let _ = mostrar_aviso(&ctx, &versoes::aviso(&notas, &imagem), &imagem);
                }
            }
        }
    }

    // ---------- Meu dia ----------
    fn tentar_manha(ctx: &Ctx, ajuda: &ksni::Handle<AjudaSinal>) -> bool {
        let pref = preferencias::ler(&ctx.c);
        let mut e: EstadoManha = json_atomico::ler(&manha::caminho(&ctx.c));
        let hoje = dia_de_trabalho(chrono::Local::now().naive_local());
        if !deve_abrir(pref.manha, ctx.conta_sede, &e, hoje) {
            return true;
        }
        if tenancy_responde(&ctx.web) {
            let _ = abrir_tenancy(&ctx.web, "/meu-dia");
            e.ultimo_dia = Some(hoje.to_string());
            let _ = json_atomico::gravar(&manha::caminho(&ctx.c), &e);
            ajuda.update(|a: &mut AjudaSinal| a.fora = false);
            true
        } else {
            ajuda.update(|a: &mut AjudaSinal| a.fora = true);
            false
        }
    }

    fn ouvir_sinal(
        barramento: fn() -> zbus::Result<zbus::blocking::Connection>,
        interface: &'static str,
        membro: &'static str,
        valor: bool,
        tx: Sender<()>,
    ) {
        std::thread::spawn(move || -> zbus::Result<()> {
            let con = barramento()?;
            let regra = zbus::MatchRule::builder()
                .msg_type(zbus::message::Type::Signal)
                .interface(interface)?
                .member(membro)?
                .build();
            for m in zbus::blocking::MessageIterator::for_match_rule(regra, &con, Some(16))? {
                if m?.body().deserialize::<bool>().ok() == Some(valor) {
                    let _ = tx.send(());
                }
            }
            Ok(())
        });
    }

    fn ciclo_manha(ctx: Ctx, ajuda: ksni::Handle<AjudaSinal>) {
        let (tx, rx) = channel::<()>();
        ouvir_sinal(
            zbus::blocking::Connection::session,
            "org.freedesktop.ScreenSaver",
            "ActiveChanged",
            false,
            tx.clone(),
        );
        ouvir_sinal(
            zbus::blocking::Connection::system,
            "org.freedesktop.login1.Manager",
            "PrepareForSleep",
            false,
            tx,
        );
        let mut falhas = if tentar_manha(&ctx, &ajuda) { 0 } else { 1 };
        loop {
            let espera = if falhas > 0 && falhas < 24 {
                Duration::from_secs(300)
            } else {
                Duration::from_secs(3600)
            };
            let veio_sinal = rx.recv_timeout(espera).is_ok();
            if veio_sinal {
                falhas = 0;
            }
            if veio_sinal || (falhas > 0 && falhas < 24) {
                falhas = if tentar_manha(&ctx, &ajuda) {
                    0
                } else {
                    falhas + 1
                };
            }
        }
    }

    // ---------- respostas e fila ----------
    fn ciclo_chamados(ctx: Ctx) {
        let via = if ChaveiroSistema::da_pessoa(&ctx.c)
            .ler(CONTA_TENANCY)
            .is_some()
            && !ctx.conta_sede
        {
            "pessoa"
        } else {
            "maquina"
        };
        let mut contador = 0u32;
        loop {
            std::thread::sleep(Duration::from_secs(120));
            contador += 1;
            let fila = Fila::da_pessoa(&ctx.c);
            if fila.tem_pendentes() {
                for c in esvaziar_fila(envio(&ctx).as_ref(), &fila) {
                    avisar(&format!("Chamado aberto{}", c.numero.map(|n| format!(" · {n}")).unwrap_or_default()), "O pedido que estava guardado saiu agora. A resposta chega aqui e no Tenancy.");
                }
            }
            if contador % 3 != 1 {
                continue;
            }
            let caminho = respostas::caminho(&ctx.c, via);
            let primeira_vez = !caminho.exists();
            let mut a: Acompanhamento = json_atomico::ler(&caminho);
            let Ok(lista) = envio(&ctx).respostas(a.desde.as_deref()) else {
                continue;
            };
            let novas = novidades(&lista, &mut a);
            let _ = json_atomico::gravar(&caminho, &a);
            if primeira_vez {
                continue;
            }
            for n in novas {
                let t = textos(&n);
                let ctx2 = ctx.clone();
                std::thread::spawn(move || {
                    let mut aviso = notify_rust::Notification::new();
                    aviso
                        .appname("Novasce OS")
                        .icon("novasce-ajuda")
                        .summary(&t.titulo)
                        .body(&t.corpo);
                    for (id, rotulo) in &t.acoes {
                        aviso.action(id, rotulo);
                    }
                    if !t.acoes.is_empty() {
                        aviso.timeout(notify_rust::Timeout::Never);
                    }
                    let Ok(h) = aviso.show() else { return };
                    if t.acoes.is_empty() {
                        return;
                    }
                    let mut escolha = String::new();
                    h.wait_for_action(|acao| escolha = acao.to_string());
                    match escolha.as_str() {
                        "resolveu" => {
                            let _ = envio(&ctx2).retorno(&n.ticket_id, true);
                        }
                        "ainda-nao" => {
                            let _ = envio(&ctx2).retorno(&n.ticket_id, false);
                            avisar(
                                "Avisamos a Novasce",
                                "O chamado volta para a fila. A resposta nova chega aqui.",
                            );
                        }
                        _ => {}
                    }
                });
            }
        }
    }

    pub fn rodar() {
        let c = Caminhos::da_pessoa();
        let casa = ConfigCasa::carregar();
        // Máquina ainda não preparada: espera em silêncio em vez de cair e deixar o systemd
        // reiniciar o serviço a cada 10 s.
        let config = esperar_config(Config::carregar, std::thread::sleep, |linha| {
            eprintln!("novasce-barra: {linha}")
        });
        let ctx = Ctx {
            conta_sede: conta_atual() == casa.sede.conta,
            web: config.tenancy_web.clone(),
            api: config.tenancy_api.clone(),
            c: c.clone(),
            casa,
        };

        let (tx_janela, rx_janela) = channel::<String>();
        let _con = zbus::blocking::connection::Builder::session()
            .and_then(|b| b.name("org.novasce.Barra"))
            .and_then(|b| {
                b.serve_at(
                    "/janela",
                    Janela {
                        tx: Mutex::new(tx_janela),
                    },
                )
            })
            .and_then(|b| {
                b.serve_at(
                    "/busca",
                    Busca {
                        c: c.clone(),
                        indice: busca::ler_indice("/usr/share/novasce/ajuda/indice.json"),
                        cache: Mutex::new((None, vec![])),
                    },
                )
            })
            .and_then(|b| b.build());

        let (tx, rx) = channel::<Pedido>();
        let ajuda = ksni::TrayService::new(AjudaSinal {
            tx: tx.clone(),
            web: ctx.web.clone(),
            fora: false,
        });
        let alca_ajuda = ajuda.handle();
        ajuda.spawn();
        if !ctx.conta_sede {
            ksni::TrayService::new(Senhas {
                web: ctx.web.clone(),
            })
            .spawn();
            let (c2, a2) = (ctx.clone(), alca_ajuda.clone());
            std::thread::spawn(move || ciclo_manha(c2, a2));
        }
        let (tx_aviso, rx_aviso) = channel::<()>();
        let (c3, t3) = (ctx.clone(), tx.clone());
        std::thread::spawn(move || ciclo_atualizacao(c3, t3, rx_aviso));
        let c4 = ctx.clone();
        std::thread::spawn(move || ciclo_chamados(c4));

        let rx_janela = Arc::new(Mutex::new(rx_janela));
        for p in rx {
            match p {
                Pedido::Central => {
                    let tela = tela_ativa(&ctx, &rx_janela.lock().unwrap());
                    let _ = rodar_solto(&["novasce-ajuda", "--central", "--tela", &tela]);
                }
                Pedido::Pedir => {
                    let _ = rodar_solto(&["novasce-ajuda", "--pedir"]);
                }
                Pedido::MostrarAviso => {
                    let _ = tx_aviso.send(());
                }
            }
        }
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    servico::rodar();
}
