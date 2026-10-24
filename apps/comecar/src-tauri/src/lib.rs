mod comandos;

use novasce_casa::{caminhos::Caminhos, chaveiro::ChaveiroSistema, config_casa::ConfigCasa, nucleo::{Config, Espaco}, sinal_local::SinalLocal};
use novasce_comecar::estado::{self, EstadoComecar};
use std::sync::Mutex;
use tauri::Manager;

pub struct Estado {
    pub caminhos: Caminhos,
    pub config: Config,
    pub casa: ConfigCasa,
    pub chaveiro: ChaveiroSistema,
    pub sinal: SinalLocal,
    pub comecar: Mutex<EstadoComecar>,
    pub fluxo_tenancy: Mutex<Option<String>>,
    pub fluxo_github: Mutex<Option<String>>,
    pub espacos: Mutex<Vec<Espaco>>,
    pub kit_url: Mutex<Option<String>>,
}

pub fn rodar() {
    // WebKitGTK com placa NVIDIA (a sede) abre em branco sem isto, como no Preparar do M1.
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    let caminhos = Caminhos::da_pessoa();
    let casa = ConfigCasa::carregar();
    if novasce_casa::sistema::conta_atual() == casa.sede.conta {
        return; // a conta Sede não tem Começar
    }
    let mut comecar = estado::ler(&caminhos);
    if std::env::args().any(|a| a == "--se-pendente") && !comecar.deve_abrir_sozinho() {
        return; // some quando tudo está feito; volta pela Ajuda
    }
    let sinal = SinalLocal::padrao();
    if !comecar.tenancy && !comecar.boas_vindas_vista {
        if let Some(b) = comandos::boas_vindas_pela_maquina(&sinal) {
            comecar.pessoa_nova = true;
            comecar.boas_vindas = Some(b);
        }
    }
    let config = Config::carregar().expect("config.toml da máquina");
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("principal") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .manage(Estado {
            chaveiro: ChaveiroSistema::da_pessoa(&caminhos),
            caminhos,
            config,
            casa,
            sinal,
            comecar: Mutex::new(comecar),
            fluxo_tenancy: Mutex::new(None),
            fluxo_github: Mutex::new(None),
            espacos: Mutex::new(Vec::new()),
            kit_url: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            comandos::estado_tela,
            comandos::tenancy_iniciar,
            comandos::tenancy_esperar,
            comandos::boas_vindas_vista,
            comandos::ia_instalar,
            comandos::ia_abrir_web,
            comandos::ia_concluir,
            comandos::ia_ligar_tenancy,
            comandos::github_iniciar,
            comandos::github_esperar,
            comandos::brain_criar,
            comandos::espacos_trazer,
            comandos::espacos_agora_nao,
            comandos::google_conectar,
            comandos::abrir_bitwarden,
            comandos::voz_testar,
            comandos::manha_definir,
            comandos::fim_visto,
            comandos::de_volta_concluir,
            comandos::abrir_url,
            comandos::abrir_tenancy,
            comandos::abrir_ajuda,
            comandos::abrir_ficha,
            comandos::abrir_wifi,
            comandos::copiar,
            comandos::fechar,
        ])
        .setup(|app| {
            if let Some(w) = app.get_webview_window("principal") {
                w.show()?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Começar");
}
