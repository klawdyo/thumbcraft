//! pdf-min: primeira página de um PDF para PNG, só com craft-pdf (Hayro), sem motor de planilha.
//! Uso: pdf-min ENTRADA SAIDA.png [--width PX]
//! ENTRADA ou SAIDA podem ser "-" para stdin e stdout (modo de serviço, sem arquivo temporário).
//! Saída sempre RGB opaco sobre fundo branco.

use craft_pdf::raster::{self, Options, RenderCache};
use std::io::{Read, Write};
use std::process::ExitCode;

/// Falha com código de saída distinto por etapa (2 leitura, 3 abrir, 4 renderizar, 5 gravar).
fn fail(code: u8, msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("pdf-min: {msg}");
    ExitCode::from(code)
}

fn main() -> ExitCode {
    let mut pos = Vec::new();
    let mut width = 600.0f32;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        if a == "--width" {
            match it.next().and_then(|v| v.parse::<f32>().ok()) {
                Some(w) if w.is_finite() && w >= 1.0 => width = w,
                _ => return fail(1, "--width precisa de um número positivo"),
            }
        } else {
            pos.push(a);
        }
    }
    let [input, output] = match <[String; 2]>::try_from(pos) {
        Ok(v) => v,
        Err(_) => return fail(1, "uso: pdf-min ENTRADA SAIDA.png [--width PX]"),
    };

    // Lê os bytes do PDF (arquivo ou stdin).
    let mut bytes = Vec::new();
    let read = if input == "-" { std::io::stdin().lock().read_to_end(&mut bytes).map(|_| ()) } else { std::fs::read(&input).map(|b| bytes = b) };
    if let Err(e) = read {
        return fail(2, format!("leitura: {e}"));
    }

    let pdf = match craft_pdf::open_slice(&bytes, None) {
        Ok(p) => p,
        Err(e) => return fail(3, format!("abrir: {e}")),
    };
    let Some(page) = pdf.pages().first() else { return fail(3, "PDF sem páginas") };

    // Escala para que a largura final seja a pedida (a altura segue a proporção da página).
    let (pw, _) = page.render_dimensions();
    if !pw.is_finite() || pw <= 0.0 {
        return fail(4, "página inválida");
    }
    let scale = width / pw;
    let opts = Options { scale: [scale, scale], background: [255; 4], ..Default::default() };
    let ras = match raster::render(page, &RenderCache::new(), &Default::default(), &opts) {
        Ok(r) => r,
        Err(e) => return fail(4, format!("render: {e}")),
    };
    if !ras.diagnostics.is_empty() {
        eprintln!("pdf-min: aviso: {:?}", ras.diagnostics);
    }

    // Achata o RGBA pré-multiplicado sobre branco, gerando RGB.
    let (w, h) = (ras.width, ras.height);
    let mut rgb = Vec::with_capacity(w as usize * h as usize * 3);
    for px in ras.bytes().chunks_exact(4) {
        let miss = 255 - px[3] as u16;
        rgb.extend(px[..3].iter().map(|&c| (c as u16 + miss).min(255) as u8));
    }

    // Codifica o PNG para um buffer e grava de uma vez.
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        if let Err(e) = enc.write_header().and_then(|mut wr| wr.write_image_data(&rgb)) {
            return fail(5, format!("png: {e}"));
        }
    }
    let written = if output == "-" { std::io::stdout().lock().write_all(&out) } else { std::fs::write(&output, &out) };
    if let Err(e) = written {
        return fail(5, format!("gravar: {e}"));
    }
    ExitCode::SUCCESS
}
