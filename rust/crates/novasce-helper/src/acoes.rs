//! O que o ajudante faz como root. Todo caminho passa pela `raiz` e todo comando
//! pelo `Executor`. Os comandos recebem caminhos do sistema; os arquivos são
//! gravados em `raiz` (em produção, `/`).
use novasce_core::contrato::Papel;
use novasce_core::sistema::{ler_papel, CAMINHO_PAPEL};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

pub const USERADD: &str = "/usr/sbin/useradd";
pub const PASSWD: &str = "/usr/bin/passwd";
pub const CHAGE: &str = "/usr/bin/chage";
pub const GETENT: &str = "/usr/bin/getent";
pub const CHOWN: &str = "/usr/bin/chown";
pub const RSVG: &str = "/usr/bin/rsvg-convert";
pub const SYSUSERS: &str = "/usr/bin/systemd-sysusers";
pub const TMPFILES: &str = "/usr/bin/systemd-tmpfiles";
pub const SYSTEMCTL: &str = "/usr/bin/systemctl";

pub const LOGIN_SEDE: &str = "sede";
pub const CAMINHO_PREPARADO: &str = "/etc/novasce/preparado";

pub const FONTE_SYSUSERS: &str = "/usr/share/novasce/sede/novasce-agentes.sysusers";
pub const DESTINO_SYSUSERS: &str = "/etc/sysusers.d/novasce-agentes.conf";
pub const FONTE_TMPFILES: &str = "/usr/share/novasce/sede/novasce-agentes.tmpfiles";
pub const DESTINO_TMPFILES: &str = "/etc/tmpfiles.d/novasce-agentes.conf";
pub const FONTE_SONO: &str = "/usr/share/novasce/sede/50-novasce-sede-sem-suspender.conf";
pub const DESTINO_SONO: &str = "/etc/systemd/sleep.conf.d/50-novasce-sede.conf";
pub const FONTE_AUTOLOGIN: &str = "/usr/share/novasce/sede/50-novasce-sede-autologin.conf";
pub const DESTINO_AUTOLOGIN: &str = "/etc/plasmalogin.conf.d/50-novasce-sede.conf";
pub const FONTE_TRAVA_DA_TELA: &str = "/usr/share/novasce/sede/kscreenlockerrc";
pub const FONTE_ENERGIA: &str = "/usr/share/novasce/sede/powerdevilrc";

const LOGINS_RESERVADOS: [&str; 4] = ["root", "sede", "agentes", "admin"];

#[derive(Debug, thiserror::Error)]
pub enum ErroHelper {
    #[error("O nome precisa ter de 1 a 80 letras, sem dois-pontos nem vírgula.")]
    NomeInvalido,
    #[error("Não deu para montar o nome de usuário a partir do e-mail {0}.")]
    EmailInvalido(String),
    #[error("Já existe a conta {0} nesta máquina.")]
    ContaExiste(String),
    #[error("Esta máquina não está no papel sede.")]
    NaoESede,
    #[error("O comando {programa} falhou: {detalhe}")]
    Comando { programa: String, detalhe: String },
    #[error("Não deu para gravar {caminho}: {fonte}")]
    Arquivo {
        caminho: String,
        fonte: std::io::Error,
    },
}

pub trait Executor {
    fn rodar(&self, programa: &str, args: &[&str]) -> Result<String, ErroHelper>;
}

/// Executa de verdade, sem shell no meio.
pub struct ExecutorReal;

impl Executor for ExecutorReal {
    fn rodar(&self, programa: &str, args: &[&str]) -> Result<String, ErroHelper> {
        let saida =
            Command::new(programa)
                .args(args)
                .output()
                .map_err(|e| ErroHelper::Comando {
                    programa: programa.to_string(),
                    detalhe: e.to_string(),
                })?;
        if saida.status.success() {
            Ok(String::from_utf8_lossy(&saida.stdout).into_owned())
        } else {
            Err(ErroHelper::Comando {
                programa: programa.to_string(),
                detalhe: String::from_utf8_lossy(&saida.stderr).trim().to_string(),
            })
        }
    }
}

pub struct Ambiente<'a> {
    pub raiz: PathBuf,
    pub executor: &'a dyn Executor,
}

