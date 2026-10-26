//! Janelas flutuantes da casa como camada wlr-layer-shell (KWin oferece): por cima de tudo,
//! presas a uma borda e, quando pedido, sem roubar o foco do campo onde a pessoa está.
//! Chamar ANTES de mostrar a janela (ela nasce com "visible": false).
#[cfg(all(target_os = "linux", feature = "gtk"))]
pub use gtk_layer_shell::{Edge, KeyboardMode};

#[cfg(all(target_os = "linux", feature = "gtk"))]
pub fn ancorar(janela: &gtk::Window, bordas: &[(Edge, i32)], teclado: KeyboardMode) {
    use gtk_layer_shell::{Layer, LayerShell};
    janela.init_layer_shell();
    janela.set_layer(Layer::Overlay);
    janela.set_namespace("novasce");
    janela.set_keyboard_mode(teclado);
    for (borda, margem) in bordas {
        janela.set_anchor(*borda, true);
        janela.set_layer_shell_margin(*borda, *margem);
    }
}
