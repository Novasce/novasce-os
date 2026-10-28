//! /usr/bin/novasce-voltar-a-sede: o "Voltar à Sede" do menu das contas pessoais na sede.
//! Faz o mesmo que a tranca depois de 5 minutos, na hora: tranca esta conta e ativa a sessão
//! da Sede que está aberta. Escolher a Sede no Trocar usuário abriria uma segunda sessão dela,
//! que trava (passo 10 da sede, 27/10). Nenhum app fecha.
fn main() {
    #[cfg(target_os = "linux")]
    {
        use novasce_casa::config_casa::ConfigCasa;
        use novasce_casa::sistema::{conta_atual, papel, Papel};
        use novasce_sede::sessoes::{
            ativar, avisar, aviso_da_volta, como_voltar, listar, trancar, Volta,
        };
        let conta_sede = ConfigCasa::carregar().sede.conta;
        let sessoes = match listar() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Não deu para ler as sessões: {e}");
                let _ = avisar(
                    "Voltar à Sede",
                    "Não deu para achar a Sede agora. Tente de novo.",
                );
                std::process::exit(1);
            }
        };
        let volta = como_voltar(
            papel() == Papel::Sede,
            &conta_atual(),
            &conta_sede,
            &sessoes,
        );
        if let Volta::Ativar(id) = &volta {
            trancar();
            std::thread::sleep(std::time::Duration::from_secs(1));
            if let Err(e) = ativar(id) {
                eprintln!("Não deu para ativar a Sede: {e}");
                std::process::exit(1);
            }
        } else if let Some(aviso) = aviso_da_volta(&volta) {
            eprintln!("{aviso}");
            let _ = avisar("Voltar à Sede", aviso);
        }
    }
}
