//! Fundo do login com o cartão "Esqueceu a senha?" e o QR da máquina. Renderizado com rsvg-convert.
use crate::esqueci::modulos;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn quebrar(texto: &str, largura: usize) -> Vec<String> {
    let mut linhas = vec![String::new()];
    for p in texto.split_whitespace() {
        let atual = linhas.last_mut().unwrap();
        if !atual.is_empty() && atual.chars().count() + 1 + p.chars().count() > largura {
            linhas.push(p.to_string());
        } else {
            if !atual.is_empty() {
                atual.push(' ');
            }
            atual.push_str(p);
        }
    }
    linhas
}

/// Cartão branco no canto inferior direito, com QR, título e linha. Medidas proporcionais à tela.
pub fn svg_fundo(
    papel: &str,
    url: &str,
    titulo: &str,
    linha: &str,
    w: u32,
    h: u32,
) -> Result<String, String> {
    let m = modulos(url)?;
    let u = w as f64 / 1280.0;
    let (cw, ch) = (520.0 * u, 200.0 * u);
    let (cx, cy) = (w as f64 - cw - 40.0 * u, h as f64 - ch - 40.0 * u);
    let lado_qr = 156.0 * u;
    let modulo = lado_qr / (m.lado as f64 + 8.0);
    let mut s = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><image href="file://{papel}" x="0" y="0" width="{w}" height="{h}" preserveAspectRatio="xMidYMid slice"/><rect x="{cx:.1}" y="{cy:.1}" width="{cw:.1}" height="{ch:.1}" rx="{r:.1}" fill="#fff"/>"##,
        r = 8.0 * u
    );
    let (qx, qy) = (cx + 22.0 * u, cy + 22.0 * u);
    for (i, escuro) in m.escuros.iter().enumerate() {
        if *escuro {
            let (x, y) = ((i % m.lado + 4) as f64, (i / m.lado + 4) as f64);
            s.push_str(&format!(r##"<rect x="{:.2}" y="{:.2}" width="{modulo:.2}" height="{modulo:.2}" fill="#1D1D1F"/>"##, qx + x * modulo, qy + y * modulo));
        }
    }
    let tx = qx + lado_qr + 22.0 * u;
    s.push_str(&format!(r##"<text x="{tx:.1}" y="{:.1}" font-family="Inter" font-weight="700" font-size="{:.1}" fill="#1D1D1F">{}</text>"##, cy + 52.0 * u, 20.0 * u, esc(titulo)));
    for (n, l) in quebrar(linha, 34).iter().enumerate() {
        s.push_str(&format!(r##"<text x="{tx:.1}" y="{:.1}" font-family="Inter" font-size="{:.1}" fill="#3B4446">{}</text>"##, cy + (84.0 + 20.0 * n as f64) * u, 14.0 * u, esc(l)));
    }
    s.push_str("</svg>");
    Ok(s)
}

pub fn conf_greeter(png: &str) -> String {
    format!("[Greeter][Wallpaper][org.kde.image][General]\nImage=file://{png}\nPreviewImage=file://{png}\n")
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn svg_do_fundo_tem_papel_qr_e_textos() {
        let svg = svg_fundo(
            "/usr/share/wallpapers/Novasce/contents/images/3840x2160.png",
            "https://t/os/esqueci?m=m-1",
            "Esqueceu a senha, Luan?",
            "Aponte o celular & veja.",
            3840,
            2160,
        )
        .unwrap();
        assert!(svg.contains(
            r#"href="file:///usr/share/wallpapers/Novasce/contents/images/3840x2160.png""#
        ));
        assert!(svg.contains("Esqueceu a senha, Luan?"));
        assert!(svg.contains("Aponte o celular &amp; veja."));
        assert!(svg.matches("<rect").count() > 100);
        assert!(!svg.contains('\u{2014}'));
    }

    #[test]
    fn configuracao_do_greeter() {
        assert_eq!(conf_greeter("/var/lib/novasce/login/fundo.png"), "[Greeter][Wallpaper][org.kde.image][General]\nImage=file:///var/lib/novasce/login/fundo.png\nPreviewImage=file:///var/lib/novasce/login/fundo.png\n");
    }

    #[test]
    fn quebra_a_linha_em_pedacos() {
        let l = quebrar(
            "Só o Eduardo troca a senha deste computador. Aponte o celular para o código.",
            30,
        );
        assert!(l.iter().all(|x| x.chars().count() <= 30));
        assert_eq!(
            l.join(" "),
            "Só o Eduardo troca a senha deste computador. Aponte o celular para o código."
        );
    }
}
