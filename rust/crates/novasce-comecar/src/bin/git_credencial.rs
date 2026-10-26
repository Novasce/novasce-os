//! /usr/libexec/novasce/git-credencial: o git pergunta, o chaveiro responde. Só para github.com.
use novasce_casa::caminhos::Caminhos;
use novasce_casa::chaveiro::{Chaveiro, ChaveiroSistema, CONTA_GITHUB};
use std::io::Read;

fn main() {
    let operacao = std::env::args().nth(1).unwrap_or_default();
    let mut entrada = String::new();
    let _ = std::io::stdin().read_to_string(&mut entrada);
    let token = if operacao == "get" {
        ChaveiroSistema::da_pessoa(&Caminhos::da_pessoa()).ler(CONTA_GITHUB)
    } else {
        None
    };
    print!(
        "{}",
        novasce_comecar::brain::resposta_credencial(&operacao, &entrada, token.as_deref())
    );
}
