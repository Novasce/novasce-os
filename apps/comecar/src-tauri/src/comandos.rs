use crate::Estado;
use novasce_casa::abrir::{abrir_app, abrir_tenancy as abrir_no_tenancy, abrir_url as abrir_endereco, juntar, rodar_solto};
use novasce_casa::chaveiro::{Chaveiro, CONTA_GITHUB, CONTA_TENANCY};
use novasce_casa::descobertas::{self, Descobertas};
use novasce_casa::nucleo::{Comecar, ErroTenancy, Tenancy};
use novasce_casa::sinal_local::{ErroLocal, PedidoLocal, RespostaLocal, Saida, SinalLocal};
use novasce_casa::{movimento, preferencias};
use novasce_comecar::brain::{self, GitReal, OrigemBrain};
use novasce_comecar::espacos::{self, EspacoVisao};
use novasce_comecar::estado::{self, BoasVindas, EstadoComecar, Extras, Modo, Pessoa, Tela};
use novasce_comecar::github::{ErroGithub, Espera, GithubApi, GithubDevice};
use novasce_comecar::{google, ias::{self, Ia}};
use serde::Serialize;
use tauri::{Manager, State};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErroVisao {
    tipo: &'static str,
    mensagem: String,
}

impl ErroVisao {
    fn novo(tipo: &'static str, mensagem: impl Into<String>) -> Self {
        ErroVisao { tipo, mensagem: mensagem.into() }
    }
}

impl From<ErroLocal> for ErroVisao {
    fn from(e: ErroLocal) -> Self {
        match e {
            ErroLocal::SemRede(_) => ErroVisao::novo("sem-internet", "Sem internet. Nada do que você já fez se perdeu."),
            ErroLocal::SemServico(_) => ErroVisao::novo("sem-servico", "O serviço da máquina não respondeu. Reinicie e tente de novo."),
            ErroLocal::Recusado { codigo: 412, mensagem } => ErroVisao::novo("sem-permissao", mensagem),
            ErroLocal::Recusado { codigo: 403, .. } => ErroVisao::novo("recusado", "A entrada foi recusada no Tenancy. Nada foi conectado."),
            ErroLocal::Recusado { codigo: 410, .. } => ErroVisao::novo("recusado", "O código venceu ou já foi usado. Peça outro."),
            ErroLocal::Recusado { mensagem, .. } => ErroVisao::novo("recusado", mensagem),
            ErroLocal::Formato(m) => ErroVisao::novo("recusado", m),
        }
    }
}

