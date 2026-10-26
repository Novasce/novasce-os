#[cfg(target_os = "linux")]
mod servico {
    use novasce_casa::caminhos::Caminhos;
    use novasce_casa::config_casa::ConfigCasa;
    use novasce_casa::sistema::{conta_atual, nome_para_arquivo};
    use novasce_casa::{json_atomico, sinal_local::SinalLocal};
    use novasce_sincronizar::brain::{sincronizar_brain, Gitleaks};
    use novasce_sincronizar::documentos::{self, cache_bisync_padrao, ConfigDocs, ErroDocs};
    use novasce_sincronizar::estado::{
        self, cor, dica, do_resumo_brain, Cor, EstadoSync, Item, Situacao,
    };
    use std::sync::mpsc;
    use std::time::Duration;

    struct Nuvem {
        estado: EstadoSync,
        avisar: mpsc::Sender<()>,
    }

    impl ksni::Tray for Nuvem {
        fn id(&self) -> String {
            "novasce-2-nuvem".into()
        }
        fn title(&self) -> String {
            "Nuvem".into()
        }
        fn category(&self) -> ksni::Category {
            ksni::Category::ApplicationStatus
        }
        fn icon_name(&self) -> String {
            match cor(&self.estado) {
                Cor::Verde => "novasce-nuvem-ok",
                Cor::Laranja => "novasce-nuvem-problema",
                Cor::Neutra => "novasce-nuvem",
            }
            .into()
        }
        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip {
                title: dica(&self.estado, chrono::Utc::now()),
                ..Default::default()
            }
        }
        fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
            use ksni::menu::StandardItem;
            vec![
                StandardItem {
                    label: "Salvar agora".into(),
                    activate: Box::new(|t: &mut Self| {
                        let _ = t.avisar.send(());
                    }),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: "Abrir Documentos".into(),
                    activate: Box::new(|_| {
                        let _ = novasce_casa::abrir::rodar_solto(&[
                            "xdg-open",
                            &Caminhos::da_pessoa().documentos().to_string_lossy(),
                        ]);
                    }),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: "Abrir o brain".into(),
                    activate: Box::new(|_| {
                        let _ = novasce_casa::abrir::rodar_solto(&[
                            "flatpak",
                            "run",
                            "md.obsidian.Obsidian",
                        ]);
                    }),
                    ..Default::default()
                }
                .into(),
            ]
        }
    }

    pub fn rodar() {
        let c = Caminhos::da_pessoa();
        let casa = ConfigCasa::carregar();
        if conta_atual() == casa.sede.conta {
            return;
        }
        let maquina = SinalLocal::padrao()
            .maquina()
            .map(|m| nome_para_arquivo(&m.name))
            .unwrap_or_else(|_| {
                nome_para_arquivo(
                    &std::fs::read_to_string("/etc/hostname")
                        .unwrap_or_else(|_| "computador".into()),
                )
            });
        let (tx, rx) = mpsc::channel::<()>();
        let inicial: EstadoSync = json_atomico::ler(&estado::caminho(&c));
        let servico = ksni::TrayService::new(Nuvem {
            estado: inicial,
            avisar: tx.clone(),
        });
        let alca = servico.handle();
        servico.spawn();

        let tx_obs = tx.clone();
        let mut observador =
            notify::recommended_watcher(move |ev: notify::Result<notify::Event>| {
                if let Ok(ev) = ev {
                    if ev
                        .paths
                        .iter()
                        .any(|p| !p.components().any(|x| x.as_os_str() == ".git"))
                    {
                        let _ = tx_obs.send(());
                    }
                }
            })
            .ok();
        if let Some(o) = observador.as_mut() {
            use notify::Watcher;
            let _ = o.watch(&c.brain(), notify::RecursiveMode::Recursive);
            let _ = o.watch(&c.documentos(), notify::RecursiveMode::Recursive);
        }

        let docs = ConfigDocs {
            local: c.documentos(),
            remoto: documentos::remoto_de(&casa.google.pasta),
            maquina: maquina.clone(),
            filtros: c.estado_novasce().join("filtros-documentos.txt"),
            cache_bisync: cache_bisync_padrao(&c.casa),
        };
        loop {
            if rx.recv_timeout(Duration::from_secs(180)).is_ok() {
                std::thread::sleep(Duration::from_secs(20));
                while rx.try_recv().is_ok() {}
            }
            let agora = chrono::Utc::now().to_rfc3339();
            let mut e: EstadoSync = json_atomico::ler(&estado::caminho(&c));
            e.brain = do_resumo_brain(
                &sincronizar_brain(&c.brain(), &maquina, &Gitleaks, &agora),
                &agora,
            );
            if google_conectado() {
                let caminho = estado::caminho(&c);
                let mut parcial = e.clone();
                let r = documentos::rodar(&docs, |p| {
                    parcial.documentos = Item {
                        estado: Situacao::Subindo,
                        quando: None,
                        progresso: Some(p),
                        mensagem: None,
                    };
                    let _ = json_atomico::gravar(&caminho, &parcial);
                    let copia = parcial.clone();
                    alca.update(|t: &mut Nuvem| t.estado = copia);
                });
                e.documentos = match r {
                    Ok(()) => Item {
                        estado: Situacao::Salvo,
                        quando: Some(agora.clone()),
                        progresso: None,
                        mensagem: None,
                    },
                    Err(ErroDocs::SemRede) => Item {
                        estado: Situacao::Problema,
                        mensagem: Some("Sem internet. Os Documentos estão neste computador e sobem quando a conexão voltar.".into()),
                        ..Item::default()
                    },
                    Err(ErroDocs::PrecisaResync) => {
                        let _ = std::fs::remove_dir_all(&docs.cache_bisync);
                        Item {
                            estado: Situacao::Subindo,
                            mensagem: Some("Conferindo os Documentos com o Drive de novo.".into()),
                            ..Item::default()
                        }
                    }
                    Err(ErroDocs::SemRemoto) => Item::default(),
                    Err(ErroDocs::Falhou(m)) => Item {
                        estado: Situacao::Problema,
                        mensagem: Some(format!("Os Documentos não subiram. Nada foi apagado. {m}")),
                        ..Item::default()
                    },
                };
            } else {
                e.documentos = Item::default();
            }
            let _ = json_atomico::gravar(&estado::caminho(&c), &e);
            alca.update(|t: &mut Nuvem| t.estado = e.clone());
        }
    }

    fn google_conectado() -> bool {
        std::process::Command::new("rclone")
            .arg("listremotes")
            .output()
            .map(|s| {
                String::from_utf8_lossy(&s.stdout)
                    .lines()
                    .any(|l| l.trim() == "gdrive:")
            })
            .unwrap_or(false)
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    servico::rodar();
}
