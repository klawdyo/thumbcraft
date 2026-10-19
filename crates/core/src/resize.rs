//! Reamostragem separável de alta qualidade (média de área e Lanczos3), sem dependências.
//!
//! Trabalha em RGBA pré-multiplicado de 8 bits com acumuladores em `f32`. A origem é uma janela
//! fracionária dentro de uma região inteira da imagem; amostras fora da região nunca entram.

use crate::fit::RectF;
use crate::opts::Filter;
use crate::raster::{Raster, Region};

/// Pesos de um eixo: para cada pixel de destino, o primeiro pixel de origem e os pesos.
struct Weights {
    start: Vec<u32>,
    off: Vec<usize>,
    w: Vec<f32>,
}

fn lanczos3(x: f64) -> f64 {
    let x = x.abs();
    if x < 1e-12 {
        return 1.0;
    }
    if x >= 3.0 {
        return 0.0;
    }
    let px = std::f64::consts::PI * x;
    3.0 * px.sin() * (px / 3.0).sin() / (px * px)
}

fn triangle(x: f64) -> f64 {
    (1.0 - x.abs()).max(0.0)
}

/// Pesos para mapear `[o0, o0 + olen)` (coordenadas de origem, em pixels) em `n` pixels, com a
/// origem limitada a `[lo, hi)`.
fn weights(o0: f64, olen: f64, n: u32, lo: u32, hi: u32, filter: Filter) -> Weights {
    let step = olen / n as f64;
    let down = step > 1.0;
    let mut out = Weights { start: Vec::with_capacity(n as usize), off: Vec::with_capacity(n as usize + 1), w: Vec::new() };
    for i in 0..n {
        let a = o0 + i as f64 * step;
        let b = a + step;
        let c = (a + b) / 2.0;
        let (first, taps): (i64, Vec<f64>);
        match (filter, down) {
            // Média de área exata na redução: peso = comprimento da interseção.
            (Filter::Area, true) => {
                let f = (a.floor() as i64).max(lo as i64);
                let l = ((b.ceil() as i64) - 1).min(hi as i64 - 1).max(f);
                first = f;
                taps = (f..=l).map(|j| ((j as f64 + 1.0).min(b) - (j as f64).max(a)).max(0.0)).collect();
            }
            _ => {
                // Triângulo (bilinear) para ampliar ou para Area ampliando; Lanczos3 em ambos.
                let (support, scale) = match (filter, down) {
                    (Filter::Lanczos3, true) => (3.0 * step, step),
                    (Filter::Lanczos3, false) => (3.0, 1.0),
                    (_, _) => (1.0, 1.0),
                };
                let f = ((c - support).floor() as i64).max(lo as i64);
                let l = ((c + support).ceil() as i64).min(hi as i64 - 1).max(f);
                first = f;
                taps = (f..=l)
                    .map(|j| {
                        let d = ((j as f64 + 0.5) - c) / scale;
                        match filter {
                            Filter::Lanczos3 => lanczos3(d),
                            Filter::Area => triangle(d),
                        }
                    })
                    .collect();
            }
        }
        let sum: f64 = taps.iter().sum();
        let mut taps = taps;
        if sum.abs() < 1e-9 {
            // Janela sem peso (não deveria ocorrer): pega o pixel mais próximo.
            let j = (c.floor() as i64).clamp(lo as i64, hi as i64 - 1);
            out.start.push(j as u32);
            out.off.push(out.w.len());
            out.w.push(1.0);
            continue;
        }
        for t in &mut taps {
            *t /= sum;
        }
        out.start.push(first as u32);
        out.off.push(out.w.len());
        out.w.extend(taps.iter().map(|t| *t as f32));
    }
    out.off.push(out.w.len());
    out
}

fn to_u8(v: f32) -> u8 {
    (v + 0.5).clamp(0.0, 255.0) as u8
}

