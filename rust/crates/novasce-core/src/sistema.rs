//! Leituras do sistema para o sinal e para o Preparar. Cada função recebe o texto
//! que a ferramenta imprime; quem chama a ferramenta é o binário.
use crate::contrato::{Papel, Variante};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub const CAMINHO_PAPEL: &str = "/etc/novasce/papel";
pub const CAMINHO_VARIANTE: &str = "/usr/share/novasce/variante";
pub const CAMINHO_OS_RELEASE: &str = "/usr/lib/os-release";
pub const TRAVA_DOS_AGENTES: &str = "/run/novasce-agentes/rodada.lock";

/// `IMAGE_VERSION` do os-release (AAAA.MM.DD.N).
pub fn versao_do_os_release(conteudo: &str) -> Option<String> {
    conteudo.lines().find_map(|linha| {
        linha
            .strip_prefix("IMAGE_VERSION=")
            .map(|v| v.trim().trim_matches('"').to_string())
            .filter(|v| !v.is_empty())
    })
}

pub fn versao_do_sistema() -> String {
    fs::read_to_string(CAMINHO_OS_RELEASE)
        .ok()
        .and_then(|texto| versao_do_os_release(&texto))
        .unwrap_or_else(|| "desconhecida".to_string())
}

#[derive(Deserialize)]
struct StatusDoRpmOstree {
    deployments: Vec<Implantacao>,
}

#[derive(Deserialize)]
struct Implantacao {
    #[serde(default)]
    staged: bool,
    version: Option<String>,
    checksum: Option<String>,
}

/// Versão já baixada esperando o próximo reinício (`rpm-ostree status --json`).
pub fn versao_preparada(json: &str) -> Result<Option<String>, serde_json::Error> {
    let status: StatusDoRpmOstree = serde_json::from_str(json)?;
    Ok(status.deployments.into_iter().find(|d| d.staged).map(|d| {
        d.version
            .or_else(|| d.checksum.map(|c| c.chars().take(12).collect()))
            .unwrap_or_else(|| "desconhecida".to_string())
    }))
}

#[derive(Deserialize)]
struct Lsblk {
    blockdevices: Vec<Dispositivo>,
}

#[derive(Deserialize)]
struct Dispositivo {
    #[serde(rename = "type")]
    tipo: String,
    #[serde(default)]
    mountpoints: Vec<Option<String>>,
    #[serde(default)]
    children: Vec<Dispositivo>,
}

/// A raiz (`/sysroot` ou `/`) está dentro de um volume LUKS?
/// Entrada: `lsblk --json --tree -o NAME,TYPE,MOUNTPOINTS`.
pub fn disco_cifrado(json: &str) -> Result<bool, serde_json::Error> {
    let lsblk: Lsblk = serde_json::from_str(json)?;
    Ok(lsblk.blockdevices.iter().any(|d| raiz_cifrada(d, false)))
}

fn raiz_cifrada(dispositivo: &Dispositivo, acima_cifrado: bool) -> bool {
    let cifrado = acima_cifrado || dispositivo.tipo == "crypt";
    let e_raiz = dispositivo.mountpoints.iter().flatten().any(|m| m == "/" || m == "/sysroot");
    (e_raiz && cifrado) || dispositivo.children.iter().any(|filho| raiz_cifrada(filho, cifrado))
}

/// Entrada: `mokutil --sb-state`.
pub fn secure_boot_ligado(saida: &str) -> bool {
    saida.lines().any(|linha| linha.trim() == "SecureBoot enabled")
}

