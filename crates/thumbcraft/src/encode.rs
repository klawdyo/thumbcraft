//! Codificação da imagem final: PNG sempre; JPEG e WebP quando compilados (features).

use thumbcraft_core::{Error, Raster, Res};

/// Formato de saída.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Png,
    Jpeg,
    WebP,
}

impl Format {
    /// Formato pelo nome (`png`, `jpeg`, `jpg`, `webp`).
    pub fn parse(s: &str) -> Res<Format> {
        match s.to_ascii_lowercase().as_str() {
            "png" => Ok(Format::Png),
            "jpg" | "jpeg" => Ok(Format::Jpeg),
            "webp" => Ok(Format::WebP),
            e => Err(Error::Usage(format!("formato de saída não suportado: '{e}' (use png, jpeg ou webp)"))),
        }
    }

    /// Formato pela extensão do arquivo de saída (vazia: PNG).
    pub fn from_ext(ext: &str) -> Res<Format> {
        if ext.is_empty() { Ok(Format::Png) } else { Format::parse(ext) }
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
            enc.write_header().map_err(bad)?.write_image_data(&rgb(img)).map_err(bad)?;
        } else {
            enc.set_color(png::ColorType::Rgba);
            enc.write_header().map_err(bad)?.write_image_data(&img.to_straight()).map_err(bad)?;
        }
    }
    Ok(out)
}

/// Só os canais RGB (a imagem precisa ser opaca).
fn rgb(img: &Raster) -> Vec<u8> {
    let mut data = Vec::with_capacity(img.w as usize * img.h as usize * 3);
    for p in img.px.chunks_exact(4) {
        data.extend_from_slice(&p[..3]);
    }
    data
}

#[cfg(feature = "jpeg")]
fn jpeg(img: &Raster, quality: u8) -> Res<Vec<u8>> {
    use image::{ExtendedColorType, ImageEncoder, codecs::jpeg::JpegEncoder};
    // JPEG não tem alfa: o que for transparente vira branco.
    let mut flat = img.clone();
    thumbcraft_core::post::flatten(&mut flat, [255, 255, 255]);
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, quality.clamp(1, 100))
        .write_image(&rgb(&flat), img.w, img.h, ExtendedColorType::Rgb8)
        .map_err(|e| Error::Internal(format!("falha ao codificar o JPEG: {e}")))?;
    Ok(out)
}

#[cfg(not(feature = "jpeg"))]
fn jpeg(_: &Raster, _: u8) -> Res<Vec<u8>> {
    Err(Error::Usage("este binário foi compilado sem JPEG".into()))
}

#[cfg(feature = "webp")]
fn webp(img: &Raster) -> Res<Vec<u8>> {
    use image::{ExtendedColorType, ImageEncoder, codecs::webp::WebPEncoder};
    let mut out = Vec::new();
    let enc = WebPEncoder::new_lossless(&mut out);
    let r = if img.opaque() {
        enc.write_image(&rgb(img), img.w, img.h, ExtendedColorType::Rgb8)
    } else {
        enc.write_image(&img.to_straight(), img.w, img.h, ExtendedColorType::Rgba8)
    };
    r.map_err(|e| Error::Internal(format!("falha ao codificar o WebP: {e}")))?;
    Ok(out)
}

#[cfg(not(feature = "webp"))]
fn webp(_: &Raster) -> Res<Vec<u8>> {
    Err(Error::Usage("este binário foi compilado sem WebP".into()))
}

/// Codifica no formato pedido (`quality` só vale para JPEG; o WebP é sem perdas).
pub fn encode(img: &Raster, fmt: Format, quality: u8) -> Res<Vec<u8>> {
    match fmt {
        Format::Png => png(img),
        Format::Jpeg => jpeg(img, quality),
        Format::WebP => webp(img),
    }
}
