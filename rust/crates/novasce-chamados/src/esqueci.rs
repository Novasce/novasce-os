//! "Esqueci a senha": o QR abre `${TENANCY_WEB}/os/esqueci?m=<id>` (contrato v2), um chamado já dizendo de qual máquina é.
use qrcode::QrCode;

pub fn url_esqueci(web: &str, machine_id: &str) -> String {
    format!("{}/os/esqueci?m={}", web.trim_end_matches('/'), machine_id)
}

fn primeira_minuscula(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|p| p.to_lowercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

pub fn textos(nome_maquina: &str, primeiro_nome: Option<&str>) -> (String, String) {
    let titulo = match primeiro_nome {
        Some(n) if !n.is_empty() => format!("Esqueceu a senha, {n}?"),
        _ => "Esqueceu a senha?".into(),
    };
    let linha = format!(
        "Só o Eduardo troca a senha deste computador. Aponte o celular para o código: ele abre um chamado no Tenancy já dizendo que é o {}.",
        primeira_minuscula(nome_maquina)
    );
    (titulo, linha)
}

pub struct Modulos {
    pub lado: usize,
    pub escuros: Vec<bool>,
}

pub fn modulos(url: &str) -> Result<Modulos, String> {
    let q = QrCode::new(url.as_bytes()).map_err(|e| e.to_string())?;
    let lado = q.width();
    let escuros = q
        .to_colors()
        .into_iter()
        .map(|c| c == qrcode::Color::Dark)
        .collect();
    Ok(Modulos { lado, escuros })
}

/// SVG quadrado com margem de 4 módulos, preto sobre branco (lê em qualquer celular).
pub fn qr_svg(url: &str, tamanho: u32) -> Result<String, String> {
    let m = modulos(url)?;
    let total = m.lado + 8;
    let mut s = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{tamanho}" height="{tamanho}" viewBox="0 0 {total} {total}" shape-rendering="crispEdges"><rect width="{total}" height="{total}" fill="#fff"/>"##
    );
    for (i, escuro) in m.escuros.iter().enumerate() {
        if *escuro {
            s.push_str(&format!(
                r##"<rect x="{}" y="{}" width="1" height="1" fill="#1D1D1F"/>"##,
                i % m.lado + 4,
                i / m.lado + 4
            ));
        }
    }
    s.push_str("</svg>");
    Ok(s)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn url_leva_a_maquina() {
        assert_eq!(
            url_esqueci("https://tenancy.novasce.com/", "m-123"),
            "https://tenancy.novasce.com/os/esqueci?m=m-123"
        );
    }

    #[test]
    fn textos_com_e_sem_dono() {
        let (t, l) = textos("Notebook do Luan", Some("Luan"));
        assert_eq!(t, "Esqueceu a senha, Luan?");
        assert_eq!(l, "Só o Eduardo troca a senha deste computador. Aponte o celular para o código: ele abre um chamado no Tenancy já dizendo que é o notebook do Luan.");
        assert_eq!(textos("Sede", None).0, "Esqueceu a senha?");
    }

    #[test]
    fn qr_em_modulos() {
        let m = modulos("https://t/os/esqueci?m=m-1").unwrap();
        assert!(m.lado >= 21);
        assert_eq!(m.escuros.len(), m.lado * m.lado);
        let svg = qr_svg("https://t/os/esqueci?m=m-1", 156).unwrap();
        assert!(svg.starts_with("<svg") && svg.contains("<rect"));
    }
}
