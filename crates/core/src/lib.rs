//! Núcleo do thumbcraft, sem dependências de motores.
//!
//! - [`render::Render`]: a porta que cada motor implementa (e [`fake::Fake`] para testes);
//! - [`route`]: escolha do motor por extensão e conteúdo;
//! - [`scale`]: escolha da escala de renderização (supersampling) e limites;
//! - [`fit`], [`trim`], [`resize`]: matemática de ajuste, margens e reamostragem;
//! - [`post`]: o pipeline completo (render, crop, trim, fit, resize).

pub mod error;
pub mod fake;
pub mod fit;
pub mod opts;
pub mod post;
pub mod raster;
pub mod render;
pub mod resize;
pub mod route;
pub mod scale;
pub mod trim;

pub use error::{Error, Res};
pub use opts::Opts;
pub use raster::{Raster, Region};
pub use render::Render;
