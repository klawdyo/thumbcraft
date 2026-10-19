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

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(bytes: &[u8]) -> (png::OutputInfo, Vec<u8>) {
        let mut r = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().unwrap();
        let mut buf = vec![0; r.output_buffer_size().unwrap()];
        let info = r.next_frame(&mut buf).unwrap();
        buf.truncate(info.buffer_size());
        (info, buf)
    }

    #[test]
    fn format_by_name_and_extension() {
        assert_eq!(Format::parse("PNG").unwrap(), Format::Png);
        assert_eq!(Format::parse("jpg").unwrap(), Format::Jpeg);
        assert_eq!(Format::parse("jpeg").unwrap(), Format::Jpeg);
        assert_eq!(Format::parse("WebP").unwrap(), Format::WebP);
        assert_eq!(Format::parse("bmp").unwrap_err().code(), 2);
        assert_eq!(Format::from_ext("").unwrap(), Format::Png);
        assert_eq!(Format::from_ext("gif").unwrap_err().code(), 2);
    }

    #[test]
    fn opaque_png_becomes_rgb_and_round_trips() {
        let mut img = Raster::filled(3, 2, [10, 20, 30, 255]);
        img.set(2, 1, [200, 100, 50, 255]);
        let (info, data) = decode(&png(&img).unwrap());
        assert_eq!((info.width, info.height, info.color_type), (3, 2, png::ColorType::Rgb));
        assert_eq!(&data[..3], &[10, 20, 30]);
        assert_eq!(&data[15..18], &[200, 100, 50]);
    }

    #[test]
    fn png_with_alpha_uses_straight_alpha() {
        // Pré-multiplicado (64, 32, 16, 128) corresponde a ~(127, 64, 32) com alfa 128.
        let img = Raster::filled(2, 2, [64, 32, 16, 128]);
        let (info, data) = decode(&png(&img).unwrap());
        assert_eq!(info.color_type, png::ColorType::Rgba);
        let (r, g, b, a) = (data[0] as i32, data[1] as i32, data[2] as i32, data[3]);
        assert_eq!(a, 128);
        assert!((r - 127).abs() <= 1 && (g - 64).abs() <= 1 && (b - 32).abs() <= 1, "{r} {g} {b}");
    }

    #[cfg(feature = "jpeg")]
    #[test]
    fn jpeg_has_signature_and_quality_matters() {
        let mut img = Raster::filled(64, 64, [255, 255, 255, 255]);
        for y in 0..64 {
            for x in 0..64 {
                img.set(x, y, [(x * 4) as u8, (y * 4) as u8, ((x * y) % 256) as u8, 255]);
            }
        }
        let low = encode(&img, Format::Jpeg, 10).unwrap();
        let high = encode(&img, Format::Jpeg, 95).unwrap();
        assert_eq!(&low[..3], &[0xFF, 0xD8, 0xFF]);
        assert!(high.len() > low.len());
    }

    #[cfg(feature = "jpeg")]
    #[test]
    fn transparent_jpeg_becomes_white_without_failing() {
        let img = Raster::filled(8, 8, [0, 0, 0, 0]);
        assert!(encode(&img, Format::Jpeg, 85).unwrap().starts_with(&[0xFF, 0xD8]));
    }

    #[cfg(feature = "webp")]
    #[test]
    fn webp_has_riff_signature() {
        for img in [Raster::filled(5, 5, [1, 2, 3, 255]), Raster::filled(5, 5, [1, 2, 3, 100])] {
            let b = encode(&img, Format::WebP, 85).unwrap();
            assert_eq!(&b[..4], b"RIFF");
            assert_eq!(&b[8..12], b"WEBP");
        }
    }
}