/// Nome da placa NVIDIA de vídeo, se houver. Entrada: `lspci -nn`.
pub fn placa_nvidia(lspci: &str) -> Option<String> {
    lspci.lines().find_map(|linha| {
        let de_video = linha.contains("[0300]") || linha.contains("[0302]");
        if !de_video || !linha.contains("[10de:") {
            return None;
        }
        let depois = linha.split_once("]: ")?.1;
        let descricao = depois.split(" [10de:").next()?.trim();
        if let (Some(inicio), true) = (descricao.rfind('['), descricao.ends_with(']')) {
            return Some(format!("NVIDIA {}", &descricao[inicio + 1..descricao.len() - 1]));
        }
        Some(descricao.replace("NVIDIA Corporation", "NVIDIA"))
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedeWifi {
    pub ssid: String,
    pub sinal: u8,
    pub protegida: bool,
    pub em_uso: bool,
}

/// Entrada: `nmcli -t -f IN-USE,SSID,SIGNAL,SECURITY device wifi list`.
pub fn redes_do_nmcli(saida: &str) -> Vec<RedeWifi> {
    let mut redes: Vec<RedeWifi> = Vec::new();
    for linha in saida.lines() {
        let campos = campos_do_nmcli(linha);
        if campos.len() < 4 || campos[1].is_empty() {
            continue;
        }
        let seguranca = campos[3].trim();
        let nova = RedeWifi {
            ssid: campos[1].clone(),
            sinal: campos[2].trim().parse().unwrap_or(0),
            protegida: !seguranca.is_empty() && seguranca != "--",
            em_uso: campos[0].trim() == "*",
        };
        match redes.iter_mut().find(|r| r.ssid == nova.ssid) {
            Some(rede) => {
                rede.sinal = rede.sinal.max(nova.sinal);
                rede.em_uso |= nova.em_uso;
                rede.protegida |= nova.protegida;
            }
            None => redes.push(nova),
        }
    }
    redes.sort_by(|a, b| b.em_uso.cmp(&a.em_uso).then(b.sinal.cmp(&a.sinal)));
    redes
}

/// O modo `-t` do nmcli separa por `:` e escapa `:` e `\` com `\`.
fn campos_do_nmcli(linha: &str) -> Vec<String> {
    let mut campos = vec![String::new()];
    let mut letras = linha.chars();
    while let Some(letra) = letras.next() {
        match letra {
            '\\' => {
                if let Some(proxima) = letras.next() {
                    campos.last_mut().expect("sempre há um campo").push(proxima);
                }
            }
            ':' => campos.push(String::new()),
            outra => campos.last_mut().expect("sempre há um campo").push(outra),
        }
    }
    campos
}

/// Entrada: `nmcli -t -f CONNECTIVITY general`.
pub fn conectividade_plena(saida: &str) -> bool {
    saida.trim() == "full"
}

/// Entrada: `nmcli -t -f TYPE,STATE device`.
pub fn cabo_conectado(saida: &str) -> bool {
    saida.lines().any(|linha| linha.trim() == "ethernet:connected")
}

pub fn ler_papel(caminho: &Path) -> Option<Papel> {
    fs::read_to_string(caminho).ok()?.parse().ok()
}

pub fn ler_variante(caminho: &Path) -> Variante {
    match fs::read_to_string(caminho).map(|t| t.trim().to_string()).as_deref() {
        Ok("nvidia") => Variante::Nvidia,
        _ => Variante::Padrao,
    }
}

/// Os agentes estão fora de uma rodada? O executor segura a trava com `flock`
/// enquanto roda (Tarefa 14). Sem arquivo de trava, não há rodada.
pub fn agentes_ociosos(trava: &Path) -> bool {
    let Ok(arquivo) = File::open(trava) else {
        return true;
    };
    let fd = arquivo.as_raw_fd();
    // SAFETY: fd vem de um File aberto que vive até o fim da função.
    let pegou = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) } == 0;
    if pegou {
        // SAFETY: mesmo fd, ainda aberto.
        unsafe { libc::flock(fd, libc::LOCK_UN) };
    }
    pegou
}

pub fn nome_da_maquina() -> String {
    ["/etc/hostname", "/proc/sys/kernel/hostname"]
        .iter()
        .find_map(|caminho| {
            fs::read_to_string(caminho).ok().map(|t| t.trim().to_string()).filter(|t| !t.is_empty())
        })
        .unwrap_or_else(|| "novasce".to_string())
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::os::unix::io::AsRawFd;

    const OS_RELEASE: &str = "NAME=\"Novasce OS\"\nID=aurora\nIMAGE_VERSION=\"2026.10.24.1\"\n";

    const RPM_OSTREE: &str = r#"{"deployments":[
      {"id":"a","booted":true,"staged":false,"version":"2026.10.24.1","checksum":"aaaa1111bbbb2222"},
      {"id":"b","booted":false,"staged":true,"version":"2026.10.31.1","checksum":"cccc3333dddd4444"}
    ],"transaction":null}"#;

    const LSBLK_CIFRADO: &str = r#"{"blockdevices":[{"name":"nvme0n1","type":"disk","mountpoints":[null],"children":[
      {"name":"nvme0n1p1","type":"part","mountpoints":["/boot/efi"]},
      {"name":"nvme0n1p2","type":"part","mountpoints":["/boot"]},
      {"name":"nvme0n1p3","type":"part","mountpoints":[null],"children":[
        {"name":"luks-1b2c","type":"crypt","mountpoints":["/var/home","/var","/sysroot","/etc"]}]}]}]}"#;

    const LSBLK_ABERTO: &str = r#"{"blockdevices":[{"name":"sda","type":"disk","mountpoints":[null],"children":[
      {"name":"sda1","type":"part","mountpoints":["/boot/efi"]},
      {"name":"sda3","type":"part","mountpoints":["/var/home","/var","/sysroot","/etc"]}]}]}"#;

    const LSPCI: &str = "\
00:02.0 VGA compatible controller [0300]: Intel Corporation CometLake-S GT2 [UHD Graphics 630] [8086:3e92] (rev 05)
01:00.0 VGA compatible controller [0300]: NVIDIA Corporation TU106 [GeForce RTX 2060] [10de:1f08] (rev a1)
01:00.1 Audio device [0403]: NVIDIA Corporation TU106 High Definition Audio Controller [10de:10f9] (rev a1)
";

    const NMCLI_WIFI: &str = "\
*:Novasce Sede:82:WPA2
 :Vizinho\\:5G:64:WPA1 WPA2
 :Café Aberto:40:
 ::30:WPA2
 :Novasce Sede:50:WPA2
