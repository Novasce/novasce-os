mod comandos;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            comandos::placa_de_video,
            comandos::estado_da_rede,
            comandos::redes_wifi,
            comandos::conectar_wifi,
            comandos::definir_papel,
            comandos::registrar,
            comandos::listar_time,
            comandos::criar_conta,
            comandos::preparar_sede,
            comandos::concluir,
            comandos::reiniciar,
            comandos::fechar,
        ])
        .run(tauri::generate_context!())
        .expect("o Preparar não abriu");
}
