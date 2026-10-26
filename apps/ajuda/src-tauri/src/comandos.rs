use crate::Estado;
use base64::Engine;
use novasce_casa::chaveiro::{Chaveiro, CONTA_TENANCY};
use novasce_casa::sistema::{self, conta_atual, papel, Papel};
use novasce_casa::{abrir, descobertas, flatpak, movimento as mov, versoes};
use novasce_chamados::dados_maquina::{montar, Fatos};
use novasce_chamados::esqueci;
use novasce_chamados::fila::Fila;
use novasce_chamados::pedido::{envio_para, enviar_ou_guardar, esvaziar_fila, Pedido, Resultado};
use novasce_voz::gravacao::Gravador;
use novasce_voz::vocabulario::{aplicar, carregar};
use novasce_voz::whisper::{threads_padrao, transcrever, Whisper};
use serde::Serialize;
use tauri::{Manager, State};

pub fn reduzir_png(png: &[u8], largura_max: u32) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(png).map_err(|e| e.to_string())?;
    let img = if img.width() > largura_max { img.resize(largura_max, u32::MAX, image::imageops::FilterType::Triangle) } else { img };
    let mut saida = std::io::Cursor::new(Vec::new());
    img.write_to(&mut saida, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    Ok(saida.into_inner())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparoPedido {
    linha_maquina: String,
    miniatura: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultadoVisao {
    tipo: &'static str,
    numero: Option<String>,
    mensagem: Option<String>,
}

fn fatos(s: &Estado, app: &str) -> Fatos {
    let status = sistema::status_ostree().unwrap_or_default();
    let imagem = status.atual.as_ref().and_then(|a| a.versao.clone());
    let maquina = s.sinal.maquina().ok();
    Fatos {
        versao_visivel: imagem.clone().unwrap_or_else(|| "sem versão".into()),
        imagem,
        nome_maquina: maquina.as_ref().map(|m| m.name.clone()),
        papel: match papel() { Papel::Sede => "sede", Papel::Notebook => "notebook", Papel::Desconhecido => "desconhecido" }.into(),
        dias_desde_atualizacao: status.atual.as_ref().map(|a| sistema::dias_desde(a.timestamp, sistema::agora_unix())),
        cifrado: sistema::disco_cifrado_agora(),
        app: app.into(),
    }
}

#[tauri::command]
pub fn tela_inicial(s: State<Estado>) -> crate::argumentos::TelaInicial {
    s.inicial.lock().unwrap().clone()
}

#[tauri::command(async)]
pub fn preparar_pedido(s: State<Estado>) -> PreparoPedido {
    let app = s.inicial.lock().unwrap().tela.clone();
    let miniatura = s.print.lock().unwrap().as_deref().and_then(|p| reduzir_png(p, 360).ok()).map(|b| format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(b)));
    PreparoPedido { linha_maquina: montar(&fatos(&s, &app)).linha, miniatura }
}

fn envio(s: &Estado) -> Box<dyn novasce_chamados::pedido::Envio> {
    envio_para(&conta_atual(), &s.casa.sede.conta, s.chaveiro.ler(CONTA_TENANCY), &s.config.tenancy_api)
}

#[tauri::command(async)]
pub fn enviar_pedido(s: State<Estado>, texto: String, com_print: bool) -> ResultadoVisao {
    let app = s.inicial.lock().unwrap().tela.clone();
    let mut info = montar(&fatos(&s, &app)).json;
    info["conta"] = serde_json::Value::String(conta_atual());
    let print = if com_print { s.print.lock().unwrap().clone() } else { None };
    let pedido = Pedido { texto: texto.trim().into(), print, dados_maquina: info };
    let r = enviar_ou_guardar(envio(&s).as_ref(), &Fila::da_pessoa(&s.caminhos), &pedido);
    let _ = descobertas::marcar(&s.caminhos, descobertas::Descoberta::PediuAjuda);
    match r {
        Resultado::Enviado { numero, .. } => {
            avisar(&format!("Chamado aberto{}", numero.as_ref().map(|n| format!(" · {n}")).unwrap_or_default()), "A resposta chega aqui e no Tenancy. Você não precisa fazer mais nada.");
            ResultadoVisao { tipo: "enviado", numero, mensagem: None }
        }
        Resultado::Guardado { .. } => ResultadoVisao { tipo: "guardado", numero: None, mensagem: None },
        Resultado::Recusado { mensagem } => ResultadoVisao { tipo: "recusado", numero: None, mensagem: Some(format!("O chamado não foi aberto. {mensagem}")) },
    }
}

fn avisar(titulo: &str, corpo: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = notify_rust::Notification::new().appname("Novasce OS").icon("novasce-ajuda").summary(titulo).body(corpo).show();
    }
    let _ = (titulo, corpo);
}

