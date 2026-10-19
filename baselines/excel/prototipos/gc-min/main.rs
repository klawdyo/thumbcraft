//! `gc-min IN.xlsx OUT.png [--range A1:Z60] [--scale S] [--fit] [--crop]`: thumbnail of a workbook.
//!
//! Thumbnail policy lives here, as parameters and page-setup commands handed to the engine's
//! `file.exportPng`, not as defaults of that command: first visible sheet, A4, the sheet's own
//! orientation, margins, print area, scale and breaks, never fitted to the page unless `--fit`,
//! page 1, whole page on white unless `--crop` (bounding box of the non-white pixels plus an 8 px white margin).

use gridcraft_engine::Session;
use serde_json::json;

fn run(args: &[String]) -> Result<(), String> {
    let (Some(input), Some(out)) = (args.first(), args.get(1)) else {
        return Err("usage: gc-min IN.xlsx OUT.png [--range A1:Z60] [--scale S] [--fit] [--crop]".into());
    };
    let mut p = json!({"path": out, "scale": 2.0, "page": 1});
    let mut crop = false;
    let mut rest = args.iter().skip(2);
    while let Some(a) = rest.next() {
        match a.as_str() {
            "--range" => p["range"] = json!(rest.next().ok_or("--range needs a value")?),
            "--scale" => p["scale"] = json!(rest.next().and_then(|v| v.parse::<f64>().ok()).ok_or("--scale needs a number")?),
            "--fit" => p["fitToPage"] = json!(true),
            "--crop" => crop = true,
            other => return Err(format!("unknown option {other}")),
        }
    }
    let mut s = Session::new();
    let run = |s: &mut Session, id: &str, p| s.execute_headless(id, p).map_err(|e| e.to_string());
    run(&mut s, "file.open", json!({"path": input}))?;
    let first_visible = {
        let d = s.doc().map_err(|e| e.to_string())?;
        (0..d.wb.sheets.len()).find(|i| d.wb.sheet(*i).is_some_and(|sh| format!("{:?}", sh.visibility) == "Visible")).unwrap_or(0)
    };
    run(&mut s, "sheet.activate", json!({"sheet": first_visible}))?;
    run(&mut s, "pageLayout.pageSetup", json!({"paper": "A4", "fitToPage": p.get("fitToPage").is_some()}))?;
    run(&mut s, "file.exportPng", p)?;
    if crop {
        crop_file(out)?;
    }
    Ok(())
}

const PAD: usize = 8;

/// Cuts the PNG at `path` to the bounding box of its non-white pixels plus `PAD` white pixels on
/// every side. An all-white page is left as it is.
fn crop_file(path: &str) -> Result<(), String> {
    let e = |e: &dyn std::fmt::Display| format!("{path}: {e}");
    let file = std::fs::File::open(path).map_err(|x| e(&x))?;
    let mut rd = png::Decoder::new(std::io::BufReader::new(file)).read_info().map_err(|x| e(&x))?;
    let mut buf = vec![0u8; rd.output_buffer_size().ok_or("image too large")?];
    let info = rd.next_frame(&mut buf).map_err(|x| e(&x))?;
    let ch = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => 4,
        (png::ColorType::Rgb, png::BitDepth::Eight) => 3,
        other => return Err(format!("unexpected PNG format {other:?}")),
    };
    let (w, h) = (info.width as usize, info.height as usize);
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * ch;
            if buf.get(i..i + 3).is_some_and(|p| p != [255, 255, 255]) {
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1));
            }
        }
    }
    if x1 <= x0 || y1 <= y0 {
        return Ok(());
    }
    let (ow, oh) = (x1 - x0 + 2 * PAD, y1 - y0 + 2 * PAD);
    let mut out = vec![255u8; ow * oh * ch];
    for y in y0..y1 {
        let (s, d) = ((y * w + x0) * ch, ((y - y0 + PAD) * ow + PAD) * ch);
        let n = (x1 - x0) * ch;
        if let (Some(src), Some(dst)) = (buf.get(s..s + n), out.get_mut(d..d + n)) {
            dst.copy_from_slice(src);
        }
    }
    let file = std::fs::File::create(path).map_err(|x| e(&x))?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), ow as u32, oh as u32);
    enc.set_color(info.color_type);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().and_then(|mut w| w.write_image_data(&out)).map_err(|x| e(&x))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(&args) {
        eprintln!("gc-min: {e}");
        std::process::exit(1);
    }
}
