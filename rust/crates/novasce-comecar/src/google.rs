//! Extras do fim: Google Drive (rclone), assinatura de e-mail e sair do dock.
//!
//! Segredo do Google (client secret e token) nunca vai na linha de comando: sem `hidepid`, qualquer
//! conta da máquina lê /proc/<pid>/cmdline, e a sede tem várias contas. O cliente próprio vai pelo
//! ambiente do filho (só o dono do processo lê /proc/<pid>/environ) e o remoto é escrito direto no
//! rclone.conf da pessoa, com permissão 0600, como o próprio rclone faz.
use base64::Engine;
use novasce_casa::arquivo_privado;
pub use novasce_casa::config_casa::SecaoGoogle;
use std::path::{Path, PathBuf};
use std::process::Command;

const ARQUIVO_CIFRADO: &str = "# Encrypted rclone configuration File";

fn cliente_proprio(g: &SecaoGoogle) -> bool {
    !g.client_id.is_empty() && !g.client_secret.is_empty()
}

pub fn args_autorizar() -> Vec<String> {
    ["rclone", "authorize", "drive"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// O cliente próprio vai para o `rclone authorize` pelo ambiente (o rclone lê `RCLONE_DRIVE_*`),
/// no lugar dos dois argumentos que ele aceita na linha de comando.
pub fn env_autorizar(g: &SecaoGoogle) -> Vec<(&'static str, String)> {
    if cliente_proprio(g) {
        vec![
            ("RCLONE_DRIVE_CLIENT_ID", g.client_id.clone()),
            ("RCLONE_DRIVE_CLIENT_SECRET", g.client_secret.clone()),
        ]
    } else {
        vec![]
    }
}

pub fn extrair_token(saida: &str) -> Option<String> {
    let inicio = saida.find("--->")? + 4;
    let fim = saida[inicio..].find("<---")? + inicio;
    let bloco = saida[inicio..fim].trim();
    if bloco.starts_with('{') {
        return Some(bloco.to_string());
    }
    let motores = [
        base64::engine::general_purpose::URL_SAFE_NO_PAD,
        base64::engine::general_purpose::URL_SAFE,
        base64::engine::general_purpose::STANDARD,
    ];
    for m in motores {
        if let Ok(b) = m.decode(bloco) {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&b) {
                if let Some(t) = v.get("token").and_then(|t| t.as_str()) {
                    return Some(t.to_string());
                }
            }
        }
    }
    None
}

/// `rclone config file`: pergunta ao rclone onde fica o rclone.conf (ele respeita RCLONE_CONFIG e o XDG).
pub fn args_onde_fica_o_conf() -> Vec<String> {
    ["rclone", "config", "file"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// A última linha absoluta da saída do `rclone config file`, exista o arquivo ou não.
pub fn extrair_caminho_conf(saida: &str) -> Option<PathBuf> {
    saida
        .lines()
        .map(str::trim)
        .rfind(|l| l.starts_with('/'))
        .map(PathBuf::from)
}

/// Monta o comando a partir de um dos `args_*` (o primeiro item é sempre "rclone"). `rclone` é o
/// programa e seus argumentos fixos: na casa é só `["rclone"]`; o teste põe um rclone de mentira.
fn comando(rclone: &[&str], argv: &[String]) -> Command {
    let mut c = Command::new(rclone[0]);
    c.args(&rclone[1..]).args(&argv[1..]);
    c
}

fn caminho_conf(rclone: &[&str], config: &Path) -> PathBuf {
    comando(rclone, &args_onde_fica_o_conf())
        .output()
        .ok()
        .filter(|s| s.status.success())
        .and_then(|s| extrair_caminho_conf(&String::from_utf8_lossy(&s.stdout)))
        .unwrap_or_else(|| config.join("rclone/rclone.conf"))
}

fn linha_unica(chave: &str, valor: &str) -> Result<(), String> {
    if valor.contains(['\n', '\r']) {
        Err(format!("{chave} com quebra de linha"))
    } else {
        Ok(())
    }
}

/// A seção `[gdrive]` do rclone.conf. O token fica como o rclone devolveu; só um JSON espalhado em
/// várias linhas é compactado, porque o conf guarda cada valor numa linha.
pub fn secao_remoto(g: &SecaoGoogle, token: &str) -> Result<String, String> {
    let token = token.trim();
    let token = match serde_json::from_str::<serde_json::Value>(token) {
        Ok(v) if token.contains(['\n', '\r']) => v.to_string(),
        _ => token.to_string(),
    };
    linha_unica("token", &token)?;
    let mut s = format!("[gdrive]\ntype = drive\nscope = drive\ntoken = {token}\n");
    if cliente_proprio(g) {
        linha_unica("client_id", &g.client_id)?;
        linha_unica("client_secret", &g.client_secret)?;
        s.push_str(&format!(
            "client_id = {}\nclient_secret = {}\n",
            g.client_id, g.client_secret
        ));
    }
    Ok(s)
}

/// Troca só a seção `[gdrive]` e deixa o resto do rclone.conf como está.
pub fn mesclar_remoto(conf: &str, secao: &str) -> String {
    let mut resto: Vec<&str> = vec![];
    let mut no_gdrive = false;
    for linha in conf.lines() {
        let t = linha.trim();
        if t.starts_with('[') && t.ends_with(']') {
            no_gdrive = t == "[gdrive]";
        }
        if !no_gdrive {
            resto.push(linha);
        }
    }
    while resto.last().is_some_and(|l| l.trim().is_empty()) {
        resto.pop();
    }
    if resto.is_empty() {
        secao.to_string()
    } else {
        format!("{}\n\n{secao}", resto.join("\n"))
    }
}

/// Escreve o remoto `gdrive` no rclone.conf, 0600. Recusa arquivo cifrado: não dá para mesclar sem a senha.
pub fn gravar_remoto(conf: &Path, g: &SecaoGoogle, token: &str) -> Result<(), String> {
    let atual = match std::fs::read_to_string(conf) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.to_string()),
    };
    if atual.starts_with(ARQUIVO_CIFRADO) {
        return Err("O rclone.conf está cifrado.".into());
    }
    let novo = mesclar_remoto(&atual, &secao_remoto(g, token)?);
    arquivo_privado::gravar(conf, novo.as_bytes()).map_err(|e| e.to_string())
}

pub fn remoto_configurado(listremotes: &str) -> bool {
    listremotes.lines().any(|l| l.trim() == "gdrive:")
}

/// Abre o navegador para a pessoa entrar no Google e cria o remoto. Bloqueia até ela terminar.
/// `config` é a pasta de configuração da pessoa (XDG), usada só se o rclone não disser onde guarda o dele.
pub fn conectar(g: &SecaoGoogle, config: &Path) -> Result<(), String> {
    conectar_com(&["rclone"], g, config)
}

fn conectar_com(rclone: &[&str], g: &SecaoGoogle, config: &Path) -> Result<(), String> {
    let s = comando(rclone, &args_autorizar())
        .envs(env_autorizar(g))
        .output()
        .map_err(|e| e.to_string())?;
    let texto = format!(
        "{}{}",
        String::from_utf8_lossy(&s.stdout),
        String::from_utf8_lossy(&s.stderr)
    );
    let token =
        extrair_token(&texto).ok_or("O Google não devolveu a autorização. Nada foi conectado.")?;
    gravar_remoto(&caminho_conf(rclone, config), g, &token)
        .map_err(|_| "A conexão com o Google não foi salva. Nada foi enviado.".to_string())
}

pub fn conectado() -> bool {
    Command::new("rclone")
        .arg("listremotes")
        .output()
        .map(|s| remoto_configurado(&String::from_utf8_lossy(&s.stdout)))
        .unwrap_or(false)
}

pub fn salvar_assinatura(documentos: &Path, html: &str) -> std::io::Result<PathBuf> {
    let pasta = documentos.join("Novasce");
    std::fs::create_dir_all(&pasta)?;
    let p = pasta.join("Assinatura de e-mail.html");
    std::fs::write(&p, html)?;
    Ok(p)
}

const SCRIPT_DOCK: &str = r#"for (const p of panels()) { for (const w of p.widgets("org.kde.plasma.icontasks")) { w.currentConfigGroup = ["General"]; const l = String(w.readConfig("launchers", "")); const n = l.split(",").filter((x) => x && x.indexOf("novasce-comecar.desktop") < 0).join(","); if (n !== l) w.writeConfig("launchers", n); } }"#;

pub fn args_tirar_do_dock() -> Vec<String> {
    [
        "busctl",
        "--user",
        "call",
        "org.kde.plasmashell",
        "/PlasmaShell",
        "org.kde.PlasmaShell",
        "evaluateScript",
        "s",
        SCRIPT_DOCK,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

pub fn tirar_do_dock() -> Result<(), String> {
    let a = args_tirar_do_dock();
    Command::new(&a[0])
        .args(&a[1..])
        .output()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod testes {
    use super::*;
    use base64::Engine;

    fn cfg(id: &str, segredo: &str) -> SecaoGoogle {
        SecaoGoogle {
            client_id: id.into(),
            client_secret: segredo.into(),
            pasta: "Novasce OS/Documentos".into(),
        }
    }

    const SEGREDO: &str = "segredo-do-cliente-de-teste";
    const TOKEN: &str = r#"{"access_token":"acesso-de-teste","token_type":"Bearer","refresh_token":"refresco-de-teste","expiry":"2026-10-23T15:00:00Z"}"#;

    #[test]
    fn autorizar_leva_o_cliente_pelo_ambiente_e_nunca_pelo_argv() {
        let g = cfg("id-de-teste", SEGREDO);
        assert_eq!(args_autorizar(), vec!["rclone", "authorize", "drive"]);
        assert_eq!(
            env_autorizar(&g),
            vec![
                ("RCLONE_DRIVE_CLIENT_ID", "id-de-teste".to_string()),
                ("RCLONE_DRIVE_CLIENT_SECRET", SEGREDO.to_string())
            ]
        );
        assert!(env_autorizar(&cfg("", "")).is_empty());
        assert!(env_autorizar(&cfg("id", "")).is_empty());
    }

    #[test]
    fn token_em_json_ou_em_base64() {
        let json = r#"{"access_token":"a","token_type":"Bearer","refresh_token":"r","expiry":"2026-10-23T15:00:00Z"}"#;
        let saida = format!(
            "Abra o navegador\nPaste the following into your remote machine --->\n{json}\n<---End paste\n"
        );
        assert_eq!(extrair_token(&saida).as_deref(), Some(json));
        let bloco = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(serde_json::json!({ "token": json }).to_string());
        let saida2 =
            format!("Paste the following into your remote machine --->\n{bloco}\n<---End paste\n");
        assert_eq!(extrair_token(&saida2).as_deref(), Some(json));
        assert_eq!(extrair_token("nada"), None);
    }

    /// Um rclone de mentira: anota o argv e o ambiente de cada chamada, responde ao `authorize`
    /// com o token e ao `config file` com o caminho do conf. É o `conectar` inteiro, sem o Google.
    fn rclone_de_mentira(pasta: &Path) -> PathBuf {
        let script = pasta.join("rclone-de-mentira.sh");
        let corpo = format!(
            r#"echo "$@" >> "{d}/argv.txt"
env | grep '^RCLONE_' >> "{d}/ambiente.txt"
case "$1 $2" in
  "authorize drive") printf 'Abra o navegador\nPaste the following into your remote machine --->\n%s\n<---End paste\n' '{TOKEN}' ;;
  "config file") printf 'Configuration file is stored at:\n%s\n' "{d}/casa/rclone.conf" ;;
esac
"#,
            d = pasta.display()
        );
        std::fs::write(&script, corpo).unwrap();
        script
    }

    #[test]
    fn conectar_nunca_poe_segredo_no_argv_e_grava_o_conf_0600() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        let script = rclone_de_mentira(d.path());
        let rclone = ["sh", script.to_str().unwrap()];
        let g = cfg("id-de-teste", SEGREDO);
        conectar_com(&rclone, &g, &d.path().join("config")).unwrap();

        let argv = std::fs::read_to_string(d.path().join("argv.txt")).unwrap();
        assert_eq!(argv, "authorize drive\nconfig file\n");
        for segredo in [
            SEGREDO,
            "id-de-teste",
            "acesso-de-teste",
            "refresco-de-teste",
        ] {
            assert!(!argv.contains(segredo), "{segredo} apareceu no argv");
        }
        let ambiente = std::fs::read_to_string(d.path().join("ambiente.txt")).unwrap();
        assert!(ambiente.contains("RCLONE_DRIVE_CLIENT_ID=id-de-teste"));
        assert!(ambiente.contains(&format!("RCLONE_DRIVE_CLIENT_SECRET={SEGREDO}")));

        let conf = d.path().join("casa/rclone.conf");
        let t = std::fs::read_to_string(&conf).unwrap();
        assert!(t.contains("[gdrive]") && t.contains(&format!("client_secret = {SEGREDO}")));
        assert!(t.contains("refresco-de-teste"));
        assert_eq!(
            std::fs::metadata(&conf).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[test]
    fn sem_o_caminho_do_rclone_usa_a_pasta_de_config_da_pessoa() {
        let d = tempfile::tempdir().unwrap();
        let sem_resposta = d.path().join("muda.sh");
        std::fs::write(&sem_resposta, "exit 1\n").unwrap();
        let rclone = ["sh", sem_resposta.to_str().unwrap()];
        assert_eq!(
            caminho_conf(&rclone, Path::new("/home/luan/.config")),
            PathBuf::from("/home/luan/.config/rclone/rclone.conf")
        );
    }

    #[test]
    fn os_argv_do_google_nao_aceitam_segredo() {
        assert_eq!(args_autorizar(), vec!["rclone", "authorize", "drive"]);
        assert_eq!(args_onde_fica_o_conf(), vec!["rclone", "config", "file"]);
    }

    #[test]
    fn remoto_com_e_sem_cliente_proprio() {
        let com = secao_remoto(&cfg("id-de-teste", SEGREDO), TOKEN).unwrap();
        assert_eq!(
            com,
            format!(
                "[gdrive]\ntype = drive\nscope = drive\ntoken = {TOKEN}\nclient_id = id-de-teste\nclient_secret = {SEGREDO}\n"
            )
        );
        let sem = secao_remoto(&cfg("", ""), TOKEN).unwrap();
        assert!(!sem.contains("client_"));
        let pretty = secao_remoto(&cfg("", ""), "{\n  \"a\": 1\n}").unwrap();
        assert!(pretty.ends_with("token = {\"a\":1}\n"));
        assert!(secao_remoto(&cfg("id\nx", SEGREDO), TOKEN).is_err());
        assert!(secao_remoto(&cfg("", ""), "solto\ncom quebra").is_err());
    }

    #[test]
    fn remoto_novo_e_escrito_0600_e_o_resto_do_conf_fica() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        let conf = d.path().join("rclone/rclone.conf");
        std::fs::create_dir_all(conf.parent().unwrap()).unwrap();
        std::fs::write(
            &conf,
            "# meu rclone\n[local]\ntype = local\n\n[gdrive]\ntype = drive\ntoken = {\"velho\":1}\n\n[outro]\ntype = s3\n",
        )
        .unwrap();
        std::fs::set_permissions(&conf, std::fs::Permissions::from_mode(0o644)).unwrap();
        gravar_remoto(&conf, &cfg("id-de-teste", SEGREDO), TOKEN).unwrap();
        let t = std::fs::read_to_string(&conf).unwrap();
        assert!(t.starts_with(
            "# meu rclone\n[local]\ntype = local\n\n[outro]\ntype = s3\n\n[gdrive]\n"
        ));
        assert!(t.contains(&format!("token = {TOKEN}\n")) && !t.contains("velho"));
        assert_eq!(t.matches("[gdrive]").count(), 1);
        assert_eq!(
            std::fs::metadata(&conf).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert!(remoto_configurado("local:\ngdrive:\n"));
        assert!(!remoto_configurado("local:\n"));
    }

    #[test]
    fn conf_que_nao_existe_nasce_0600_e_conf_cifrado_nao_e_tocado() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        let conf = d.path().join("novo/rclone.conf");
        gravar_remoto(&conf, &cfg("", ""), TOKEN).unwrap();
        assert_eq!(
            std::fs::read_to_string(&conf).unwrap(),
            format!("[gdrive]\ntype = drive\nscope = drive\ntoken = {TOKEN}\n")
        );
        assert_eq!(
            std::fs::metadata(&conf).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let cifrado = d.path().join("cifrado.conf");
        let texto = "# Encrypted rclone configuration File\n\nRCLONE_ENCRYPT_V0:\nabc\n";
        std::fs::write(&cifrado, texto).unwrap();
        assert!(gravar_remoto(&cifrado, &cfg("", ""), TOKEN).is_err());
        assert_eq!(std::fs::read_to_string(&cifrado).unwrap(), texto);
    }

    #[test]
    fn caminho_do_conf_vem_da_saida_do_rclone() {
        assert_eq!(
            extrair_caminho_conf(
                "Configuration file is stored at:\n/home/luan/.config/rclone/rclone.conf\n"
            ),
            Some(PathBuf::from("/home/luan/.config/rclone/rclone.conf"))
        );
        assert_eq!(
            extrair_caminho_conf("Configuration file doesn't exist, but rclone will use this path:\n/home/luan/.config/rclone/rclone.conf\n"),
            Some(PathBuf::from("/home/luan/.config/rclone/rclone.conf"))
        );
        assert_eq!(extrair_caminho_conf("nada\n"), None);
    }

    #[test]
    fn script_do_dock_tira_so_o_comecar() {
        let a = args_tirar_do_dock();
        assert_eq!(
            &a[..7],
            &[
                "busctl",
                "--user",
                "call",
                "org.kde.plasmashell",
                "/PlasmaShell",
                "org.kde.PlasmaShell",
                "evaluateScript"
            ]
        );
        assert!(
            a[8].contains("novasce-comecar.desktop") && a[8].contains("org.kde.plasma.icontasks")
        );
    }

    #[test]
    fn assinatura_vai_para_documentos() {
        let d = tempfile::tempdir().unwrap();
        let p = salvar_assinatura(d.path(), "<p>Luan</p>").unwrap();
        assert_eq!(p, d.path().join("Novasce/Assinatura de e-mail.html"));
        assert_eq!(std::fs::read_to_string(p).unwrap(), "<p>Luan</p>");
    }
}
