//! Opções do pós-processamento e seus analisadores de texto.

use crate::error::{Error, Res};
use crate::raster::Region;

/// Maior lado (px) aceito para a imagem final.
pub const MAX_OUT_SIDE: u32 = 16_384;
/// Maior número de pixels da imagem final.
pub const MAX_OUT_PIXELS: u64 = 64_000_000;

/// Como a renderização ocupa a caixa quando largura e altura são dadas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    /// Preenche a caixa e corta o excesso.
    Cover,
    /// Cabe inteira e completa o resto com o fundo.
    Contain,
}

impl Fit {
    pub fn parse(s: &str) -> Res<Fit> {
        match s {
            "cover" => Ok(Fit::Cover),
            "contain" => Ok(Fit::Contain),
            _ => Err(Error::Usage(format!("valor inválido para --fit: '{s}' (use cover ou contain)"))),
        }
    }
}

/// Âncora, em frações: 0 = início, 0,5 = centro, 1 = fim, em cada eixo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gravity {
    pub x: f64,
    pub y: f64,
}

impl Gravity {
    pub const TOP: Gravity = Gravity { x: 0.5, y: 0.0 };
    pub const CENTER: Gravity = Gravity { x: 0.5, y: 0.5 };

    pub fn parse(s: &str) -> Res<Gravity> {
        let g = |x, y| Ok(Gravity { x, y });
        match s {
            "top" => g(0.5, 0.0),
            "bottom" => g(0.5, 1.0),
            "left" => g(0.0, 0.5),
            "right" => g(1.0, 0.5),
            "center" => g(0.5, 0.5),
            "top-left" => g(0.0, 0.0),
            "top-right" => g(1.0, 0.0),
            "bottom-left" => g(0.0, 1.0),
            "bottom-right" => g(1.0, 1.0),
            _ => Err(Error::Usage(format!(
                "valor inválido para --gravity: '{s}' (use top, bottom, left, right, center, top-left, top-right, bottom-left ou bottom-right)"
            ))),
        }
    }
}

/// Fundo usado no contain e sob transparência.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Background {
    /// Cor opaca RGB.
    Solid([u8; 3]),
    /// Mantém o alfa (PNG e WebP; o JPEG não tem alfa e cai em branco).
    Transparent,
}

impl Background {
    pub const WHITE: Background = Background::Solid([255, 255, 255]);

    pub fn parse(s: &str) -> Res<Background> {
        if s.eq_ignore_ascii_case("transparent") {
            return Ok(Background::Transparent);
        }
        if s.eq_ignore_ascii_case("white") {
            return Ok(Background::WHITE);
        }
        let bad = || Error::Usage(format!("valor inválido para --background: '{s}' (use transparent, white ou #RRGGBB)"));
        let hex = s.strip_prefix('#').ok_or_else(bad)?;
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(bad());
        }
        let v = u32::from_str_radix(hex, 16).map_err(|_| bad())?;
        Ok(Background::Solid([(v >> 16) as u8, (v >> 8) as u8, v as u8]))
    }
}

