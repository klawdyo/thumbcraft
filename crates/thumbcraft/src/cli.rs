//! Linha de comando: análise dos argumentos.

use thumbcraft_core::opts::{Background, Crop, Filter, Fit, Gravity, RenderScale};
use thumbcraft_core::{Error, Opts, Res};

use crate::encode::Format;

pub const USO: &str = "\
uso: thumbcraft ENTRADA SAÍDA [opções]
     thumbcraft --formats | --version | --help | --detect ARQUIVO
     thumbcraft - - [opções]      lê o documento da stdin e grava a imagem na stdout

Gera uma miniatura (PNG) da primeira página de um documento Word, planilha ou apresentação.

Entrada e saída:
  -                       como ENTRADA lê o documento inteiro da stdin (até 256 MiB; acima disso, código 4);
                          como SAÍDA grava só os bytes da imagem na stdout (mensagens vão para a stderr);
                          vale também em --detect e pode ser usado só na entrada ou só na saída
      --input-name <nome> dica de nome (extensão) para a escolha do motor quando a entrada é '-'
                          (sem ela decide só o conteúdo; formato da saída '-': PNG, ou use --format)

Tamanho final:
  -W, --width <px>        largura; sozinha, preserva a proporção
  -H, --height <px>       altura; sozinha, preserva a proporção
      --fit <cover|contain>      com largura e altura: preenche e corta (padrão) ou cabe inteira
      --gravity <pos>     top, bottom, left, right, center, top-left, top-right, bottom-left,
                          bottom-right (padrão: top no cover, center no contain)
      --background <cor>  fundo do contain e sob transparência: white (padrão), transparent ou #RRGGBB
Cortes (desligados por padrão):
      --trim              remove margens brancas ou transparentes
      --trim-pad <px>     margem mantida pelo trim, em px da renderização nativa (padrão 8)
      --crop x,y,l,a      retângulo em FRAÇÕES (0 a 1) da página renderizada, aplicado antes do resize
Renderização:
      --render-scale <auto|px/pt>  escala nativa (auto: o dobro do alvo, de 1 px/pt a 8192 px e 64 MP)
      --filter <lanczos|area>      filtro de redução (padrão lanczos)
      --page <N>          página, slide ou página de impressão, a partir de 1 (padrão 1)
Saída:
      --format <png|jpeg|webp>  (padrão: pela extensão da saída; WebP sem perdas)
      --quality <1-100>   qualidade do JPEG (padrão 85)
      --native-scale <S>  teste: grava a renderização nativa a S px/pt, sem pós-processamento

Códigos de saída: 0 ok; 2 uso incorreto; 3 formato não suportado; 4 arquivo inválido ou falha ao
abrir/renderizar; 5 erro interno. Com panic=abort uma falha interna derruba o processo: use um
processo por arquivo e um timeout.
";

/// O que a linha de comando pediu.
#[derive(Debug)]
pub enum Cmd {
    Help,
    Version,
    Formats,
    /// Imprime o motor que trataria o arquivo (word, grid, deck, pdf).
    Detect { input: String, input_name: Option<String> },
    Run(Job),
}

/// Um trabalho de miniatura.
#[derive(Debug)]
pub struct Job {
    pub input: String,
    /// Dica de nome para o roteamento quando a entrada é `-` (stdin).
    pub input_name: Option<String>,
    pub output: String,
    pub opts: Opts,
    pub format: Format,
    /// Qualidade do JPEG (1 a 100).
    pub quality: u8,
    /// Modo de teste: escala nativa, sem pós-processamento.
    pub native: Option<f64>,
}

fn num<T: std::str::FromStr>(flag: &str, v: &str) -> Res<T> {
    v.parse::<T>().map_err(|_| Error::Usage(format!("valor inválido para {flag}: '{v}'")))
}