impl Ambiente<'_> {
    pub fn caminho(&self, absoluto: &str) -> PathBuf {
        self.raiz.join(absoluto.trim_start_matches('/'))
    }

    fn rodar(&self, programa: &str, args: &[&str]) -> Result<String, ErroHelper> {
        self.executor.rodar(programa, args)
    }

    fn existe_usuario(&self, login: &str) -> bool {
        self.rodar(GETENT, &["passwd", login]).is_ok()
    }

    fn casa_de(&self, login: &str) -> Result<String, ErroHelper> {
        let linha = self.rodar(GETENT, &["passwd", login])?;
        linha
            .trim()
            .split(':')
            .nth(5)
            .filter(|casa| casa.starts_with('/'))
            .map(str::to_string)
            .ok_or_else(|| ErroHelper::Comando {
                programa: GETENT.to_string(),
                detalhe: format!("sem pasta pessoal para {login}"),
            })
    }

    fn gravar(&self, destino: &str, conteudo: &[u8], modo: u32) -> Result<(), ErroHelper> {
        let caminho = self.caminho(destino);
        let erro = |fonte| ErroHelper::Arquivo {
            caminho: destino.to_string(),
            fonte,
        };
        if let Some(pasta) = caminho.parent() {
            fs::create_dir_all(pasta).map_err(erro)?;
        }
        fs::write(&caminho, conteudo).map_err(erro)?;
        fs::set_permissions(&caminho, fs::Permissions::from_mode(modo)).map_err(erro)
    }

    pub fn copiar(&self, origem: &str, destino: &str, modo: u32) -> Result<(), ErroHelper> {
        let conteudo = fs::read(self.caminho(origem)).map_err(|fonte| ErroHelper::Arquivo {
            caminho: origem.to_string(),
            fonte,
        })?;
        self.gravar(destino, &conteudo, modo)
    }

    fn remover(&self, destino: &str) -> Result<(), ErroHelper> {
        match fs::remove_file(self.caminho(destino)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(fonte) => Err(ErroHelper::Arquivo {
                caminho: destino.to_string(),
                fonte,
            }),
        }
    }
}

/// Grava o papel. Na sede, cria o usuário `agentes` e tira a suspensão.
/// No notebook, tira o que era só da sede (reinstalar um notebook que voltou).
pub fn definir_papel(amb: &Ambiente, papel: Papel) -> Result<(), ErroHelper> {
    amb.gravar(
        CAMINHO_PAPEL,
        format!("{}\n", papel.como_texto()).as_bytes(),
        0o644,
    )?;
    match papel {
        Papel::Sede => {
            amb.copiar(FONTE_SYSUSERS, DESTINO_SYSUSERS, 0o644)?;
            amb.copiar(FONTE_TMPFILES, DESTINO_TMPFILES, 0o644)?;
            amb.copiar(FONTE_SONO, DESTINO_SONO, 0o644)?;
            amb.rodar(SYSUSERS, &[DESTINO_SYSUSERS])?;
            amb.rodar(TMPFILES, &["--create", DESTINO_TMPFILES])?;
        }
        Papel::Notebook => {
            for destino in [
                DESTINO_SYSUSERS,
                DESTINO_TMPFILES,
                DESTINO_SONO,
                DESTINO_AUTOLOGIN,
            ] {
                amb.remover(destino)?;
            }
        }
    }
    Ok(())
}

pub fn validar_nome(nome: &str) -> Result<(), ErroHelper> {
    let nome = nome.trim();
    let invalido = nome.is_empty()
        || nome.chars().count() > 80
        || nome.chars().any(|c| c == ':' || c == ',' || c.is_control());
    if invalido {
        Err(ErroHelper::NomeInvalido)
    } else {
        Ok(())
    }
}

/// "luan.silva@novasce.com" vira "luan-silva".
pub fn login_para(email: &str) -> Result<String, ErroHelper> {
    let invalido = || ErroHelper::EmailInvalido(email.to_string());
    let (local, _) = email.trim().split_once('@').ok_or_else(invalido)?;
    let mut login = String::new();
    for letra in local.chars().flat_map(char::to_lowercase).map(sem_acento) {
        if letra.is_ascii_lowercase() || letra.is_ascii_digit() {
            login.push(letra);
        } else if matches!(letra, '.' | '_' | '-') && !login.is_empty() && !login.ends_with('-') {
            login.push('-');
        }
    }
    login.truncate(32);
    while login.ends_with('-') {
        login.pop();
    }
    let comeca_com_letra = login.chars().next().is_some_and(|c| c.is_ascii_lowercase());
    if comeca_com_letra && !LOGINS_RESERVADOS.contains(&login.as_str()) {
        Ok(login)
    } else {
        Err(invalido())
    }
}

fn sem_acento(letra: char) -> char {
    match letra {
        'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        'ñ' => 'n',
        outra => outra,
    }
}

