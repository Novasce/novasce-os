//! A única porta do M3 para o novasce-core (M1). Os nomes do M3 nascem aqui.
pub use crate::cliente::{ErroTenancy, Tenancy};
pub use novasce_core::config::Config;
pub use novasce_core::contrato::{
    Area, ChamadoAberto as ChamadoCriado, Comecar, DispositivoIniciado as DeviceInicio, Espaco,
    Kit, Maquina, Pessoa as DonoMaquina, PessoaComPapel as UsuarioResumo, PrimeiraTarefa,
    RespostaDoChamado as RespostaChamado, TipoDeResposta as TipoResposta,
    TokenDaPessoa as DeviceToken,
};
pub use novasce_core::token::{
    MaquinaSalva as TokenMaquina, CAMINHO_DA_MAQUINA as CAMINHO_MAQUINA,
};

/// Lê o token da máquina pela guarda do M1 (recusa arquivo aberto para outros e token sem `nvos_m_`).
pub fn ler_maquina(caminho: &std::path::Path) -> Result<TokenMaquina, String> {
    novasce_core::token::GuardaDoToken::em(caminho)
        .ler()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "esta máquina ainda não foi registrada".to_string())
}