/// Analisa os argumentos (sem o nome do programa).
pub fn parse(args: &[String]) -> Res<Cmd> {
    let mut o = Opts::default();
    let mut pos: Vec<String> = Vec::new();
    let mut native = None;
    let mut detect = false;
    let mut input_name: Option<String> = None;
    let mut format: Option<Format> = None;
    let mut quality: u8 = 85;
    let mut it = args.iter().peekable();
    while let Some(a) = it.next() {
        if !a.starts_with('-') || a == "-" {
            pos.push(a.clone());
            continue;
        }
        let (flag, inline) = match a.split_once('=') {
            Some((f, v)) if f.starts_with("--") => (f.to_string(), Some(v.to_string())),
            _ => (a.clone(), None),
        };
        let mut value = |f: &str| -> Res<String> {
            if let Some(v) = &inline {
                return Ok(v.clone());
            }
            it.next().cloned().ok_or_else(|| Error::Usage(format!("a opção {f} precisa de um valor")))
        };
        match flag.as_str() {
            "-h" | "--help" => return Ok(Cmd::Help),
            "--version" | "-V" => return Ok(Cmd::Version),
            "--formats" => return Ok(Cmd::Formats),
            "--detect" => detect = true,
            "--input-name" => input_name = Some(value("--input-name")?),
            "-W" | "--width" => o.width = Some(num("--width", &value("--width")?)?),
            "-H" | "--height" => o.height = Some(num("--height", &value("--height")?)?),
            "--fit" => o.fit = Fit::parse(&value("--fit")?)?,
            "--gravity" => o.gravity = Some(Gravity::parse(&value("--gravity")?)?),
            "--background" => o.background = Background::parse(&value("--background")?)?,
            "--trim" => o.trim = true,
            "--trim-pad" => o.trim_pad = num("--trim-pad", &value("--trim-pad")?)?,
            "--crop" => o.crop = Some(Crop::parse(&value("--crop")?)?),
            "--render-scale" => o.scale = RenderScale::parse(&value("--render-scale")?)?,
            "--filter" => o.filter = Filter::parse(&value("--filter")?)?,
            "--format" => format = Some(Format::parse(&value("--format")?)?),
            "--quality" => quality = num("--quality", &value("--quality")?)?,
            "--page" => o.page = num("--page", &value("--page")?)?,
            "--native-scale" => {
                let s: f64 = num("--native-scale", &value("--native-scale")?)?;
                if !(s.is_finite() && s > 0.0) {
                    return Err(Error::Usage("--native-scale deve ser um número maior que 0".into()));
                }
                native = Some(s);
            }
            other => return Err(Error::Usage(format!("opção desconhecida: {other} (veja thumbcraft --help)"))),
        }
    }
    if detect {
        return match pos.as_slice() {
            [f] => Ok(Cmd::Detect { input: f.clone(), input_name }),
            _ => Err(Error::Usage("--detect recebe um único arquivo".into())),
        };
    }
    if pos.len() != 2 {
        return Err(Error::Usage(format!("esperava ENTRADA e SAÍDA, recebi {} argumento(s) posicional(is); veja thumbcraft --help", pos.len())));
    }
    o.check()?;
    let output = pos.pop().unwrap_or_default();
    let input = pos.pop().unwrap_or_default();
    let ext = thumbcraft_core::route::ext_of(&output);
    let format = match format {
        Some(f) => f,
        None => Format::from_ext(&ext)?,
    };
    if !(1..=100).contains(&quality) {
        return Err(Error::Usage("--quality deve ficar entre 1 e 100".into()));
    }
    Ok(Cmd::Run(Job { input, input_name, output, opts: o, format, quality, native }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Res<Cmd> {
        parse(&s.split_whitespace().map(String::from).collect::<Vec<_>>())
    }

    #[test]
    fn basic() {
        let Cmd::Run(j) = p("a.docx b.png -W 120 -H 120 --fit contain --gravity top-left --background=#ff0000").unwrap() else { panic!() };
        assert_eq!((j.opts.width, j.opts.height), (Some(120), Some(120)));
        assert_eq!(j.opts.fit, Fit::Contain);
        assert_eq!(j.opts.background, Background::Solid([255, 0, 0]));
    }

    #[test]
    fn usage_errors() {
        for s in ["a.docx", "a.docx b.png c.png", "a.docx b.png --quality 0", "a.docx b.bmp", "a.docx b.png -W", "a.docx b.png -W x", "a.docx b.png --nada", "a.docx b.png --fit zoom", "a.docx b.png --page 0", "a.docx b.png --native-scale 0"] {
            assert_eq!(p(s).unwrap_err().code(), 2, "{s}");
        }
    }

    #[test]
    fn auxiliary_commands() {
        assert!(matches!(p("--version").unwrap(), Cmd::Version));
        assert!(matches!(p("--formats").unwrap(), Cmd::Formats));
        assert!(matches!(p("-h").unwrap(), Cmd::Help));
    }

    #[test]
    fn dash_means_stdin_and_stdout() {
        let Cmd::Run(j) = p("- - -W 120").unwrap() else { panic!() };
        assert_eq!((j.input.as_str(), j.output.as_str()), ("-", "-"));
        assert_eq!(j.format, Format::Png);
        assert_eq!(j.input_name, None);
        let Cmd::Run(j) = p("- b.jpg").unwrap() else { panic!() };
        assert_eq!((j.input.as_str(), j.format), ("-", Format::Jpeg));
        let Cmd::Run(j) = p("a.docx - --format webp").unwrap() else { panic!() };
        assert_eq!((j.output.as_str(), j.format), ("-", Format::WebP));
    }

    #[test]
    fn input_name_is_a_hint_with_a_value() {
        let Cmd::Run(j) = p("- - --input-name x.pptx").unwrap() else { panic!() };
        assert_eq!(j.input_name.as_deref(), Some("x.pptx"));
        let Cmd::Run(j) = p("- - --input-name=rel.docx").unwrap() else { panic!() };
        assert_eq!(j.input_name.as_deref(), Some("rel.docx"));
        assert_eq!(p("- - --input-name").unwrap_err().code(), 2);
    }

    #[test]
    fn detect_accepts_stdin_and_hint() {
        let Cmd::Detect { input, input_name } = p("--detect - --input-name a.xlsx").unwrap() else { panic!() };
        assert_eq!((input.as_str(), input_name.as_deref()), ("-", Some("a.xlsx")));
        assert_eq!(p("--detect").unwrap_err().code(), 2);
    }

    #[test]
    fn native() {
        let Cmd::Run(j) = p("a.pptx b.png --native-scale 2").unwrap() else { panic!() };
        assert_eq!(j.native, Some(2.0));
    }
}
