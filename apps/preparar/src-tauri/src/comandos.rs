//! Os comandos que a tela do Preparar chama. O que pede administrador vai pelo
//! pkexec para o novasce-helper; o resto lê o sistema direto.
use novasce_core::sistema::{
    cabo_conectado, conectividade_plena, placa_nvidia, redes_do_nmcli, RedeWifi, CAMINHO_VARIANTE,
};
use novasce_core::wifi;
use serde::Serialize;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter};

const PKEXEC: &str = "/usr/bin/pkexec";
const HELPER: &str = "/usr/libexec/novasce/novasce-helper";
const NMCLI: &str = "/usr/bin/nmcli";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacaDeVideo {
    nvidia: Option<String>,
    variante_da_imagem: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDaRede {
    conectada: bool,
    por_cabo: bool,
}

async fn bloqueante<T: Send + 'static>(
    tarefa: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(tarefa).await.map_err(|e| e.to_string())?
}

fn rodar(programa: &str, args: &[&str]) -> Result<String, String> {
    let saida = Command::new(programa)
        .args(args)
        .output()
        .map_err(|e| format!("{programa} não abriu: {e}"))?;
    if saida.status.success() {
        Ok(String::from_utf8_lossy(&saida.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&saida.stderr).trim().to_string())
    }
}

/// Uma linha JSON por evento; o resto (avisos do pkexec) é ignorado.
pub fn ler_linhas(saida: &str) -> Vec<Value> {
    saida.lines().filter_map(|linha| serde_json::from_str(linha).ok()).collect()
}

pub fn mensagem_de_erro(linhas: &[Value], codigo: Option<i32>) -> String {
    let do_ajudante = linhas
        .iter()
        .rev()
        .find(|v| v["evento"] == "erro")
        .and_then(|v| v["mensagem"].as_str());
    if let Some(mensagem) = do_ajudante {
        return mensagem.to_string();
    }
    match codigo {
        Some(126) | Some(127) => "A senha de administrador não foi aceita, ou o pedido foi cancelado.".to_string(),
        _ => "O ajudante da Novasce parou sem dizer o motivo.".to_string(),
    }
}

fn ajudante(args: &[&str]) -> Result<Vec<Value>, String> {
    let saida = Command::new(PKEXEC)
        .arg(HELPER)
        .args(args)
        .output()
        .map_err(|e| format!("O ajudante não abriu: {e}"))?;
    let linhas = ler_linhas(&String::from_utf8_lossy(&saida.stdout));
    if saida.status.success() {
        Ok(linhas)
    } else {
        Err(mensagem_de_erro(&linhas, saida.status.code()))
    }
}

fn evento<'a>(linhas: &'a [Value], nome: &str) -> Option<&'a Value> {
    linhas.iter().find(|v| v["evento"] == nome)
}

#[tauri::command]
pub async fn placa_de_video() -> Result<PlacaDeVideo, String> {
    bloqueante(|| {
        let lspci = rodar("/usr/bin/lspci", &["-nn"]).unwrap_or_default();
        let variante = std::fs::read_to_string(CAMINHO_VARIANTE)
            .map(|t| t.trim().to_string())
            .unwrap_or_else(|_| "padrao".to_string());
        Ok(PlacaDeVideo { nvidia: placa_nvidia(&lspci), variante_da_imagem: variante })
    })
    .await
}

#[tauri::command]
pub async fn estado_da_rede() -> Result<EstadoDaRede, String> {
    bloqueante(|| {
        let geral = rodar(NMCLI, &["-t", "-f", "CONNECTIVITY", "general"]).unwrap_or_default();
        let aparelhos = rodar(NMCLI, &["-t", "-f", "TYPE,STATE", "device"]).unwrap_or_default();
        Ok(EstadoDaRede { conectada: conectividade_plena(&geral), por_cabo: cabo_conectado(&aparelhos) })
    })
    .await
}

#[tauri::command]
pub async fn redes_wifi() -> Result<Vec<RedeWifi>, String> {
    bloqueante(|| {
        let saida = rodar(
            NMCLI,
            &["-t", "-f", "IN-USE,SSID,SIGNAL,SECURITY", "device", "wifi", "list", "--rescan", "auto"],
        )
        .unwrap_or_default();
        Ok(redes_do_nmcli(&saida))
    })
    .await
}

