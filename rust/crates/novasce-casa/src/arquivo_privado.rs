//! Segredo em arquivo que só a pessoa lê. Na linha de comando ele vazaria: sem `hidepid`, qualquer
//! conta da máquina lê /proc/<pid>/cmdline, e a sede tem várias contas (inclusive a Sede, aberta).
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::{fs, io};

/// Grava `conteudo` em `caminho` com permissão 0600, de uma vez: escreve num arquivo vizinho e troca de
/// nome, então ninguém lê o arquivo pela metade nem com a permissão do arquivo antigo. Pasta que não
/// existe nasce 0700; as que já existem ficam como estão. Se `caminho` for um link, o alvo é que troca.
pub fn gravar(caminho: &Path, conteudo: &[u8]) -> io::Result<()> {
    let alvo = fs::canonicalize(caminho).unwrap_or_else(|_| caminho.to_path_buf());
    let pasta = alvo
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&pasta)?;
    let nome = alvo
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "caminho sem nome de arquivo"))?
        .to_string_lossy();
    let novo = pasta.join(format!(".{nome}.{}.novo", std::process::id()));
    let _ = fs::remove_file(&novo);
    let escrito = (|| {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&novo)?;
        fs::set_permissions(&novo, fs::Permissions::from_mode(0o600))?;
        f.write_all(conteudo)?;
        f.sync_all()?;
        fs::rename(&novo, &alvo)
    })();
    if escrito.is_err() {
        let _ = fs::remove_file(&novo);
    }
    escrito
}

#[cfg(test)]
mod testes {
    use super::*;

    fn modo(p: &Path) -> u32 {
        fs::metadata(p).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn arquivo_novo_sai_0600_e_pasta_nova_0700() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a/b/segredo.conf");
        gravar(&p, b"um").unwrap();
        assert_eq!(fs::read(&p).unwrap(), b"um");
        assert_eq!(modo(&p), 0o600);
        assert_eq!(modo(p.parent().unwrap()), 0o700);
    }

    #[test]
    fn troca_o_arquivo_aberto_e_nao_deixa_resto() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("segredo.conf");
        fs::write(&p, "velho").unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o644)).unwrap();
        gravar(&p, b"novo").unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), "novo");
        assert_eq!(modo(&p), 0o600);
        let restos: Vec<_> = fs::read_dir(d.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(restos, vec!["segredo.conf"]);
    }

    #[test]
    fn pasta_que_ja_existe_nao_muda_de_permissao() {
        let d = tempfile::tempdir().unwrap();
        fs::set_permissions(d.path(), fs::Permissions::from_mode(0o755)).unwrap();
        gravar(&d.path().join("x"), b"1").unwrap();
        assert_eq!(modo(d.path()), 0o755);
    }

    #[test]
    fn link_continua_link_e_o_alvo_e_que_troca() {
        let d = tempfile::tempdir().unwrap();
        let real = d.path().join("real.conf");
        let link = d.path().join("link.conf");
        fs::write(&real, "velho").unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();
        gravar(&link, b"novo").unwrap();
        assert!(fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(fs::read_to_string(&real).unwrap(), "novo");
        assert_eq!(modo(&real), 0o600);
    }
}
