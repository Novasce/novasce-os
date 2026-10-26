//! Conectar no Wi-Fi sem a senha passar pela linha de comando.
//!
//! No Linux qualquer conta lê `/proc/<pid>/cmdline`, então `nmcli ... password <senha>`
//! entrega a senha do Wi-Fi a todas as contas da máquina. Aqui o perfil nasce sem segredo
//! (`nmcli connection add`) e a senha entra por `nmcli connection up ... passwd-file
//! <arquivo>`, opção que o manual do nmcli documenta para entregar credenciais na
//! ativação: uma linha `setting.propriedade:senha` por segredo. O agente do nmcli responde
//! ao NetworkManager com esse valor, e o NetworkManager grava o segredo no perfil porque o
//! `psk-flags` padrão (0) é "segredo do sistema": em `get_secrets_done_cb`
//! (nm-settings-connection.c) ele só persiste quando o agente devolve segredos do sistema.
//! É isso que faz a máquina reconectar sozinha depois do boot.
//!
//! A outra saída seria `nmcli --ask` com a senha pelo stdin, mas o manual diz para não usar
//! essa opção em script. O arquivo fica numa pasta 0700, com modo 0600, e some quando o
//! `ArquivoDeSenha` sai de escopo.
use crate::sistema::campos_do_nmcli;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Segundos que o nmcli espera a ativação.
const ESPERA: &str = "30";
/// Propriedade que o NetworkManager pede ao agente do nmcli. Vale para WPA2 e WPA3 (SAE).
const PROPRIEDADE_DA_SENHA: &str = "802-11-wireless-security.psk";
/// Quantos perfis com o mesmo nome apagar antes de criar o novo (o NetworkManager aceita nomes repetidos).
const PERFIS_ANTIGOS_MAX: usize = 5;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ErroWifi {
    #[error("Não conectou na rede {0}. Confira a senha e tente de novo.")]
    NaoConectou(String),
    #[error("A rede {0} não apareceu na busca. Chegue mais perto do roteador e tente de novo.")]
    NaoApareceu(String),
    #[error("A rede {0} usa um tipo de segurança que o Preparar não configura. Use o cabo ou outra rede.")]
    SegurancaNaoSuportada(String),
    #[error("A senha tem um caractere que o Wi-Fi não aceita. Digite de novo.")]
    SenhaComCaractereInvalido,
    #[error("Não deu para guardar a senha com segurança nesta máquina: {0}")]
    ArquivoDaSenha(String),
}

/// Onde criar a pasta privada da senha: o `XDG_RUNTIME_DIR` da conta (tmpfs 0700) ou o temporário.
pub fn pasta_para_senha() -> PathBuf {
    match std::env::var_os("XDG_RUNTIME_DIR") {
        Some(pasta) if Path::new(&pasta).is_dir() => PathBuf::from(pasta),
        _ => std::env::temp_dir(),
    }
}

/// A linha do `passwd-file`. O valor passa por escape de barra invertida (o NetworkManager
/// desfaz `\\` e `\NNN` e corta espaços nas pontas), então barra e espaço saem escapados.
/// Caractere de controle não existe em senha de Wi-Fi e quebraria a linha: recusa.
pub fn linha_da_senha(senha: &str) -> Result<String, ErroWifi> {
    if senha.chars().any(|c| c.is_ascii_control()) {
        return Err(ErroWifi::SenhaComCaractereInvalido);
    }
    let mut linha = format!("{PROPRIEDADE_DA_SENHA}:");
    for letra in senha.chars() {
        match letra {
            '\\' => linha.push_str("\\\\"),
            ' ' => linha.push_str("\\040"),
            outra => linha.push(outra),
        }
    }
    linha.push('\n');
    Ok(linha)
}