#[tauri::command(async)]
pub fn tentar_agora(s: State<Estado>) -> usize {
    esvaziar_fila(envio(&s).as_ref(), &Fila::da_pessoa(&s.caminhos)).len()
}

#[tauri::command(async)]
pub fn ditar_iniciar(s: State<Estado>) -> Result<(), String> {
    let g = Gravador::iniciar().map_err(|_| "Nenhum microfone encontrado. Confira o som nas Configurações.".to_string())?;
    *s.gravador.lock().unwrap() = Some(g);
    Ok(())
}

#[tauri::command]
pub fn ditar_nivel(s: State<Estado>) -> f32 {
    s.gravador.lock().unwrap().as_ref().map(|g| g.nivel()).unwrap_or(0.0)
}

#[tauri::command(async)]
pub fn ditar_parar(s: State<Estado>) -> Result<String, String> {
    let g = s.gravador.lock().unwrap().take().ok_or("Nada estava sendo ouvido.")?;
    let v = &s.casa.voz;
    let modelo = if std::path::Path::new(&v.modelo).exists() { v.modelo.clone() } else { v.modelo_reserva.clone() };
    let w = Whisper { binario: v.whisper.clone().into(), modelo: modelo.into(), threads: threads_padrao() };
    let voc = carregar(&v.vocabulario, &s.caminhos.config_novasce().join("voz-vocabulario.toml"));
    let texto = transcrever(&w, &g.parar(), &voc.prompt, &s.caminhos.runtime_novasce()).map_err(|e| e.texto().to_string())?;
    let _ = descobertas::marcar(&s.caminhos, descobertas::Descoberta::Falou);
    Ok(aplicar(&voc, &texto))
}

#[tauri::command(async)]
pub fn recomendados_instalados(ids: Vec<String>) -> Vec<String> {
    flatpak::instalados(&flatpak::listar(), &ids)
}

#[tauri::command(async)]
pub fn instalar_recomendado(id: String) -> Result<(), String> {
    flatpak::instalar(&id)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EsqueciVisao {
    svg: String,
    titulo: String,
    linha: String,
}

#[tauri::command(async)]
pub fn esqueci(s: State<Estado>) -> Result<EsqueciVisao, String> {
    let m = s.sinal.maquina().map_err(|_| "Esta máquina ainda não foi registrada. Fale com o Eduardo.".to_string())?;
    let (titulo, linha) = esqueci::textos(&m.name, m.owner.as_ref().map(|o| o.first_name.as_str()));
    Ok(EsqueciVisao { svg: esqueci::qr_svg(&esqueci::url_esqueci(&s.config.tenancy_web, &m.id), 156)?, titulo, linha })
}

#[derive(Serialize)]
pub struct NovidadeVisao {
    versao: String,
    linha: String,
}

#[tauri::command]
pub fn novidades(s: State<Estado>) -> Vec<NovidadeVisao> {
    let notas = versoes::ler_local(&s.casa.atualizacao.notas_local);
    let atual = sistema::status_ostree().ok().and_then(|st| st.atual.and_then(|a| a.versao)).unwrap_or_else(|| "9999".into());
    versoes::recentes(&notas, &atual, 3).into_iter().map(|v| NovidadeVisao { versao: v.versao.clone(), linha: v.linha.clone() }).collect()
}

#[tauri::command]
pub fn abrir_pedir(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("pedir") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

#[tauri::command(async)]
pub fn abrir_comecar() -> Result<(), String> {
    abrir::rodar_solto(&["novasce-comecar"])
}

#[tauri::command(async)]
pub fn abrir_url(s: State<Estado>, url: String) -> Result<(), String> {
    if url.starts_with('/') {
        return abrir::abrir_tenancy(&s.config.tenancy_web, &url);
    }
    if !url.starts_with("https://") {
        return Err("Endereço inválido.".into());
    }
    abrir::abrir_url(&url)
}

#[tauri::command]
pub fn fechar_janela(app: tauri::AppHandle, rotulo: String) {
    if let Some(w) = app.get_webview_window(&rotulo) {
        let _ = w.hide();
    }
    if app.webview_windows().values().all(|w| !w.is_visible().unwrap_or(false)) {
        app.exit(0);
    }
}

#[tauri::command]
pub fn movimento(s: State<Estado>) -> f32 {
    mov::fator_atual(&s.caminhos)
}