/// Retângulo em FRAÇÕES (0 a 1) da página renderizada: x, y, largura, altura.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crop {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Crop {
    /// Analisa "x,y,l,a".
    pub fn parse(s: &str) -> Res<Crop> {
        let bad = |why: &str| Error::Usage(format!("valor inválido para --crop: '{s}' ({why}); use x,y,l,a em frações de 0 a 1"));
        let v: Vec<&str> = s.split(',').map(str::trim).collect();
        if v.len() != 4 {
            return Err(bad("esperava 4 números"));
        }
        let mut n = [0f64; 4];
        for (i, t) in v.iter().enumerate() {
            n[i] = t.parse::<f64>().map_err(|_| bad("número ilegível"))?;
            if !n[i].is_finite() {
                return Err(bad("número não finito"));
            }
        }
        let [x, y, w, h] = n;
        const EPS: f64 = 1e-9;
        if x < 0.0 || y < 0.0 || x > 1.0 || y > 1.0 {
            return Err(bad("x e y devem estar entre 0 e 1"));
        }
        if w <= 0.0 || h <= 0.0 {
            return Err(bad("largura e altura devem ser maiores que 0"));
        }
        if x + w > 1.0 + EPS || y + h > 1.0 + EPS {
            return Err(bad("o retângulo passa do limite da página"));
        }
        Ok(Crop { x, y, w, h })
    }

    /// Converte para pixels de uma imagem `iw` x `ih`, com ao menos 1 px de cada lado.
    pub fn to_region(&self, iw: u32, ih: u32) -> Region {
        let x0 = (self.x * iw as f64).round().clamp(0.0, iw.saturating_sub(1) as f64) as u32;
        let y0 = (self.y * ih as f64).round().clamp(0.0, ih.saturating_sub(1) as f64) as u32;
        let x1 = ((self.x + self.w) * iw as f64).round().clamp((x0 + 1) as f64, iw as f64) as u32;
        let y1 = ((self.y + self.h) * ih as f64).round().clamp((y0 + 1) as f64, ih as f64) as u32;
        Region { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
    }
}

/// Filtro de reamostragem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    /// Média de área (caixa com cobertura fracionária): suave, sem anéis.
    Area,
    /// Lanczos de 3 lóbulos: mais nítido.
    Lanczos3,
}

impl Filter {
    pub fn parse(s: &str) -> Res<Filter> {
        match s {
            "area" => Ok(Filter::Area),
            "lanczos" | "lanczos3" => Ok(Filter::Lanczos3),
            _ => Err(Error::Usage(format!("valor inválido para --filter: '{s}' (use area ou lanczos)"))),
        }
    }
}

/// Escolha da escala de renderização (px/pt).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderScale {
    /// Escolhida pelo tamanho do alvo (supersampling de 2x).
    Auto,
    /// Valor fixo em px/pt.
    Fixed(f64),
}

impl RenderScale {
    pub fn parse(s: &str) -> Res<RenderScale> {
        if s == "auto" {
            return Ok(RenderScale::Auto);
        }
        match s.parse::<f64>() {
            Ok(v) if v.is_finite() && v > 0.0 => Ok(RenderScale::Fixed(v)),
            _ => Err(Error::Usage(format!("valor inválido para --render-scale: '{s}' (use auto ou um número maior que 0, em px/pt)"))),
        }
    }
}

/// Todas as opções do pós-processamento.
#[derive(Clone, Debug, PartialEq)]
pub struct Opts {
    /// Largura final em px.
    pub width: Option<u32>,
    /// Altura final em px.
    pub height: Option<u32>,
    pub fit: Fit,
    /// `None` = padrão do fit (top para cover, center para contain).
    pub gravity: Option<Gravity>,
    pub background: Background,
    pub trim: bool,
    /// Margem (px da renderização nativa) mantida em volta do conteúdo no trim.
    pub trim_pad: u32,
    pub crop: Option<Crop>,
    pub filter: Filter,
    pub scale: RenderScale,
    /// Página (slide) a partir de 1.
    pub page: u32,
}

impl Default for Opts {
    fn default() -> Opts {
        Opts {
            width: None,
            height: None,
            fit: Fit::Cover,
            gravity: None,
            background: Background::WHITE,
            trim: false,
            trim_pad: 8,
            crop: None,
            filter: Filter::Lanczos3,
            scale: RenderScale::Auto,
            page: 1,
        }
    }
}

impl Opts {
    /// Âncora efetiva (padrão por fit).
    pub fn gravity(&self) -> Gravity {
        self.gravity.unwrap_or(match self.fit {
            Fit::Cover => Gravity::TOP,
            Fit::Contain => Gravity::CENTER,
        })
    }