/// Confere tudo antes de qualquer efeito: nome, login e se a conta já existe.
pub fn conferir_conta_nova(amb: &Ambiente, nome: &str, email: &str) -> Result<String, ErroHelper> {
    validar_nome(nome)?;
    let login = login_para(email)?;
    if amb.existe_usuario(&login) {
        return Err(ErroHelper::ContaExiste(login));
    }
    Ok(login)
}

/// Conta pessoal: sem `wheel` (sem sudo), sem senha e com a senha vencida,
/// para a pessoa criar a dela no primeiro login.
pub fn criar_conta_pessoal(amb: &Ambiente, nome: &str, email: &str) -> Result<String, ErroHelper> {
    let login = conferir_conta_nova(amb, nome, email)?;
    let nome = nome.trim();
    amb.rodar(USERADD, &["-m", "-c", nome, "-s", "/bin/bash", &login])?;
    amb.rodar(PASSWD, &["-d", &login])?;
    amb.rodar(CHAGE, &["-d", "0", &login])?;
    gravar_avatar(amb, &login, nome)?;
    Ok(login)
}

/// Avatar com a inicial no círculo teal claro, como o login do canvas (prancha 2).
pub fn svg_do_avatar(nome: &str) -> String {
    let inicial: String = nome
        .trim()
        .chars()
        .next()
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_else(|| "?".to_string());
    let inicial = inicial
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 96 96\">\
<circle cx=\"48\" cy=\"48\" r=\"48\" fill=\"#E8F2F2\"/>\
<text x=\"48\" y=\"60\" text-anchor=\"middle\" font-family=\"Inter, sans-serif\" font-weight=\"600\" font-size=\"34\" fill=\"#00666C\">{inicial}</text></svg>"
    )
}

fn gravar_avatar(amb: &Ambiente, login: &str, nome: &str) -> Result<(), ErroHelper> {
    let svg = format!("/var/lib/AccountsService/icons/{login}.svg");
    let png = format!("/var/lib/AccountsService/icons/{login}");
    amb.gravar(&svg, svg_do_avatar(nome).as_bytes(), 0o644)?;
    amb.rodar(RSVG, &["-w", "256", "-h", "256", "-o", &png, &svg])?;
    amb.remover(&svg)?;
    let usuario = format!("[User]\nIcon={png}\n");
    amb.gravar(
        &format!("/var/lib/AccountsService/users/{login}"),
        usuario.as_bytes(),
        0o600,
    )?;
    // Se o accounts-daemon não estiver rodando, o avatar aparece no próximo boot.
    let _ = amb.rodar(SYSTEMCTL, &["try-restart", "accounts-daemon.service"]);
    Ok(())
}

/// Conta Sede: entra sozinha ao ligar, sem senha, sem sudo, tela que não tranca
/// e sem suspender. Rodar de novo não recria a conta.
pub fn preparar_sede(amb: &Ambiente) -> Result<(), ErroHelper> {
    if ler_papel(&amb.caminho(CAMINHO_PAPEL)) != Some(Papel::Sede) {
        return Err(ErroHelper::NaoESede);
    }
    if !amb.existe_usuario(LOGIN_SEDE) {
        amb.rodar(
            USERADD,
            &["-m", "-c", "Sede", "-s", "/bin/bash", LOGIN_SEDE],
        )?;
    }
    amb.rodar(PASSWD, &["-d", LOGIN_SEDE])?;
    amb.copiar(FONTE_AUTOLOGIN, DESTINO_AUTOLOGIN, 0o644)?;
    let config = format!("{}/.config", amb.casa_de(LOGIN_SEDE)?);
    amb.copiar(
        FONTE_TRAVA_DA_TELA,
        &format!("{config}/kscreenlockerrc"),
        0o644,
    )?;
    amb.copiar(FONTE_ENERGIA, &format!("{config}/powerdevilrc"), 0o644)?;
    amb.rodar(CHOWN, &["-R", "sede:sede", &config])?;
    Ok(())
}

