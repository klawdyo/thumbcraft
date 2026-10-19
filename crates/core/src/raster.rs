//! Imagem em memória e retângulos inteiros.

use crate::error::{Error, Res};

/// Imagem RGBA de 8 bits com alfa PRÉ-MULTIPLICADO (o formato que os motores entregam).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Raster {
    pub w: u32,
    pub h: u32,
    pub px: Vec<u8>,
}

impl Raster {
    /// Imagem preenchida com uma cor (já pré-multiplicada).
    pub fn filled(w: u32, h: u32, rgba: [u8; 4]) -> Raster {
        let mut px = Vec::with_capacity(w as usize * h as usize * 4);
        for _ in 0..(w as usize * h as usize) {
            px.extend_from_slice(&rgba);
        }
        Raster { w, h, px }
    }

    /// Confere se o vetor de pixels tem o tamanho certo.
    pub fn check(&self) -> Res<()> {
        let want = self.w as usize * self.h as usize * 4;
        if self.w == 0 || self.h == 0 || self.px.len() != want {
            return Err(Error::Internal(format!("imagem inconsistente: {}x{} com {} bytes", self.w, self.h, self.px.len())));
        }
        Ok(())
    }

    /// Pixel (x, y) pré-multiplicado.
    pub fn at(&self, x: u32, y: u32) -> [u8; 4] {
        let i = (y as usize * self.w as usize + x as usize) * 4;
        [self.px[i], self.px[i + 1], self.px[i + 2], self.px[i + 3]]
    }

    /// Escreve o pixel (x, y).
    pub fn set(&mut self, x: u32, y: u32, p: [u8; 4]) {
        let i = (y as usize * self.w as usize + x as usize) * 4;
        self.px[i..i + 4].copy_from_slice(&p);
    }

    /// Região que cobre a imagem inteira.
    pub fn full(&self) -> Region {
        Region { x: 0, y: 0, w: self.w, h: self.h }
    }

    /// Todos os pixels são opacos?
    pub fn opaque(&self) -> bool {
        self.px.chunks_exact(4).all(|p| p[3] == 255)
    }

    /// Converte para alfa direto (não pré-multiplicado).
    pub fn to_straight(&self) -> Vec<u8> {
        let mut out = self.px.clone();
        for p in out.chunks_exact_mut(4) {
            let a = p[3] as u32;
            if a != 0 && a != 255 {
                for c in &mut p[..3] {
                    *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
                }
            }
        }
        out
    }
}

/// Retângulo inteiro dentro de uma imagem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Region {
    /// Região vazia (sem área)?
    pub fn empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }
}
