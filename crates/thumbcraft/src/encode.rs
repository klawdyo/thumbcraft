//! Codificação da imagem final (PNG; JPEG e WebP quando compilados).

use thumbcraft_core::{Error, Raster, Res};

/// Formato de saída.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Png,
}

impl Format {
    /// Formato pela extensão do arquivo de saída (padrão PNG).
    pub fn from_ext(ext: &str) -> Res<Format> {
        match ext {
            "png" | "" => Ok(Format::Png),
            e => Err(Error::Usage(format!("formato de saída não suportado: .{e} (use .png)"))),
        }
    }
}

/// PNG de 8 bits: RGB quando a imagem é opaca, RGBA (alfa direto) caso contrário.
pub fn png(img: &Raster) -> Res<Vec<u8>> {
    let bad = |e: png::EncodingError| Error::Internal(format!("falha ao codificar o PNG: {e}"));
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, img.w, img.h);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::Balanced);
        enc.set_filter(png::Filter::Adaptive);
        if img.opaque() {
            enc.set_color(png::ColorType::Rgb);
            let mut data = Vec::with_capacity(img.w as usize * img.h as usize * 3);
            for p in img.px.chunks_exact(4) {
                data.extend_from_slice(&p[..3]);
            }
            enc.write_header().map_err(bad)?.write_image_data(&data).map_err(bad)?;
        } else {
            enc.set_color(png::ColorType::Rgba);
            enc.write_header().map_err(bad)?.write_image_data(&img.to_straight()).map_err(bad)?;
        }
    }
    Ok(out)
}

/// Codifica no formato pedido.
pub fn encode(img: &Raster, fmt: Format) -> Res<Vec<u8>> {
    match fmt {
        Format::Png => png(img),
    }
}