/// Reamostra a janela `win` (coordenadas absolutas de `src`, dentro de `clip`) para `dw` x `dh`.
pub fn resample(src: &Raster, clip: Region, win: RectF, dw: u32, dh: u32, filter: Filter) -> Raster {
    let wx = weights(win.x, win.w, dw, clip.x, clip.x + clip.w, filter);
    let wy = weights(win.y, win.h, dh, clip.y, clip.y + clip.h, filter);
    // Linhas de origem realmente usadas.
    let row_lo = wy.start.iter().copied().min().unwrap_or(clip.y);
    let row_hi = (0..dh as usize).map(|j| wy.start[j] as usize + (wy.off[j + 1] - wy.off[j])).max().unwrap_or(row_lo as usize + 1);
    let nrows = row_hi - row_lo as usize;
    let sw = src.w as usize;
    // Passo horizontal: nrows x dw, 8 bits.
    let mut mid = vec![0u8; nrows * dw as usize * 4];
    for r in 0..nrows {
        let srow = &src.px[(row_lo as usize + r) * sw * 4..(row_lo as usize + r + 1) * sw * 4];
        let mrow = &mut mid[r * dw as usize * 4..(r + 1) * dw as usize * 4];
        for i in 0..dw as usize {
            let (s, ws) = (wx.start[i] as usize, &wx.w[wx.off[i]..wx.off[i + 1]]);
            let mut acc = [0f32; 4];
            for (k, w) in ws.iter().enumerate() {
                let p = &srow[(s + k) * 4..(s + k) * 4 + 4];
                acc[0] += p[0] as f32 * w;
                acc[1] += p[1] as f32 * w;
                acc[2] += p[2] as f32 * w;
                acc[3] += p[3] as f32 * w;
            }
            for c in 0..4 {
                mrow[i * 4 + c] = to_u8(acc[c]);
            }
        }
    }
    // Passo vertical.
    let mut out = vec![0u8; dw as usize * dh as usize * 4];
    let mut acc = vec![0f32; dw as usize * 4];
    for j in 0..dh as usize {
        acc.iter_mut().for_each(|v| *v = 0.0);
        let (s, ws) = (wy.start[j] as usize - row_lo as usize, &wy.w[wy.off[j]..wy.off[j + 1]]);
        for (k, w) in ws.iter().enumerate() {
            let mrow = &mid[(s + k) * dw as usize * 4..(s + k + 1) * dw as usize * 4];
            for (a, m) in acc.iter_mut().zip(mrow) {
                *a += *m as f32 * w;
            }
        }
        let orow = &mut out[j * dw as usize * 4..(j + 1) * dw as usize * 4];
        for (o, a) in orow.chunks_exact_mut(4).zip(acc.chunks_exact(4)) {
            let al = to_u8(a[3]);
            o[3] = al;
            // Pré-multiplicado: a cor nunca passa do alfa (o Lanczos pode estourar).
            for c in 0..3 {
                o[c] = to_u8(a[c]).min(al);
            }
        }
    }
    Raster { w: dw, h: dh, px: out }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, c: [u8; 4]) -> Raster {
        Raster::filled(w, h, c)
    }

    fn window(r: &Raster) -> RectF {
        RectF { x: 0.0, y: 0.0, w: r.w as f64, h: r.h as f64 }
    }

    #[test]
    fn solid_color_stays_solid() {
        for f in [Filter::Area, Filter::Lanczos3] {
            let s = solid(100, 60, [10, 120, 250, 255]);
            let d = resample(&s, s.full(), window(&s), 17, 9, f);
            assert!(d.px.chunks_exact(4).all(|p| p == [10, 120, 250, 255]), "{f:?}");
            let d = resample(&s, s.full(), window(&s), 300, 200, f);
            assert!(d.px.chunks_exact(4).all(|p| p == [10, 120, 250, 255]), "{f:?} ampliando");
        }
    }

    #[test]
    fn exact_area_average_on_2x_reduction() {
        // Linhas alternadas preto/branco: a média é cinza médio.
        let mut s = solid(8, 8, [255, 255, 255, 255]);
        for y in (0..8).step_by(2) {
            for x in 0..8 {
                s.set(x, y, [0, 0, 0, 255]);
            }
        }
        let d = resample(&s, s.full(), window(&s), 4, 4, Filter::Area);
        assert!(d.px.chunks_exact(4).all(|p| (p[0] as i32 - 128).abs() <= 1 && p[3] == 255), "{:?}", &d.px[..8]);
    }

    #[test]
    fn identity_does_not_change() {
        let mut s = solid(6, 6, [255, 255, 255, 255]);
        s.set(2, 3, [0, 0, 0, 255]);
        let d = resample(&s, s.full(), window(&s), 6, 6, Filter::Area);
        assert_eq!(d, s);
        let d = resample(&s, s.full(), window(&s), 6, 6, Filter::Lanczos3);
        assert_eq!(d, s);
    }

    #[test]
    fn fractional_window_respects_region() {
        // Esquerda vermelha, direita azul; a janela só da metade direita não pode ter vermelho.
        let mut s = solid(100, 10, [0, 0, 255, 255]);
        for y in 0..10 {
            for x in 0..50 {
                s.set(x, y, [255, 0, 0, 255]);
            }
        }
        let clip = Region { x: 50, y: 0, w: 50, h: 10 };
        for f in [Filter::Area, Filter::Lanczos3] {
            let d = resample(&s, clip, RectF { x: 50.0, y: 0.0, w: 50.0, h: 10.0 }, 5, 1, f);
            assert!(d.px.chunks_exact(4).all(|p| p[0] == 0 && p[2] == 255), "{f:?}");
        }
    }

    #[test]
    fn premultiplied_alpha_stays_consistent() {
        // Borda dura entre opaco e transparente: a cor nunca passa do alfa.
        let mut s = solid(40, 40, [0, 0, 0, 0]);
        for y in 0..40 {
            for x in 0..20 {
                s.set(x, y, [200, 100, 50, 255]);
            }
        }
        for f in [Filter::Area, Filter::Lanczos3] {
            let d = resample(&s, s.full(), window(&s), 7, 7, f);
            assert!(d.px.chunks_exact(4).all(|p| p[0] <= p[3] && p[1] <= p[3] && p[2] <= p[3]), "{f:?}");
        }
    }

    #[test]
    fn extreme_sizes_do_not_break() {
        let s = solid(3, 2, [1, 2, 3, 255]);
        let d = resample(&s, s.full(), window(&s), 1, 1, Filter::Lanczos3);
        assert_eq!((d.w, d.h), (1, 1));
        let d = resample(&s, s.full(), window(&s), 50, 1, Filter::Area);
        assert_eq!((d.w, d.h), (50, 1));
        let single_px = solid(1, 1, [9, 9, 9, 255]);
        let d = resample(&single_px, single_px.full(), window(&single_px), 4, 4, Filter::Lanczos3);
        assert!(d.px.chunks_exact(4).all(|p| p == [9, 9, 9, 255]));
    }
}