    /// Valida combinações e limites.
    pub fn check(&self) -> Res<()> {
        for (label, v) in [("largura", self.width), ("altura", self.height)] {
            if let Some(v) = v {
                if v == 0 || v > MAX_OUT_SIDE {
                    return Err(Error::Usage(format!("{label} inválida: {v} (use de 1 a {MAX_OUT_SIDE} px)")));
                }
            }
        }
        if let (Some(w), Some(h)) = (self.width, self.height) {
            if w as u64 * h as u64 > MAX_OUT_PIXELS {
                return Err(Error::Usage(format!("{w}x{h} passa do limite de {MAX_OUT_PIXELS} pixels")));
            }
        }
        if self.page == 0 {
            return Err(Error::Usage("--page começa em 1".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gravity_names() {
        assert_eq!(Gravity::parse("top").unwrap(), Gravity { x: 0.5, y: 0.0 });
        assert_eq!(Gravity::parse("bottom-right").unwrap(), Gravity { x: 1.0, y: 1.0 });
        assert_eq!(Gravity::parse("left").unwrap(), Gravity { x: 0.0, y: 0.5 });
        assert!(Gravity::parse("norte").is_err());
    }

    #[test]
    fn background_forms() {
        assert_eq!(Background::parse("transparent").unwrap(), Background::Transparent);
        assert_eq!(Background::parse("#FF8000").unwrap(), Background::Solid([255, 128, 0]));
        assert_eq!(Background::parse("#ff8000").unwrap(), Background::Solid([255, 128, 0]));
        assert!(Background::parse("#fff").is_err());
        assert!(Background::parse("vermelho").is_err());
        assert!(Background::parse("#GG0000").is_err());
    }

    #[test]
    fn crop_validation() {
        assert!(Crop::parse("0,0,1,1").is_ok());
        assert!(Crop::parse("0.1, 0.2, 0.5, 0.5").is_ok());
        assert!(Crop::parse("0,0,1").is_err());
        assert!(Crop::parse("0,0,0,1").is_err());
        assert!(Crop::parse("0.5,0,0.6,1").is_err());
        assert!(Crop::parse("-0.1,0,1,1").is_err());
        assert!(Crop::parse("a,b,c,d").is_err());
        assert!(Crop::parse("nan,0,1,1").is_err());
    }

    #[test]
    fn crop_to_pixels() {
        let c = Crop::parse("0.25,0.5,0.5,0.5").unwrap();
        assert_eq!(c.to_region(200, 100), Region { x: 50, y: 50, w: 100, h: 50 });
        // Fração minúscula ainda dá 1 pixel.
        let c = Crop::parse("0.999,0.999,0.001,0.001").unwrap();
        let r = c.to_region(100, 100);
        assert!(r.w >= 1 && r.h >= 1 && r.x + r.w <= 100 && r.y + r.h <= 100);
    }

    #[test]
    fn render_scale_parse() {
        assert_eq!(RenderScale::parse("auto").unwrap(), RenderScale::Auto);
        assert_eq!(RenderScale::parse("2.5").unwrap(), RenderScale::Fixed(2.5));
        assert!(RenderScale::parse("0").is_err());
        assert!(RenderScale::parse("-1").is_err());
        assert!(RenderScale::parse("inf").is_err());
    }

    #[test]
    fn opts_limits() {
        let mut o = Opts::default();
        assert!(o.check().is_ok());
        o.width = Some(0);
        assert!(o.check().is_err());
        o.width = Some(MAX_OUT_SIDE + 1);
        assert!(o.check().is_err());
        o.width = Some(16_000);
        o.height = Some(16_000);
        assert!(o.check().is_err());
        o.height = Some(100);
        o.page = 0;
        assert!(o.check().is_err());
    }

    #[test]
    fn default_gravity_per_fit() {
        let mut o = Opts::default();
        assert_eq!(o.gravity(), Gravity::TOP);
        o.fit = Fit::Contain;
        assert_eq!(o.gravity(), Gravity::CENTER);
        o.gravity = Some(Gravity { x: 1.0, y: 1.0 });
        assert_eq!(o.gravity().x, 1.0);
    }
}