#[tauri::command]
pub async fn conectar_wifi(ssid: String, senha: String) -> Result<(), String> {
    bloqueante(move || {
        // A senha nunca vai na linha de comando (qualquer conta lê /proc/<pid>/cmdline): entra
        // por um arquivo 0600 que o nmcli lê e que some logo depois. O NetworkManager guarda
        // a senha no perfil desta máquina e a reconexão depois do boot sai dele.
        wifi::conectar(|args| rodar(NMCLI, args), &wifi::pasta_para_senha(), &ssid, &senha)
            .map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
pub async fn definir_papel(papel: String) -> Result<(), String> {
    bloqueante(move || {
        if papel != "sede" && papel != "notebook" {
            return Err("Papel desconhecido.".to_string());
        }
        ajudante(&["papel", &papel]).map(|_| ())
    })
    .await
}

/// Abre o registro e devolve na hora. Cada linha do ajudante vira o evento "registro".
#[tauri::command]
pub fn registrar(app: AppHandle) -> Result<(), String> {
    let mut filho = Command::new(PKEXEC)
        .arg(HELPER)
        .arg("registrar")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("O ajudante não abriu: {e}"))?;
    let saida = filho.stdout.take().ok_or("O ajudante abriu sem saída.")?;
    std::thread::spawn(move || {
        let mut houve_erro = false;
        for linha in BufReader::new(saida).lines().map_while(Result::ok) {
            if let Ok(valor) = serde_json::from_str::<Value>(&linha) {
                houve_erro |= valor["evento"] == "erro";
                let _ = app.emit("registro", valor);
            }
        }
        let codigo = match filho.wait() {
            Ok(status) if status.success() => return,
            Ok(status) => status.code(),
            Err(_) => None,
        };
        if !houve_erro {
            let mensagem = mensagem_de_erro(&[], codigo);
            let _ = app.emit("registro", json!({"evento": "erro", "mensagem": mensagem}));
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn listar_time() -> Result<Value, String> {
    bloqueante(|| {
        let linhas = ajudante(&["time"])?;
        evento(&linhas, "time")
            .map(|v| v["pessoas"].clone())
            .ok_or_else(|| "O Tenancy não mandou a lista do time.".to_string())
    })
    .await
}

#[tauri::command]
pub async fn criar_conta(nome: String, email: String, dono: Option<String>) -> Result<String, String> {
    bloqueante(move || {
        let mut args = vec!["criar-conta", "--nome", nome.as_str(), "--email", email.as_str()];
        if let Some(id) = dono.as_deref() {
            args.extend(["--dono", id]);
        }
        let linhas = ajudante(&args)?;
        evento(&linhas, "conta")
            .and_then(|v| v["login"].as_str())
            .map(str::to_string)
            .ok_or_else(|| "A conta não voltou do ajudante.".to_string())
    })
    .await
}

#[tauri::command]
pub async fn preparar_sede() -> Result<(), String> {
    bloqueante(|| ajudante(&["preparar-sede"]).map(|_| ())).await
}

#[tauri::command]
pub async fn concluir() -> Result<(), String> {
    bloqueante(|| ajudante(&["concluir"]).map(|_| ())).await
}

#[tauri::command]
pub async fn reiniciar() -> Result<(), String> {
    bloqueante(|| rodar("/usr/bin/systemctl", &["reboot"]).map(|_| ())).await
}

#[tauri::command]
pub fn fechar(app: AppHandle) {
    app.exit(0);
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn le_so_as_linhas_json() {
        let linhas = ler_linhas("aviso do pkexec\n{\"evento\":\"codigo\",\"codigo\":\"R7M2PX\"}\n");
        assert_eq!(linhas.len(), 1);
        assert_eq!(linhas[0]["codigo"], "R7M2PX");
    }

    #[test]
    fn a_mensagem_do_ajudante_vem_primeiro() {
        let linhas = ler_linhas("{\"evento\":\"erro\",\"mensagem\":\"Já existe a conta luan nesta máquina.\"}");
        assert_eq!(mensagem_de_erro(&linhas, Some(1)), "Já existe a conta luan nesta máquina.");
    }

    #[test]
    fn senha_recusada_no_pkexec() {
        assert!(mensagem_de_erro(&[], Some(126)).contains("senha de administrador"));
        assert!(mensagem_de_erro(&[], Some(127)).contains("senha de administrador"));
    }
}