";

    #[test]
    fn le_a_versao_do_os_release() {
        assert_eq!(versao_do_os_release(OS_RELEASE).as_deref(), Some("2026.10.24.1"));
        assert_eq!(versao_do_os_release("NAME=x\n"), None);
    }

    #[test]
    fn versao_preparada_e_a_do_deployment_staged() {
        assert_eq!(versao_preparada(RPM_OSTREE).unwrap().as_deref(), Some("2026.10.31.1"));
    }

    #[test]
    fn sem_staged_nao_ha_versao_preparada() {
        let so_booted = r#"{"deployments":[{"booted":true,"staged":false,"version":"2026.10.24.1"}]}"#;
        assert_eq!(versao_preparada(so_booted).unwrap(), None);
    }

    #[test]
    fn staged_sem_versao_usa_o_checksum() {
        let sem_versao = r#"{"deployments":[{"staged":true,"checksum":"cccc3333dddd4444eeee"}]}"#;
        assert_eq!(versao_preparada(sem_versao).unwrap().as_deref(), Some("cccc3333dddd"));
    }

    #[test]
    fn disco_cifrado_quando_a_raiz_esta_dentro_do_luks() {
        assert!(disco_cifrado(LSBLK_CIFRADO).unwrap());
        assert!(!disco_cifrado(LSBLK_ABERTO).unwrap());
    }

    #[test]
    fn secure_boot_pelo_mokutil() {
        assert!(secure_boot_ligado("SecureBoot enabled\n"));
        assert!(!secure_boot_ligado("SecureBoot disabled\n"));
        assert!(!secure_boot_ligado("EFI variables are not supported on this system\n"));
    }

    #[test]
    fn placa_nvidia_com_o_nome_de_mercado() {
        assert_eq!(placa_nvidia(LSPCI).as_deref(), Some("NVIDIA GeForce RTX 2060"));
        assert_eq!(placa_nvidia(LSPCI.lines().next().unwrap()), None);
    }

    #[test]
    fn redes_wifi_sem_repetir_e_por_sinal() {
        let redes = redes_do_nmcli(NMCLI_WIFI);
        assert_eq!(
            redes,
            vec![
                RedeWifi { ssid: "Novasce Sede".into(), sinal: 82, protegida: true, em_uso: true },
                RedeWifi { ssid: "Vizinho:5G".into(), sinal: 64, protegida: true, em_uso: false },
                RedeWifi { ssid: "Café Aberto".into(), sinal: 40, protegida: false, em_uso: false },
            ]
        );
    }

    #[test]
    fn conectividade_e_cabo() {
        assert!(conectividade_plena("full\n"));
        assert!(!conectividade_plena("limited\n"));
        assert!(cabo_conectado("ethernet:connected\nwifi:disconnected\n"));
        assert!(!cabo_conectado("ethernet:unavailable\nwifi:connected\n"));
    }

    #[test]
    fn papel_e_variante_dos_arquivos() {
        let pasta = tempfile::tempdir().unwrap();
        let papel = pasta.path().join("papel");
        let variante = pasta.path().join("variante");
        assert_eq!(ler_papel(&papel), None);
        std::fs::write(&papel, "sede\n").unwrap();
        std::fs::write(&variante, "nvidia\n").unwrap();
        assert_eq!(ler_papel(&papel), Some(Papel::Sede));
        assert_eq!(ler_variante(&variante), Variante::Nvidia);
        assert_eq!(ler_variante(&pasta.path().join("nada")), Variante::Padrao);
    }

    #[test]
    fn trava_presa_quer_dizer_rodada_em_andamento() {
        let trava = tempfile::NamedTempFile::new().unwrap();
        assert!(agentes_ociosos(trava.path()));
        let segurando = std::fs::File::open(trava.path()).unwrap();
        assert_eq!(unsafe { libc::flock(segurando.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) }, 0);
        assert!(!agentes_ociosos(trava.path()));
        drop(segurando);
        assert!(agentes_ociosos(trava.path()));
    }

    #[test]
    fn sem_trava_nao_ha_rodada() {
        assert!(agentes_ociosos(Path::new("/nao/existe/rodada.lock")));
    }
}
