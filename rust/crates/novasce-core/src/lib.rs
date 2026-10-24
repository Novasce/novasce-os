//! Núcleo do Novasce OS: config, contrato com o Tenancy, guarda do token e
//! leituras do sistema. Os binários e os apps em Tauri usam só o que está aqui.

pub mod config;

/// Versão do núcleo, a mesma do Cargo.
pub const VERSAO: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod testes {
    #[test]
    fn versao_vem_do_cargo() {
        assert_eq!(super::VERSAO, "0.1.0");
    }
}