impl From<ErroGithub> for ErroVisao {
    fn from(e: ErroGithub) -> Self {
        match e {
            ErroGithub::SemClientId => ErroVisao::novo("sem-configuracao", "O GitHub ainda não foi ligado ao Novasce OS pela Novasce. Dá para pular este passo e voltar depois."),
            ErroGithub::SemRede(_) => ErroVisao::novo("sem-internet", "Sem internet. Nada do que você já fez se perdeu."),
            ErroGithub::Recusado { codigo: 403 | 404, .. } => ErroVisao::novo("sem-permissao", "A sua conta do GitHub não tem acesso a isso. Peça ajuda e a Novasce libera."),
            ErroGithub::Recusado { mensagem, .. } => ErroVisao::novo("recusado", mensagem),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodigoVisao {
    codigo: String,
    url: String,
    intervalo: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoTela {
    tela: Tela,
    passos_feitos: u8,
    pessoa: Option<Pessoa>,
    boas_vindas: Option<BoasVindas>,
    antes_do_passo1: bool,
    espacos: Vec<EspacoVisao>,
    kit_disponivel: bool,
    extras: Extras,
    manha: bool,
    descobertas: Descobertas,
    ias: Vec<String>,
    github_configurado: bool,
    github_login: Option<String>,
    brain_repo: Option<String>,
    movimento: f32,
    sincronizar: Option<serde_json::Value>,
    google_conectado: bool,
    voz_qualquer_campo: bool,
}

fn mudar(s: &Estado, f: impl FnOnce(&mut EstadoComecar)) -> EstadoComecar {
    let mut e = s.comecar.lock().unwrap();
    f(&mut e);
    let _ = estado::gravar(&s.caminhos, &e);
    e.clone()
}

/// Contrato v2: `owner.isNew` em `GET /os/machine`, antes do passo 1. Só o primeiro nome vem daí:
/// a área e a primeira tarefa chegam com o token da pessoa (`carregar_comecar`). Tenancy antigo sem o campo: nada.
pub fn boas_vindas_pela_maquina(sinal: &SinalLocal) -> Option<BoasVindas> {
    match sinal.pedir(&PedidoLocal::Maquina).ok()? {
        RespostaLocal::Ok { dados } => estado::boas_vindas_de(&dados["owner"]),
        _ => None,
    }
}

/// GET /os/me/comecar com o token da pessoa (contrato v2): espaços, kit (`downloadPath`, ou `/os/me/kit`),
/// pessoa nova com área e primeira tarefa, troca de notebook (`returning`) e a assinatura de e-mail.
fn carregar_comecar(s: &Estado) -> Result<(), ErroVisao> {
    let token = s.chaveiro.ler(CONTA_TENANCY).ok_or_else(|| ErroVisao::novo("sem-permissao", "Entre no Tenancy primeiro."))?;
    let t = Tenancy::new(&s.config.tenancy_api, Some(token));
    let bruto: serde_json::Value = t.get("/os/me/comecar").map_err(|e| match e {
        ErroTenancy::SemRede(_) => ErroVisao::novo("sem-internet", "Sem internet. Nada do que você já fez se perdeu."),
        outro => ErroVisao::novo("recusado", format!("{outro:?}")),
    })?;
    let c: Comecar = serde_json::from_value(bruto.clone()).map_err(|e| ErroVisao::novo("recusado", e.to_string()))?;
    *s.espacos.lock().unwrap() = c.spaces.clone();
    let kit = &bruto["kit"];
    *s.kit_url.lock().unwrap() =
        (kit["available"].as_bool() == Some(true)).then(|| juntar(&s.config.tenancy_api, kit["downloadPath"].as_str().unwrap_or("/os/me/kit")));
    let _ = google::salvar_assinatura(&s.caminhos.documentos(), &c.email_signature_html);
    let boas = estado::boas_vindas_de(&bruto);
    let voltando = bruto.get("returning").and_then(|v| v.as_bool()) == Some(true);
    mudar(s, |e| {
        if let Some(b) = boas {
            // Antes do passo 1 só se sabe que a pessoa é nova (owner.isNew). Se as boas-vindas já foram
            // vistas sem a área e a primeira tarefa, elas voltam uma vez, completas, antes do passo 2.
            let tinha = e.boas_vindas.as_ref().is_some_and(|x| x.area.is_some() || x.primeira_tarefa.is_some());
            let tem = b.area.is_some() || b.primeira_tarefa.is_some();
            if !e.boas_vindas_vista || (!tinha && tem) {
                e.pessoa_nova = true;
                e.boas_vindas = Some(b);
                e.boas_vindas_vista = false;
            }
        }
        if voltando && !e.github {
            e.modo = Modo::DeVolta;
        }
    });
    Ok(())
}

#[tauri::command(async)]
pub fn estado_tela(s: State<Estado>) -> EstadoTela {
    let precisa = s.comecar.lock().unwrap().tenancy && s.espacos.lock().unwrap().is_empty();
    if precisa {
        let _ = carregar_comecar(&s);
    }
    let e = s.comecar.lock().unwrap().clone();
    let sincronizar = std::fs::read(s.caminhos.estado_novasce().join("sincronizar.json")).ok().and_then(|b| serde_json::from_slice(&b).ok());
    EstadoTela {
        tela: e.tela(),
        passos_feitos: e.passos_feitos(),
        antes_do_passo1: !e.tenancy,
        espacos: espacos::visoes(&s.espacos.lock().unwrap(), &s.caminhos.espacos()),
        kit_disponivel: s.kit_url.lock().unwrap().is_some(),
        manha: preferencias::ler(&s.caminhos).manha,
        descobertas: descobertas::ler(&s.caminhos),
        github_configurado: !s.casa.github.client_id.trim().is_empty(),
        movimento: movimento::fator_atual(&s.caminhos),
        sincronizar,
        google_conectado: google::conectado(),
        voz_qualquer_campo: s.casa.voz.qualquer_campo,
        pessoa: e.pessoa,
        boas_vindas: e.boas_vindas,
        extras: e.extras,
        ias: e.ias,
        github_login: e.github_login,
        brain_repo: e.brain_repo,
    }
}

#[tauri::command(async)]
pub fn tenancy_iniciar(s: State<Estado>) -> Result<CodigoVisao, ErroVisao> {
    let i = s.sinal.entrar()?;
    *s.fluxo_tenancy.lock().unwrap() = Some(i.device_code.clone());
    Ok(CodigoVisao { codigo: i.user_code, url: i.verification_uri, intervalo: i.interval.max(2) })
}

#[tauri::command(async)]
pub fn tenancy_esperar(s: State<Estado>) -> Result<&'static str, ErroVisao> {
    let dc = s.fluxo_tenancy.lock().unwrap().clone().ok_or_else(|| ErroVisao::novo("recusado", "Peça um código novo."))?;
    match s.sinal.entrar_token(&dc)? {
        Saida::Pendente => Ok("pendente"),
        Saida::Pronto(t) => {
            s.chaveiro.salvar(CONTA_TENANCY, &t.user_token).map_err(|m| ErroVisao::novo("recusado", m))?;
            mudar(&s, |e| {
                e.tenancy = true;
                e.pessoa = Some(Pessoa { first_name: t.user.first_name.clone(), name: t.user.name.clone(), email: t.user.email.clone() });
            });
            let _ = carregar_comecar(&s);
            Ok("pronto")
        }
    }
}

#[tauri::command(async)]
pub fn boas_vindas_vista(s: State<Estado>) {
    mudar(&s, |e| e.boas_vindas_vista = true);
}

fn ia(id: &str) -> Result<Ia, ErroVisao> {
    Ia::de_id(id).ok_or_else(|| ErroVisao::novo("recusado", "IA desconhecida."))
}

#[tauri::command(async)]
pub fn ia_instalar(s: State<Estado>, ia_id: String) -> Result<(), ErroVisao> {
    let i = ia(&ia_id)?;
    ias::instalar(i, &s.caminhos.casa)
        .map_err(|m| ErroVisao::novo("recusado", format!("O {} não foi instalado. Nada mudou no computador. {m}", i.nome_cli())))
}

#[tauri::command(async)]
pub fn ia_abrir_web(ia_id: String) -> Result<(), ErroVisao> {
    let i = ia(&ia_id)?;
    abrir_app(i.desktop(), i.url_web()).map_err(|m| ErroVisao::novo("recusado", m))
}

#[tauri::command(async)]
pub fn ia_concluir(s: State<Estado>, ias_marcadas: Vec<String>) {
    mudar(&s, |e| {
        e.ia = true;
        e.ias = ias_marcadas;
    });
}

#[tauri::command(async)]
pub fn ia_ligar_tenancy(s: State<Estado>, ia_id: String, chave: String) -> Result<(), ErroVisao> {
    let url = juntar(&s.config.tenancy_web, "/mcp");
    let r = match ia(&ia_id)? {
        Ia::Claude => ias::ligar_claude(&s.caminhos.casa, &url, chave.trim()),
        Ia::Gemini => ias::ligar_gemini(&s.caminhos.casa.join(".gemini/settings.json"), &url, chave.trim()).map_err(|e| e.to_string()),
        Ia::Chatgpt => Err("O Codex ainda não liga ao Tenancy por aqui. Use o Claude ou o Gemini.".into()),
    };
    r.map_err(|m| ErroVisao::novo("recusado", format!("A IA não foi ligada ao Tenancy. {m}")))
}

#[tauri::command(async)]
pub fn github_iniciar(s: State<Estado>) -> Result<CodigoVisao, ErroVisao> {
    let c = GithubDevice::do_github(&s.casa.github.client_id).pedir_codigo()?;
    *s.fluxo_github.lock().unwrap() = Some(c.device_code.clone());
    let _ = abrir_endereco(&c.verification_uri);
    Ok(CodigoVisao { codigo: c.user_code, url: c.verification_uri, intervalo: c.interval.max(5) })
}

#[tauri::command(async)]
pub fn github_esperar(s: State<Estado>) -> Result<&'static str, ErroVisao> {
    let dc = s.fluxo_github.lock().unwrap().clone().ok_or_else(|| ErroVisao::novo("recusado", "Peça um código novo."))?;
    match GithubDevice::do_github(&s.casa.github.client_id).esperar(&dc)? {
        Espera::Pendente => Ok("pendente"),
        Espera::Devagar => Ok("devagar"),
        Espera::Expirou => Err(ErroVisao::novo("recusado", "O código venceu. Nada foi conectado.")),
        Espera::Negado => Err(ErroVisao::novo("recusado", "A conexão foi recusada no GitHub. Nada foi conectado.")),
        Espera::Pronto(token) => {
            s.chaveiro.salvar(CONTA_GITHUB, &token).map_err(|m| ErroVisao::novo("recusado", m))?;
            let login = GithubApi::do_github(token).login()?;
            let pessoa = s.comecar.lock().unwrap().pessoa.clone().unwrap_or_default();
            let _ = brain::configurar_git(&pessoa.name, &pessoa.email);
            mudar(&s, |e| e.github_login = Some(login));
            Ok("pronto")
        }
    }
}

#[tauri::command(async)]
pub fn brain_criar(s: State<Estado>) -> Result<OrigemBrain, ErroVisao> {
    let token = s.chaveiro.ler(CONTA_GITHUB).ok_or_else(|| ErroVisao::novo("sem-permissao", "Conecte o GitHub primeiro."))?;
    let login = s.comecar.lock().unwrap().github_login.clone().ok_or_else(|| ErroVisao::novo("sem-permissao", "Conecte o GitHub primeiro."))?;
    let destino = s.caminhos.brain();
    let origem = brain::preparar_brain(&GithubApi::do_github(token), &s.casa.brain.modelo, &login, &destino, &GitReal, 1000)
        .map_err(|e| ErroVisao::novo("recusado", format!("O brain não foi criado. Nada mudou no seu GitHub. {e}")))?;
    let _ = brain::registrar_no_obsidian(&brain::caminho_config_obsidian(&s.caminhos.casa), &destino, chrono::Utc::now().timestamp_millis());
    let espacos_lista = s.espacos.lock().unwrap().clone();
    if origem == OrigemBrain::JaExistia {
        let membros: Vec<String> = espacos_lista.iter().filter(|e| e.member).map(|e| e.slug.clone()).collect();
        espacos::trazer(&espacos_lista, &membros, &s.caminhos.espacos(), &GitReal);
    }
    mudar(&s, |e| {
        e.github = true;
        e.brain_repo = Some(brain::nome_do_brain(&login));
    });
    let _ = rodar_solto(&["systemctl", "--user", "restart", "novasce-sincronizar.service"]);
    Ok(origem)
}

#[tauri::command(async)]
pub fn espacos_trazer(s: State<Estado>, slugs: Vec<String>, kit: bool) -> Result<(), ErroVisao> {
    let lista = s.espacos.lock().unwrap().clone();
    for (slug, r) in espacos::trazer(&lista, &slugs, &s.caminhos.espacos(), &GitReal) {
        if r.is_err() {
            let nome = lista.iter().find(|e| e.slug == slug).map(|e| e.name.clone()).unwrap_or(slug);
            return Err(ErroVisao::novo("sem-permissao", format!("O {nome} não veio. A sua conta do GitHub ainda não tem acesso a ele. Peça ajuda e a Novasce libera.")));
        }
    }
    if kit {
        let url = s.kit_url.lock().unwrap().clone();
        let token = s.chaveiro.ler(CONTA_TENANCY);
        if let (Some(u), Some(t)) = (url, token) {
            espacos::baixar_kit(&u, &t, &s.caminhos.brain().join("kit-da-frente"))
                .map_err(|m| ErroVisao::novo("recusado", format!("O kit não veio. Os espaços já estão aqui. {m}")))?;
        }
    }
    mudar(&s, |e| e.espacos_vistos = true);
    Ok(())
}

#[tauri::command(async)]
pub fn espacos_agora_nao(s: State<Estado>) {
    mudar(&s, |e| e.espacos_vistos = true);
}

#[tauri::command(async)]
pub fn google_conectar(s: State<Estado>) -> Result<(), ErroVisao> {
    google::conectar(&s.casa.google, &s.caminhos.config).map_err(|m| ErroVisao::novo("recusado", m))?;
    mudar(&s, |e| e.extras.nuvem = true);
    let _ = rodar_solto(&["systemctl", "--user", "restart", "novasce-sincronizar.service"]);
    Ok(())
}

#[tauri::command(async)]
pub fn abrir_bitwarden(s: State<Estado>) -> Result<(), ErroVisao> {
    mudar(&s, |e| e.extras.senhas = true);
    rodar_solto(&["flatpak", "run", "com.bitwarden.desktop"]).map_err(|m| ErroVisao::novo("recusado", m))
}

#[tauri::command(async)]
pub fn voz_testar(s: State<Estado>) -> Result<(), ErroVisao> {
    mudar(&s, |e| e.extras.voz = true);
    rodar_solto(&["novasce-voz", "--abrir"]).map_err(|m| ErroVisao::novo("recusado", m))
}

#[tauri::command(async)]
pub fn manha_definir(s: State<Estado>, ligada: bool) -> Result<(), ErroVisao> {
    preferencias::gravar(&s.caminhos, &preferencias::Preferencias { manha: ligada }).map_err(|e| ErroVisao::novo("recusado", e.to_string()))
}

#[tauri::command(async)]
pub fn fim_visto(s: State<Estado>) {
    let e = mudar(&s, |e| e.fim_visto = true);
    if e.tudo_feito() {
        let _ = google::tirar_do_dock();
    }
}

#[tauri::command(async)]
pub fn de_volta_concluir(s: State<Estado>) {
    mudar(&s, |e| {
        e.ia = true;
        e.espacos_vistos = true;
        e.fim_visto = true;
    });
    let _ = google::tirar_do_dock();
}

#[tauri::command(async)]
pub fn abrir_url(url: String) -> Result<(), ErroVisao> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(ErroVisao::novo("recusado", "Endereço inválido."));
    }
    abrir_endereco(&url).map_err(|m| ErroVisao::novo("recusado", m))
}