/// Escolhe o `wifi-sec.key-mgmt` do perfil pela coluna SECURITY de
/// `nmcli -t -f SSID,SECURITY device wifi list`. Roteador com WPA2 (ou misto WPA2/WPA3) usa
/// `wpa-psk`; só WPA3 usa `sae`. WEP, rede corporativa (802.1X) e aberta ficam de fora.
pub fn gestao_de_chaves(lista: &str, ssid: &str) -> Result<&'static str, ErroWifi> {
    let (mut achou, mut psk, mut sae) = (false, false, false);
    for linha in lista.lines() {
        let campos = campos_do_nmcli(linha);
        if campos.len() < 2 || campos[0] != ssid {
            continue;
        }
        achou = true;
        let tem = |palavra: &str| campos[1].split_whitespace().any(|p| p == palavra);
        if tem("802.1X") || tem("WPA-EAP-SUITE-B-192") || tem("WEP") {
            continue;
        }
        if tem("WPA1") || tem("WPA2") {
            psk = true;
        } else if tem("WPA3") {
            sae = true;
        }
    }
    match (achou, psk, sae) {
        (false, _, _) => Err(ErroWifi::NaoApareceu(ssid.to_string())),
        (true, true, _) => Ok("wpa-psk"),
        (true, false, true) => Ok("sae"),
        (true, false, false) => Err(ErroWifi::SegurancaNaoSuportada(ssid.to_string())),
    }
}

/// O arquivo temporário com a senha: pasta 0700, arquivo 0600, apagado no `Drop`.
/// A struct guarda só os caminhos, nunca a senha.
#[derive(Debug)]
pub struct ArquivoDeSenha {
    pasta: PathBuf,
    caminho: PathBuf,
}

impl ArquivoDeSenha {
    pub fn criar(base: &Path, conteudo: &str) -> Result<Self, ErroWifi> {
        static SEQUENCIA: AtomicU32 = AtomicU32::new(0);
        let erro = |fonte: std::io::Error| ErroWifi::ArquivoDaSenha(fonte.to_string());
        let instante = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let pasta = base.join(format!(
            "novasce-wifi-{}-{}-{}",
            std::process::id(),
            instante,
            SEQUENCIA.fetch_add(1, Ordering::Relaxed)
        ));
        // Não recursivo: falha se o nome já existir, em vez de seguir um link posto de antemão.
        DirBuilder::new().mode(0o700).create(&pasta).map_err(erro)?;
        let arquivo = Self {
            caminho: pasta.join("senha"),
            pasta,
        };
        // Daqui para baixo, qualquer falha apaga a pasta pelo Drop de `arquivo`.
        let mut aberto = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&arquivo.caminho)
            .map_err(erro)?;
        aberto.write_all(conteudo.as_bytes()).map_err(erro)?;
        Ok(arquivo)
    }

    pub fn caminho(&self) -> &Path {
        &self.caminho
    }
}

impl Drop for ArquivoDeSenha {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.caminho);
        let _ = fs::remove_dir(&self.pasta);
    }
}

