//! Corpo multipart/form-data para POST /os/tickets (texto, print e dados da máquina).

pub enum Parte<'a> {
    Texto {
        nome: &'a str,
        valor: &'a str,
    },
    Arquivo {
        nome: &'a str,
        arquivo: &'a str,
        tipo: &'a str,
        dados: &'a [u8],
    },
}

pub fn corpo(partes: &[Parte], fronteira: &str) -> (String, Vec<u8>) {
    let mut b = Vec::new();
    for p in partes {
        b.extend_from_slice(format!("--{fronteira}\r\n").as_bytes());
        match p {
            Parte::Texto { nome, valor } => {
                b.extend_from_slice(
                    format!("Content-Disposition: form-data; name=\"{nome}\"\r\n\r\n").as_bytes(),
                );
                b.extend_from_slice(valor.as_bytes());
            }
            Parte::Arquivo {
                nome,
                arquivo,
                tipo,
                dados,
            } => {
                b.extend_from_slice(
                    format!("Content-Disposition: form-data; name=\"{nome}\"; filename=\"{arquivo}\"\r\nContent-Type: {tipo}\r\n\r\n")
                        .as_bytes(),
                );
                b.extend_from_slice(dados);
            }
        }
        b.extend_from_slice(b"\r\n");
    }
    b.extend_from_slice(format!("--{fronteira}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={fronteira}"), b)
}

pub fn fronteira_nova() -> String {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("novasce-{n:x}")
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn monta_texto_e_arquivo() {
        let (tipo, corpo) = corpo(
            &[
                Parte::Texto {
                    nome: "text",
                    valor: "A impressora sumiu.",
                },
                Parte::Arquivo {
                    nome: "screenshot",
                    arquivo: "print.png",
                    tipo: "image/png",
                    dados: b"PNG",
                },
            ],
            "xyz",
        );
        assert_eq!(tipo, "multipart/form-data; boundary=xyz");
        let s = String::from_utf8(corpo).unwrap();
        assert!(s.starts_with(
            "--xyz\r\nContent-Disposition: form-data; name=\"text\"\r\n\r\nA impressora sumiu.\r\n"
        ));
        assert!(s.contains(
            "name=\"screenshot\"; filename=\"print.png\"\r\nContent-Type: image/png\r\n\r\nPNG\r\n"
        ));
        assert!(s.ends_with("--xyz--\r\n"));
    }
}