#[tauri::command(async)]
pub fn abrir_tenancy(s: State<Estado>, caminho: String) -> Result<(), ErroVisao> {
    abrir_no_tenancy(&s.config.tenancy_web, &caminho).map_err(|m| ErroVisao::novo("recusado", m))
}

#[tauri::command(async)]
pub fn abrir_ajuda() -> Result<(), ErroVisao> {
    rodar_solto(&["novasce-ajuda", "--central", "--tela", "comecar"]).map_err(|m| ErroVisao::novo("recusado", m))
}

#[tauri::command(async)]
pub fn abrir_ficha(id: String) -> Result<(), ErroVisao> {
    rodar_solto(&["novasce-ajuda", "--ficha", &id]).map_err(|m| ErroVisao::novo("recusado", m))
}

#[tauri::command(async)]
pub fn abrir_wifi() -> Result<(), ErroVisao> {
    rodar_solto(&["kcmshell6", "kcm_networkmanagement"]).map_err(|m| ErroVisao::novo("recusado", m))
}

#[tauri::command(async)]
pub fn copiar(texto: String) -> Result<(), ErroVisao> {
    use std::io::Write;
    let mut filho = std::process::Command::new("wl-copy").stdin(std::process::Stdio::piped()).spawn().map_err(|e| ErroVisao::novo("recusado", e.to_string()))?;
    filho.stdin.take().map(|mut i| i.write_all(texto.as_bytes()));
    Ok(())
}

#[tauri::command]
pub fn fechar(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("principal") {
        let _ = w.hide();
    }
    app.exit(0);
}
