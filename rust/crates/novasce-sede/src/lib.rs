//! A sede: volta à Sala, conta pessoal que tranca, navegador que esquece e reiniciar com segurança.
pub mod kwin;
pub mod navegador;
#[cfg(target_os = "linux")]
pub mod ociosidade;
pub mod sessoes;
