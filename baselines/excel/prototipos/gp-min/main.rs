//! gp-min: xlsx -> PNG thumbnail without touching GridCraft.
//! Pipeline: GridCraft engine (`file.open`, `pageLayout.pageSetup`, `file.exportPdf`) -> PDF bytes
//! in memory -> `craft-pdf` raster (Hayro) of page 0 -> flatten on white -> crop -> PNG.

use craft_pdf::raster::{self, Options, RenderCache};
use gridcraft_engine::core::RangeRef;
use gridcraft_engine::model::Visibility;
use gridcraft_engine::{Session, io::base64_decode};
use serde_json::json;
use std::process::ExitCode;

const PAD: u32 = 8;

struct Args {
    input: String,
    output: String,
    range: Option<String>,
    scale: f32,
    gridlines: bool,
    crop: bool,
    fit: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut pos = Vec::new();
    let (mut range, mut scale, mut gridlines, mut crop, mut fit) = (None, 2.0f32, false, false, false);
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--range" => range = Some(it.next().ok_or("--range needs a value")?),
            "--scale" => {
                scale = it.next().ok_or("--scale needs a value")?.parse().map_err(|_| "--scale must be a number")?;
                if !(scale.is_finite() && scale > 0.0) {
                    return Err("--scale must be positive".into());
                }
            }
            "--gridlines" => gridlines = true,
            "--crop" => crop = true,
            "--fit" => fit = true,
            s if s.starts_with("--") => return Err(format!("unknown option {s}")),
            _ => pos.push(a),
        }
    }
    let [input, output] = <[String; 2]>::try_from(pos).map_err(|_| "expected IN.xlsx OUT.png".to_string())?;
    Ok(Args { input, output, range, scale, gridlines, crop, fit })
}

fn fail(code: u8, msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("gp-min: {msg}");
    ExitCode::from(code)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("gp-min: {e}\nusage: gp-min IN.xlsx OUT.png [--range A1:F40] [--scale S] [--crop] [--fit] [--gridlines]");
            return ExitCode::from(1);
        }
    };
    let mut s = Session::new();
    if let Err(e) = s.execute("file.open", json!({"path": args.input})) {
        return fail(2, format!("open: {e}"));
    }

    // First visible sheet (like LibreOffice's first printed page), then follow the sheet's own
    // Page Setup (orientation, margins, scale, print area, headers, titles); only the paper is forced to A4.
    let first = {
        let Ok(d) = s.doc() else { return fail(2, "no document") };
        d.wb.sheets.iter().position(|sh| sh.visibility == Visibility::Visible).unwrap_or(0)
    };
    if let Err(e) = s.execute("sheet.activate", json!({"sheet": first})) {
        return fail(3, format!("activate: {e}"));
    }
    let mut setup = json!({"paper": "A4"});
    if args.gridlines {
        setup["gridlines"] = json!(true);
    }
    if args.fit {
        setup["fitWidth"] = json!(1);
        setup["fitHeight"] = json!(1);
    }
    if let Err(e) = s.execute("pageLayout.pageSetup", setup) {
        return fail(3, format!("pageSetup: {e}"));
    }
    // fitToPage=false overrides any fit-to-pages stored in the sheet: no zoom, the sheet's own % scale applies.
    let mut p = json!({"fitToPage": args.fit, "sheets": "active"});
    let range_txt = match &args.range {
        Some(r) => {
            if RangeRef::parse(r.rsplit('!').next().unwrap_or(r)).is_none() {
                return fail(1, "invalid --range");
            }
            p["range"] = json!(r);
            r.clone()
        }
        None => "print area".to_string(),
    };
    let out = match s.execute("file.exportPdf", p) {
        Ok(o) => o,
        Err(e) => return fail(3, format!("exportPdf: {e}")),
    };
    let npages = out.get("pages").and_then(|n| n.as_u64()).unwrap_or(0);
    let Some(pdf_bytes) = out.get("base64").and_then(|b| b.as_str()).and_then(base64_decode) else {
        return fail(3, "exportPdf returned no data");
    };
    drop(s);

    let pdf = match craft_pdf::open_slice(&pdf_bytes, None) {
        Ok(p) => p,
        Err(e) => return fail(4, format!("pdf open: {e}")),
    };
    let Some(page) = pdf.pages().first() else { return fail(4, "pdf has no pages") };
    let opts = Options { scale: [args.scale; 2], background: [255; 4], ..Default::default() };
    let ras = match raster::render(page, &RenderCache::new(), &Default::default(), &opts) {
        Ok(r) => r,
        Err(e) => return fail(4, format!("raster: {e}")),
    };
    if !ras.diagnostics.is_empty() {
        eprintln!("gp-min: warning: raster diagnostics {:?}", ras.diagnostics);
    }

    // Flatten premultiplied RGBA over white into RGB, then crop to the ink plus padding.
    let (w, h) = (ras.width as usize, ras.height as usize);
    let src = ras.bytes();
    let mut rgb = Vec::with_capacity(w * h * 3);
    for px in src.chunks_exact(4) {
        let miss = 255 - px[3] as u16;
        rgb.extend(px[..3].iter().map(|&c| (c as u16 + miss).min(255) as u8));
    }
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0usize, 0usize);
    if args.crop {
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) * 3;
                if rgb[i] < 250 || rgb[i + 1] < 250 || rgb[i + 2] < 250 {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x + 1);
                    y1 = y1.max(y + 1);
                }
            }
        }
    }
    if !args.crop || x1 <= x0 {
        (x0, y0, x1, y1) = (0, 0, w, h);
    } else {
        let p = PAD as usize;
        (x0, y0, x1, y1) = (x0.saturating_sub(p), y0.saturating_sub(p), (x1 + p).min(w), (y1 + p).min(h));
    }
    let (cw, ch) = (x1 - x0, y1 - y0);
    let mut cropped = Vec::with_capacity(cw * ch * 3);
    for y in y0..y1 {
        cropped.extend_from_slice(&rgb[(y * w + x0) * 3..(y * w + x1) * 3]);
    }

    let file = match std::fs::File::create(&args.output) {
        Ok(f) => std::io::BufWriter::new(f),
        Err(e) => return fail(5, format!("create {}: {e}", args.output)),
    };
    let mut enc = png::Encoder::new(file, cw as u32, ch as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let res = enc.write_header().and_then(|mut w| w.write_image_data(&cropped));
    if let Err(e) = res {
        return fail(5, format!("png: {e}"));
    }
    eprintln!("gp-min: {} ({} pages in PDF, range {}, {}x{} px)", args.output, npages, range_txt, cw, ch);
    ExitCode::SUCCESS
}