/// Conecta na rede. `nmcli` roda um comando do nmcli com estes argumentos e devolve o stdout.
/// Rede aberta (senha vazia) usa `device wifi connect`, que não leva segredo. Rede com senha
/// ganha um perfil sem segredo e a senha entra pelo `passwd-file`. Se a ativação falhar, o
/// perfil é apagado, para a máquina não ficar tentando uma senha errada.
pub fn conectar(
    mut nmcli: impl FnMut(&[&str]) -> Result<String, String>,
    pasta: &Path,
    ssid: &str,
    senha: &str,
) -> Result<(), ErroWifi> {
    let falhou = || ErroWifi::NaoConectou(ssid.to_string());
    if senha.is_empty() {
        return nmcli(&["--wait", ESPERA, "device", "wifi", "connect", ssid])
            .map(|_| ())
            .map_err(|_| falhou());
    }
    let linha = linha_da_senha(senha)?;
    let lista = nmcli(&[
        "-t",
        "-f",
        "SSID,SECURITY",
        "device",
        "wifi",
        "list",
        "--rescan",
        "auto",
    ])
    .map_err(|_| falhou())?;
    let gestao = gestao_de_chaves(&lista, ssid)?;
    let arquivo = ArquivoDeSenha::criar(pasta, &linha)?;
    drop(linha);

    // Quem roda o Preparar de novo já tem um perfil com este nome.
    for _ in 0..PERFIS_ANTIGOS_MAX {
        if nmcli(&["connection", "delete", "id", ssid]).is_err() {
            break;
        }
    }
    nmcli(&[
        "connection",
        "add",
        "type",
        "wifi",
        "ifname",
        "*",
        "con-name",
        ssid,
        "ssid",
        ssid,
        "wifi-sec.key-mgmt",
        gestao,
    ])
    .map_err(|_| falhou())?;
    let caminho = arquivo.caminho().to_string_lossy().into_owned();
    let ativou = nmcli(&[
        "--wait",
        ESPERA,
        "connection",
        "up",
        "id",
        ssid,
        "passwd-file",
        caminho.as_str(),
    ]);
    if ativou.is_err() {
        let _ = nmcli(&["connection", "delete", "id", ssid]);
        return Err(falhou());
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    /// Senha inventada para os testes, com espaço e barra para exercitar o escape.
    const SENHA: &str = "segredo de teste\\9";

    /// O que o nmcli falso enxerga do arquivo no momento do `connection up`.
    #[derive(Default)]
    struct Visto {
        conteudo: String,
        modo_do_arquivo: u32,
        modo_da_pasta: u32,
    }

    /// Faz o papel do nmcli: guarda os argumentos de cada chamada e, no `connection up`,
    /// lê o `passwd-file` como o nmcli leria.
    fn nmcli_falso<'a>(
        chamadas: &'a mut Vec<Vec<String>>,
        visto: &'a mut Visto,
        lista: &'a str,
        ativa: bool,
        perfis_antigos: usize,
    ) -> impl FnMut(&[&str]) -> Result<String, String> + 'a {
        let mut antigos = perfis_antigos;
        move |args: &[&str]| {
            chamadas.push(args.iter().map(|a| a.to_string()).collect());
            if args.contains(&"list") {
                return Ok(lista.to_string());
            }
            if args.contains(&"delete") {
                return if antigos > 0 {
                    antigos -= 1;
                    Ok(String::new())
                } else {
                    Err("Error: unknown connection".to_string())
                };
            }
            if args.contains(&"up") {
                let posicao = args.iter().position(|a| *a == "passwd-file").unwrap();
                let arquivo = Path::new(args[posicao + 1]);
                visto.conteudo = fs::read_to_string(arquivo).unwrap();
                visto.modo_do_arquivo = fs::metadata(arquivo).unwrap().permissions().mode() & 0o777;
                visto.modo_da_pasta = fs::metadata(arquivo.parent().unwrap())
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777;
                return if ativa {
                    Ok(String::new())
                } else {
                    Err("Error: Connection activation failed".to_string())
                };
            }
            Ok(String::new())
        }
    }

    fn subcomandos(chamadas: &[Vec<String>]) -> Vec<&str> {
        chamadas
            .iter()
            .map(|c| {
                if c.iter().any(|a| a == "list") {
                    "list"
                } else {
                    ["delete", "add", "up"]
                        .into_iter()
                        .find(|s| c.iter().any(|a| a == s))
                        .unwrap_or("outro")
                }
            })
            .collect()
    }

    /// Desfaz o escape como o NetworkManager desfaz (nm_utils_buf_utf8safe_unescape,
    /// com corte de espaços nas pontas), para provar que o valor volta igual.
    fn como_o_networkmanager_le(linha: &str) -> String {
        let valor = linha
            .trim_end_matches('\n')
            .split_once(':')
            .expect("a linha tem dois pontos")
            .1;
        let valor = valor.trim_start_matches(|c: char| c.is_ascii_whitespace());
        let bytes = valor.as_bytes();
        let mut saida: Vec<u8> = Vec::new();
        let mut cauda = 0;
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] != b'\\' {
                saida.push(bytes[i]);
                i += 1;
                continue;
            }
            i += 1;
            if bytes[i].is_ascii_digit() {
                let mut v = u32::from(bytes[i] - b'0');
                i += 1;
                for _ in 0..2 {
                    if i < bytes.len() && (b'0'..=b'7').contains(&bytes[i]) {
                        v = v * 8 + u32::from(bytes[i] - b'0');
                        i += 1;
                    } else {
                        break;
                    }
                }
                saida.push(v as u8);
            } else {
                saida.push(bytes[i]);
                i += 1;
            }
            cauda = saida.len();
        }
        // O NetworkManager só corta espaço do trecho literal depois do último escape.
        while saida.len() > cauda && saida.last().is_some_and(|b| b.is_ascii_whitespace()) {
            saida.pop();
        }
        String::from_utf8(saida).unwrap()
    }

    #[test]
    fn senha_fica_fora_do_argv_e_o_arquivo_e_privado() {
        let base = tempfile::tempdir().unwrap();
        let (mut chamadas, mut visto) = (Vec::new(), Visto::default());
        let lista = "Outra Rede:WPA2\nNovasce Sede:WPA2 WPA3\n";
        let nmcli = nmcli_falso(&mut chamadas, &mut visto, lista, true, 0);

        conectar(nmcli, base.path(), "Novasce Sede", SENHA).unwrap();

        assert_eq!(subcomandos(&chamadas), ["list", "delete", "add", "up"]);
        for chamada in &chamadas {
            for arg in chamada {
                assert!(!arg.contains(SENHA), "senha no argv: {chamada:?}");
                assert!(
                    !arg.contains("segredo"),
                    "pedaço da senha no argv: {chamada:?}"
                );
            }
            assert!(!chamada
                .iter()
                .any(|a| a == "password" || a == "wifi-sec.psk"));
        }
        let criar = &chamadas[2];
        assert!(criar
            .windows(2)
            .any(|p| p == ["wifi-sec.key-mgmt", "wpa-psk"]));
        assert_eq!(
            visto.conteudo,
            "802-11-wireless-security.psk:segredo\\040de\\040teste\\\\9\n"
        );
        assert_eq!(visto.modo_do_arquivo, 0o600);
        assert_eq!(visto.modo_da_pasta, 0o700);
        assert_eq!(
            fs::read_dir(base.path()).unwrap().count(),
            0,
            "o arquivo sobrou"
        );
    }

    #[test]
    fn ativacao_que_falha_apaga_o_perfil_e_o_arquivo() {
        let base = tempfile::tempdir().unwrap();
        let (mut chamadas, mut visto) = (Vec::new(), Visto::default());
        let nmcli = nmcli_falso(&mut chamadas, &mut visto, "Casa:WPA2\n", false, 0);

        let erro = conectar(nmcli, base.path(), "Casa", SENHA).unwrap_err();

        assert_eq!(erro, ErroWifi::NaoConectou("Casa".to_string()));
        assert_eq!(
            subcomandos(&chamadas),
            ["list", "delete", "add", "up", "delete"]
        );
        assert_eq!(
            fs::read_dir(base.path()).unwrap().count(),
            0,
            "o arquivo sobrou"
        );
    }

    #[test]
    fn perfis_antigos_com_o_mesmo_nome_saem_antes() {
        let base = tempfile::tempdir().unwrap();
        let (mut chamadas, mut visto) = (Vec::new(), Visto::default());
        let nmcli = nmcli_falso(&mut chamadas, &mut visto, "Casa:WPA2\n", true, 2);

        conectar(nmcli, base.path(), "Casa", SENHA).unwrap();

        assert_eq!(
            subcomandos(&chamadas),
            ["list", "delete", "delete", "delete", "add", "up"]
        );
    }

    #[test]
    fn rede_aberta_nao_leva_segredo_nem_arquivo() {
        let base = tempfile::tempdir().unwrap();
        let (mut chamadas, mut visto) = (Vec::new(), Visto::default());
        let nmcli = nmcli_falso(&mut chamadas, &mut visto, "", true, 0);

        conectar(nmcli, base.path(), "Café Aberto", "").unwrap();

        assert_eq!(
            chamadas,
            [["--wait", "30", "device", "wifi", "connect", "Café Aberto"]]
        );
        assert_eq!(fs::read_dir(base.path()).unwrap().count(), 0);
    }

    #[test]
    fn rede_ausente_ou_sem_suporte_para_antes_de_criar_qualquer_coisa() {
        for (lista, esperado) in [
            ("Outra:WPA2\n", ErroWifi::NaoApareceu("Casa".into())),
            ("Casa:WEP\n", ErroWifi::SegurancaNaoSuportada("Casa".into())),
            (
                "Casa:WPA2 802.1X\n",
                ErroWifi::SegurancaNaoSuportada("Casa".into()),
            ),
            ("Casa:--\n", ErroWifi::SegurancaNaoSuportada("Casa".into())),
        ] {
            let base = tempfile::tempdir().unwrap();
            let (mut chamadas, mut visto) = (Vec::new(), Visto::default());
            let nmcli = nmcli_falso(&mut chamadas, &mut visto, lista, true, 0);

            assert_eq!(
                conectar(nmcli, base.path(), "Casa", SENHA).unwrap_err(),
                esperado
            );

            assert_eq!(subcomandos(&chamadas), ["list"], "{lista}");
            assert_eq!(fs::read_dir(base.path()).unwrap().count(), 0);
        }
    }

    #[test]
    fn senha_com_caractere_de_controle_nem_chega_ao_nmcli() {
        let base = tempfile::tempdir().unwrap();
        let (mut chamadas, mut visto) = (Vec::new(), Visto::default());
        let nmcli = nmcli_falso(&mut chamadas, &mut visto, "Casa:WPA2\n", true, 0);

        let erro = conectar(nmcli, base.path(), "Casa", "abc\ndef").unwrap_err();

        assert_eq!(erro, ErroWifi::SenhaComCaractereInvalido);
        assert!(chamadas.is_empty());
    }

    #[test]
    fn gestao_de_chaves_pela_coluna_security() {
        let lista = "Misto:WPA2 WPA3\nSo3:WPA3\nVelha:WPA1 WPA2\nWep:WEP\nCorp:WPA2 802.1X\nAberta:--\nVizinho\\:5G:WPA2\n\
                     Dupla:WPA2 802.1X\nDupla:WPA3\nDupla2:WPA2 802.1X\nDupla2:WPA2\n";
        assert_eq!(gestao_de_chaves(lista, "Misto"), Ok("wpa-psk"));
        assert_eq!(gestao_de_chaves(lista, "So3"), Ok("sae"));
        assert_eq!(gestao_de_chaves(lista, "Velha"), Ok("wpa-psk"));
        assert_eq!(gestao_de_chaves(lista, "Vizinho:5G"), Ok("wpa-psk"));
        assert_eq!(gestao_de_chaves(lista, "Dupla"), Ok("sae"));
        assert_eq!(gestao_de_chaves(lista, "Dupla2"), Ok("wpa-psk"));
        assert!(matches!(
            gestao_de_chaves(lista, "Wep"),
            Err(ErroWifi::SegurancaNaoSuportada(_))
        ));
        assert!(matches!(
            gestao_de_chaves(lista, "Corp"),
            Err(ErroWifi::SegurancaNaoSuportada(_))
        ));
        assert!(matches!(
            gestao_de_chaves(lista, "Aberta"),
            Err(ErroWifi::SegurancaNaoSuportada(_))
        ));
        assert!(matches!(
            gestao_de_chaves(lista, "Nenhuma"),
            Err(ErroWifi::NaoApareceu(_))
        ));
    }

    #[test]
    fn escape_da_senha_volta_igual_no_networkmanager() {
        for senha in [
            "simples12345",
            " espaço no começo e no fim ",
            "com:dois=pontos",
            "barra\\no meio\\",
            "literal\\040de\\012escape",
            "acentuação ç ã 日本",
            "  ",
        ] {
            let linha = linha_da_senha(senha).unwrap();
            assert!(linha.starts_with("802-11-wireless-security.psk:"));
            assert_eq!(linha.matches('\n').count(), 1, "uma linha só: {senha:?}");
            assert_eq!(como_o_networkmanager_le(&linha), senha, "{senha:?}");
        }
    }

    #[test]
    fn arquivo_da_senha_e_privado_e_some_ao_sair_de_escopo() {
        let base = tempfile::tempdir().unwrap();
        let arquivo = ArquivoDeSenha::criar(base.path(), "x:y\n").unwrap();
        let caminho = arquivo.caminho().to_path_buf();
        let modo = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(modo(&caminho), 0o600);
        assert_eq!(modo(caminho.parent().unwrap()), 0o700);
        assert_eq!(fs::read_to_string(&caminho).unwrap(), "x:y\n");
        assert!(!format!("{arquivo:?}").contains("x:y"));
        drop(arquivo);
        assert!(!caminho.exists());
        assert!(!caminho.parent().unwrap().exists());
    }

    #[test]
    fn dois_arquivos_ao_mesmo_tempo_nao_se_pisam() {
        let base = tempfile::tempdir().unwrap();
        let a = ArquivoDeSenha::criar(base.path(), "a\n").unwrap();
        let b = ArquivoDeSenha::criar(base.path(), "b\n").unwrap();
        assert_ne!(a.caminho(), b.caminho());
        assert_eq!(fs::read_to_string(a.caminho()).unwrap(), "a\n");
        assert_eq!(fs::read_to_string(b.caminho()).unwrap(), "b\n");
    }
}
