//! Super + Espaço: um toque abre e ouve até o próximo toque; segurar fala e soltar escreve.
//! Igual ao Tenancy (macOS). Tempos em ms desde o início do serviço.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Entrada {
    Apertou(u64),
    Soltou(u64),
    Tique(u64),
    /// Clique no sinal de falar, "Testar agora" ou silêncio longo no modo de mãos livres.
    Clique(u64),
    Cancelar,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Acao {
    Nada,
    OuvirSegurando,
    AbrirEOuvir,
    Escrever,
    Cancelar,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Fase {
    Parado,
    Apertado(u64),
    Segurando(u64),
    MaosLivres(u64),
    EsperandoSoltar,
}

pub struct Gesto {
    fase: Fase,
    limite_ms: u64,
    teto_ms: u64,
}

impl Gesto {
    pub fn novo(limite_ms: u64) -> Self {
        Gesto {
            fase: Fase::Parado,
            limite_ms,
            teto_ms: 60_000,
        }
    }

    pub fn ouvindo(&self) -> bool {
        matches!(self.fase, Fase::Segurando(_) | Fase::MaosLivres(_))
    }

    pub fn maos_livres(&self) -> bool {
        matches!(self.fase, Fase::MaosLivres(_))
    }

    pub fn passo(&mut self, e: Entrada) -> Acao {
        use Entrada::*;
        let (fase, acao) = match (self.fase, e) {
            (_, Cancelar) => (Fase::Parado, Acao::Cancelar),
            (Fase::Parado, Apertou(t)) => (Fase::Apertado(t), Acao::Nada),
            (Fase::Parado, Clique(t)) => (Fase::MaosLivres(t), Acao::AbrirEOuvir),
            (Fase::Apertado(t0), Soltou(t)) if t.saturating_sub(t0) < self.limite_ms => {
                (Fase::MaosLivres(t), Acao::AbrirEOuvir)
            }
            (Fase::Apertado(_), Soltou(_)) => (Fase::Parado, Acao::Nada),
            (Fase::Apertado(t0), Tique(t)) if t.saturating_sub(t0) >= self.limite_ms => {
                (Fase::Segurando(t), Acao::OuvirSegurando)
            }
            (Fase::Segurando(_), Soltou(_)) => (Fase::Parado, Acao::Escrever),
            (Fase::Segurando(t0), Tique(t)) if t.saturating_sub(t0) >= self.teto_ms => {
                (Fase::Parado, Acao::Escrever)
            }
            (Fase::MaosLivres(_), Apertou(_)) => (Fase::EsperandoSoltar, Acao::Escrever),
            (Fase::MaosLivres(_), Clique(_)) => (Fase::Parado, Acao::Escrever),
            (Fase::MaosLivres(t0), Tique(t)) if t.saturating_sub(t0) >= self.teto_ms => {
                (Fase::Parado, Acao::Escrever)
            }
            (Fase::EsperandoSoltar, Soltou(_)) => (Fase::Parado, Acao::Nada),
            (f, _) => (f, Acao::Nada),
        };
        self.fase = fase;
        acao
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn um_toque_abre_e_ouve_ate_o_proximo_toque() {
        let mut g = Gesto::novo(350);
        assert_eq!(g.passo(Entrada::Apertou(0)), Acao::Nada);
        assert_eq!(g.passo(Entrada::Soltou(120)), Acao::AbrirEOuvir);
        assert!(g.ouvindo());
        assert_eq!(g.passo(Entrada::Apertou(4000)), Acao::Escrever);
        assert_eq!(g.passo(Entrada::Soltou(4100)), Acao::Nada);
        assert!(!g.ouvindo());
    }

    #[test]
    fn segurar_fala_e_soltar_escreve() {
        let mut g = Gesto::novo(350);
        g.passo(Entrada::Apertou(0));
        assert_eq!(g.passo(Entrada::Tique(200)), Acao::Nada);
        assert_eq!(g.passo(Entrada::Tique(360)), Acao::OuvirSegurando);
        assert_eq!(g.passo(Entrada::Soltou(3000)), Acao::Escrever);
    }

    #[test]
    fn sem_soltar_para_no_teto() {
        let mut g = Gesto::novo(350);
        g.passo(Entrada::Apertou(0));
        g.passo(Entrada::Tique(400));
        assert_eq!(g.passo(Entrada::Tique(60_500)), Acao::Escrever);
    }

    #[test]
    fn clique_na_barra_e_cancelar() {
        let mut g = Gesto::novo(350);
        assert_eq!(g.passo(Entrada::Clique(0)), Acao::AbrirEOuvir);
        assert_eq!(g.passo(Entrada::Cancelar), Acao::Cancelar);
        assert!(!g.ouvindo());
    }
}
