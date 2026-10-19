//! A porta `Render`: tudo que o pós-processamento precisa de um motor de renderização.

use crate::error::{Error, Res};
use crate::raster::Raster;

/// Um documento aberto, pronto para renderizar páginas (ou slides) em qualquer escala.
///
/// Implementações reais ficam no binário (`Word`, `Grid`, `Deck`); a `Fake` serve aos testes
/// do pós-processamento e do roteamento sem abrir arquivo nenhum.
pub trait Render {
    /// Como o motor chama a unidade ("página", "slide"), para as mensagens de erro.
    fn unit(&self) -> &'static str;

    /// Quantidade de páginas (ou slides) disponíveis.
    fn pages(&self) -> usize;

    /// Tamanho da página `page` (a partir de 0) em pontos (1/72 de polegada): (largura, altura).
    fn size(&mut self, page: usize) -> Res<(f64, f64)>;

    /// Renderiza a página `page` (a partir de 0) a `scale` px/pt, em RGBA pré-multiplicado.
    fn render(&mut self, page: usize, scale: f64) -> Res<Raster>;

    /// Modo de teste: os bytes do PNG nativo do motor, idênticos aos do mini correspondente
    /// (wc-min, dc-min, gc-min), sem nenhum pós-processamento.
    fn native_png(&mut self, _page: usize, _scale: f64) -> Res<Vec<u8>> {
        Err(Error::Internal("este motor não tem modo nativo".into()))
    }
}