/// A máquina está preparada: o Preparar não abre mais sozinho no login.
pub fn concluir(amb: &Ambiente) -> Result<(), ErroHelper> {
    amb.gravar(CAMINHO_PREPARADO, b"ok\n", 0o644)
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::cell::RefCell;
    use std::path::Path;

    /// Executor falso: anota cada comando e simula o `getent` e o `useradd`.
    struct Falso {
        chamadas: RefCell<Vec<String>>,
        usuarios: RefCell<Vec<String>>,
    }

    impl Falso {
        fn com(usuarios: &[&str]) -> Self {
            Self {
                chamadas: RefCell::new(Vec::new()),
                usuarios: RefCell::new(usuarios.iter().map(|u| u.to_string()).collect()),
            }
        }
        fn chamou(&self, comando: &str) -> bool {
            self.chamadas.borrow().iter().any(|c| c == comando)
        }
    }

    impl Executor for Falso {
        fn rodar(&self, programa: &str, args: &[&str]) -> Result<String, ErroHelper> {
            self.chamadas
                .borrow_mut()
                .push(format!("{programa} {}", args.join(" ")));
            match programa {
                GETENT => {
                    let login = args[1];
                    if self.usuarios.borrow().iter().any(|u| u == login) {
                        Ok(format!(
                            "{login}:x:1001:1001:{login}:/home/{login}:/bin/bash\n"
                        ))
                    } else {
                        Err(ErroHelper::Comando {
                            programa: GETENT.into(),
                            detalhe: "não existe".into(),
                        })
                    }
                }
                USERADD => {
                    self.usuarios
                        .borrow_mut()
                        .push(args.last().unwrap().to_string());
                    Ok(String::new())
                }
                _ => Ok(String::new()),
            }
        }
    }

    fn raiz_com_a_imagem() -> tempfile::TempDir {
        let raiz = tempfile::tempdir().unwrap();
        for fonte in [
            FONTE_SYSUSERS,
            FONTE_TMPFILES,
            FONTE_SONO,
            FONTE_AUTOLOGIN,
            FONTE_TRAVA_DA_TELA,
            FONTE_ENERGIA,
        ] {
            let caminho = raiz.path().join(fonte.trim_start_matches('/'));
            fs::create_dir_all(caminho.parent().unwrap()).unwrap();
            fs::write(&caminho, format!("conteúdo de {fonte}\n")).unwrap();
        }
        raiz
    }

    fn ler(raiz: &Path, absoluto: &str) -> String {
        fs::read_to_string(raiz.join(absoluto.trim_start_matches('/'))).unwrap()
    }

    #[test]
    fn papel_sede_liga_os_agentes_e_tira_o_sono() {
        let raiz = raiz_com_a_imagem();
        let falso = Falso::com(&[]);
        let amb = Ambiente {
            raiz: raiz.path().to_path_buf(),
            executor: &falso,
        };
        definir_papel(&amb, Papel::Sede).unwrap();
        assert_eq!(ler(raiz.path(), CAMINHO_PAPEL), "sede\n");
        assert_eq!(
            ler(raiz.path(), DESTINO_SYSUSERS),
            format!("conteúdo de {FONTE_SYSUSERS}\n")
        );
        assert_eq!(
            ler(raiz.path(), DESTINO_SONO),
            format!("conteúdo de {FONTE_SONO}\n")
        );
        assert!(falso.chamou(&format!("{SYSUSERS} {DESTINO_SYSUSERS}")));
        assert!(falso.chamou(&format!("{TMPFILES} --create {DESTINO_TMPFILES}")));
    }

    #[test]
    fn papel_notebook_tira_o_que_era_da_sede() {
        let raiz = raiz_com_a_imagem();
        let falso = Falso::com(&[]);
        let amb = Ambiente {
            raiz: raiz.path().to_path_buf(),
            executor: &falso,
        };
        definir_papel(&amb, Papel::Sede).unwrap();
        amb.copiar(FONTE_AUTOLOGIN, DESTINO_AUTOLOGIN, 0o644)
            .unwrap();
        definir_papel(&amb, Papel::Notebook).unwrap();
        assert_eq!(ler(raiz.path(), CAMINHO_PAPEL), "notebook\n");
        for destino in [
            DESTINO_SYSUSERS,
            DESTINO_TMPFILES,
            DESTINO_SONO,
            DESTINO_AUTOLOGIN,
        ] {
            assert!(
                !raiz.path().join(destino.trim_start_matches('/')).exists(),
                "{destino} ficou"
            );
        }
    }

    #[test]
    fn login_sai_do_email() {
        assert_eq!(login_para("luan.silva@novasce.com").unwrap(), "luan-silva");
        assert_eq!(login_para("Ana_Paula@novasce.com").unwrap(), "ana-paula");
        assert_eq!(login_para("joão@novasce.com").unwrap(), "joao");
        assert!(login_para("1abc@novasce.com").is_err());
        assert!(login_para("sem-arroba").is_err());
        assert!(login_para("sede@novasce.com").is_err());
    }

    #[test]
    fn nome_sem_separador_do_passwd() {
        assert!(validar_nome("Luan Silva").is_ok());
        assert!(validar_nome("  ").is_err());
        assert!(validar_nome("Luan:Silva").is_err());
        assert!(validar_nome("Luan, Silva").is_err());
    }

    #[test]
    fn conta_pessoal_sem_sudo_e_com_senha_no_primeiro_login() {
        let raiz = raiz_com_a_imagem();
        let falso = Falso::com(&[]);
        let amb = Ambiente {
            raiz: raiz.path().to_path_buf(),
            executor: &falso,
        };
        let login = criar_conta_pessoal(&amb, "Luan Silva", "luan.silva@novasce.com").unwrap();
        assert_eq!(login, "luan-silva");
        assert!(falso.chamou(&format!(
            "{USERADD} -m -c Luan Silva -s /bin/bash luan-silva"
        )));
        assert!(falso.chamou(&format!("{PASSWD} -d luan-silva")));
        assert!(falso.chamou(&format!("{CHAGE} -d 0 luan-silva")));
        assert!(!falso.chamadas.borrow().iter().any(|c| c.contains("wheel")));
        assert_eq!(
            ler(raiz.path(), "/var/lib/AccountsService/users/luan-silva"),
            "[User]\nIcon=/var/lib/AccountsService/icons/luan-silva\n"
        );
    }

    #[test]
    fn conta_que_ja_existe_nao_e_tocada() {
        let raiz = raiz_com_a_imagem();
        let falso = Falso::com(&["luan-silva"]);
        let amb = Ambiente {
            raiz: raiz.path().to_path_buf(),
            executor: &falso,
        };
        let erro = conferir_conta_nova(&amb, "Luan Silva", "luan.silva@novasce.com").unwrap_err();
        assert!(matches!(erro, ErroHelper::ContaExiste(l) if l == "luan-silva"));
        assert!(!falso
            .chamadas
            .borrow()
            .iter()
            .any(|c| c.starts_with(USERADD)));
    }

    #[test]
    fn avatar_com_a_inicial_no_circulo_teal() {
        let svg = svg_do_avatar("élida souza");
        assert!(svg.contains(">É</text>"));
        assert!(svg.contains("#E8F2F2") && svg.contains("#00666C"));
        assert!(svg_do_avatar("<x>").contains("&lt;"));
    }

    #[test]
    fn sede_so_no_papel_sede() {
        let raiz = raiz_com_a_imagem();
        let falso = Falso::com(&[]);
        let amb = Ambiente {
            raiz: raiz.path().to_path_buf(),
            executor: &falso,
        };
        definir_papel(&amb, Papel::Notebook).unwrap();
        assert!(matches!(preparar_sede(&amb), Err(ErroHelper::NaoESede)));
    }

    #[test]
    fn preparar_sede_cria_a_conta_e_liga_a_entrada_automatica() {
        let raiz = raiz_com_a_imagem();
        let falso = Falso::com(&[]);
        let amb = Ambiente {
            raiz: raiz.path().to_path_buf(),
            executor: &falso,
        };
        definir_papel(&amb, Papel::Sede).unwrap();
        preparar_sede(&amb).unwrap();
        assert!(falso.chamou(&format!("{USERADD} -m -c Sede -s /bin/bash sede")));
        assert!(falso.chamou(&format!("{PASSWD} -d sede")));
        assert_eq!(
            ler(raiz.path(), DESTINO_AUTOLOGIN),
            format!("conteúdo de {FONTE_AUTOLOGIN}\n")
        );
        assert_eq!(
            ler(raiz.path(), "/home/sede/.config/kscreenlockerrc"),
            format!("conteúdo de {FONTE_TRAVA_DA_TELA}\n")
        );
        assert_eq!(
            ler(raiz.path(), "/home/sede/.config/powerdevilrc"),
            format!("conteúdo de {FONTE_ENERGIA}\n")
        );
        assert!(falso.chamou(&format!("{CHOWN} -R sede:sede /home/sede/.config")));
    }

    #[test]
    fn preparar_sede_de_novo_nao_recria_a_conta() {
        let raiz = raiz_com_a_imagem();
        let falso = Falso::com(&["sede"]);
        let amb = Ambiente {
            raiz: raiz.path().to_path_buf(),
            executor: &falso,
        };
        definir_papel(&amb, Papel::Sede).unwrap();
        preparar_sede(&amb).unwrap();
        assert!(!falso
            .chamadas
            .borrow()
            .iter()
            .any(|c| c.starts_with(USERADD)));
    }

    #[test]
    fn concluir_marca_a_maquina() {
        let raiz = raiz_com_a_imagem();
        let falso = Falso::com(&[]);
        let amb = Ambiente {
            raiz: raiz.path().to_path_buf(),
            executor: &falso,
        };
        concluir(&amb).unwrap();
        assert_eq!(ler(raiz.path(), CAMINHO_PREPARADO), "ok\n");
    }
}
