//! Guarda de layout: detecta que uma atualização dos *craft mexeu na posição ou na quantidade de tinta
//! do que é desenhado (docs/UPDATING.md, "Onde uma atualização quebra").
//!
//! Para cada arquivo de `FIXTURES` o teste renderiza a página 1 em um processo novo e mede: dimensões
//! da imagem, retângulo que contém todos os pixels que não são fundo (a "caixa de tinta") e quantos pixels
//! são tinta. Compara com `layout-<plataforma>.tsv`. Dimensões têm de ser iguais; a caixa pode andar
//! `TOL_CAIXA` px e a tinta `TOL_TINTA` (fração). Isso pega o que o teste de "PNG válido" não pega:
//! título e corpo que sobem alguns pixels, uma linha que muda de página, um gráfico que encolhe, texto que sai.
//!
//! Quando a mudança é esperada (um motor novo, uma fonte nova): confira as imagens com
//! `scripts/compare-runs.py` e regrave com
//!
//! ```text
//! THUMBCRAFT_RECORD=1 cargo test -p thumbcraft --test layout
//! ```
//!
//! O texto depende das fontes do sistema (Windows e Linux divergem, ver docs/PITFALLS.md), então há um arquivo
//! por plataforma. Sem o arquivo da plataforma o teste só avisa; grave-o na primeira vez que rodar nela.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_thumbcraft");
const TIMEOUT: Duration = Duration::from_secs(60);
/// Quanto cada borda da caixa de tinta pode andar, em px (a 2 px/pt: meio ponto e pouco).
const TOL_CAIXA: i64 = 6;
/// Variação relativa aceita no total de pixels de tinta.
const TOL_TINTA: f64 = 0.15;

/// (arquivo no corpus, opções). Cobre os quatro motores e os pontos que já mudaram em atualizações:
/// espaço entre título e corpo (Word), quebra de página (Word), banner e gráfico (Excel), texto não latino (Excel),
/// células mescladas e bordas (Excel), tabelas e gráficos (PowerPoint) e o PDF.
const FIXTURES: &[(&str, &[&str])] = &[
    ("docx/01_docx_simples_acentos.docx", &["--native-scale", "2"]),
    ("docx/02_docx_titulos_listas.docx", &["--native-scale", "2"]),
    ("docx/03_docx_tabelas.docx", &["--native-scale", "2"]),
    ("docx/11_odt_rico.odt", &["--native-scale", "2"]),
    ("docx/12_rtf_rico.rtf", &["--native-scale", "2"]),
    ("docx/19_md_multipagina.md", &["--native-scale", "2"]),
    ("xlsx/01-texto-acentos.xlsx", &["--native-scale", "2"]),
    ("xlsx/07-celulas-mescladas.xlsx", &["--native-scale", "2"]),
    ("xlsx/08-bordas-cores-fundo-fonte.xlsx", &["--native-scale", "2"]),
    ("xlsx/31-texto-nao-latino-emoji.xlsx", &["--native-scale", "2"]),
    ("xlsx/34-painel-composto.xlsx", &["--native-scale", "2"]),
    ("xlsx/regressao-amostras/10_mixed.xlsx", &["--native-scale", "2"]),
    ("pptx/01_titulo_acentos.pptx", &["--native-scale", "2"]),
    ("pptx/07_tabela_com_bordas.pptx", &["--native-scale", "2"]),
    ("pptx/09_grafico_barras.pptx", &["--native-scale", "2"]),
    ("pdf/11_word_tabelas.pdf", &["-W", "600"]),
];

#[derive(Debug, Clone, PartialEq)]
struct Medida {
    largura: u32,
    altura: u32,
    caixa: [u32; 4],
    tinta: u64,
}

fn raiz() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn plataforma() -> &'static str {
    if cfg!(windows) { "windows" } else { "linux" }
}

fn arquivo_expectativas() -> PathBuf {
    raiz().join("tests").join(format!("layout-{}.tsv", plataforma()))
}

fn renderizar(rel: &str, extra: &[&str], saida: &Path) {
    let entrada = raiz().join("../../corpus").join(rel);
    let mut child = Command::new(BIN)
        .arg(&entrada)
        .arg(saida)
        .args(extra)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("não foi possível executar o thumbcraft");
    let t0 = Instant::now();
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if t0.elapsed() > TIMEOUT {
            let _ = child.kill();
            panic!("timeout renderizando {rel}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let o = child.wait_with_output().unwrap();
    assert_eq!(o.status.code(), Some(0), "{rel}: {}", String::from_utf8_lossy(&o.stderr));
}

/// Mede a imagem: tinta é todo pixel que, composto sobre branco, difere do branco em algum canal por mais de 20.
fn medir(png: &Path) -> Medida {
    let f = std::fs::File::open(png).unwrap();
    let mut dec = png::Decoder::new(std::io::BufReader::new(f));
    dec.set_transformations(png::Transformations::EXPAND);
    let mut r = dec.read_info().expect("PNG inválido");
    let mut buf = vec![0; r.output_buffer_size().expect("tamanho do PNG")];
    let info = r.next_frame(&mut buf).expect("quadro do PNG");
    assert_eq!(info.bit_depth, png::BitDepth::Eight, "profundidade inesperada");
    let canais = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Grayscale => 1,
        other => panic!("tipo de cor inesperado: {other:?}"),
    };
    let (w, h) = (info.width as usize, info.height as usize);
    let (mut x0, mut y0, mut x1, mut y1, mut tinta) = (w, h, 0usize, 0usize, 0u64);
    for y in 0..h {
        for x in 0..w {
            let p = &buf[(y * w + x) * canais..(y * w + x + 1) * canais];
            let (cor, alfa): ([u32; 3], u32) = match canais {
                4 => ([p[0] as u32, p[1] as u32, p[2] as u32], p[3] as u32),
                3 => ([p[0] as u32, p[1] as u32, p[2] as u32], 255),
                2 => ([p[0] as u32; 3], p[1] as u32),
                _ => ([p[0] as u32; 3], 255),
            };
            let sobre_branco = |c: u32| (c * alfa + 255 * (255 - alfa)) / 255;
            if cor.iter().any(|&c| sobre_branco(c) < 235) {
                tinta += 1;
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    if tinta == 0 {
        (x0, y0) = (0, 0);
    }
    Medida { largura: w as u32, altura: h as u32, caixa: [x0 as u32, y0 as u32, x1 as u32, y1 as u32], tinta }
}

fn ler_expectativas(texto: &str) -> BTreeMap<String, Medida> {
    texto
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("arquivo\t"))
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            assert_eq!(c.len(), 8, "linha inválida: {l}");
            let n = |i: usize| c[i].parse::<u64>().unwrap_or_else(|_| panic!("número inválido em {l}"));
            (
                c[0].to_string(),
                Medida { largura: n(1) as u32, altura: n(2) as u32, caixa: [n(3) as u32, n(4) as u32, n(5) as u32, n(6) as u32], tinta: n(7) },
            )
        })
        .collect()
}

fn escrever_expectativas(m: &BTreeMap<String, Medida>) -> String {
    let mut s = String::from("arquivo\tlargura\taltura\tx0\ty0\tx1\ty1\ttinta\n");
    for (nome, v) in m {
        s += &format!("{nome}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n", v.largura, v.altura, v.caixa[0], v.caixa[1], v.caixa[2], v.caixa[3], v.tinta);
    }
    s
}

fn comparar(nome: &str, esperado: &Medida, obtido: &Medida) -> Vec<String> {
    let mut erros = Vec::new();
    if (esperado.largura, esperado.altura) != (obtido.largura, obtido.altura) {
        erros.push(format!(
            "{nome}: dimensões {}x{}, esperado {}x{}",
            obtido.largura, obtido.altura, esperado.largura, esperado.altura
        ));
    }
    for (i, lado) in ["x0", "y0", "x1", "y1"].iter().enumerate() {
        let d = obtido.caixa[i] as i64 - esperado.caixa[i] as i64;
        if d.abs() > TOL_CAIXA {
            erros.push(format!("{nome}: {lado} da caixa de tinta andou {d:+} px ({} para {})", esperado.caixa[i], obtido.caixa[i]));
        }
    }
    let base = esperado.tinta.max(1) as f64;
    let var = (obtido.tinta as f64 - esperado.tinta as f64) / base;
    if var.abs() > TOL_TINTA {
        erros.push(format!("{nome}: tinta variou {:+.1}% ({} para {} px)", var * 100.0, esperado.tinta, obtido.tinta));
    }
    erros
}

#[test]
fn layout_of_reference_files_has_not_drifted() {
    let dir = std::env::temp_dir().join(format!("thumbcraft-layout-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut obtidas = BTreeMap::new();
    for (i, (rel, extra)) in FIXTURES.iter().enumerate() {
        let png = dir.join(format!("{i}.png"));
        renderizar(rel, extra, &png);
        obtidas.insert((*rel).to_string(), medir(&png));
    }
    let arq = arquivo_expectativas();
    if std::env::var_os("THUMBCRAFT_RECORD").is_some() {
        std::fs::write(&arq, escrever_expectativas(&obtidas)).unwrap();
        eprintln!("expectativas gravadas em {}", arq.display());
        return;
    }
    let Ok(texto) = std::fs::read_to_string(&arq) else {
        eprintln!("sem {}: nada a comparar nesta plataforma; grave com THUMBCRAFT_RECORD=1", arq.display());
        return;
    };
    let esperadas = ler_expectativas(&texto);
    let mut erros = Vec::new();
    for (nome, obtido) in &obtidas {
        match esperadas.get(nome) {
            Some(e) => erros.extend(comparar(nome, e, obtido)),
            None => erros.push(format!("{nome}: sem expectativa em {} (regrave com THUMBCRAFT_RECORD=1)", arq.display())),
        }
    }
    for nome in esperadas.keys().filter(|n| !obtidas.contains_key(*n)) {
        erros.push(format!("{nome}: está nas expectativas mas saiu de FIXTURES"));
    }
    assert!(erros.is_empty(), "o layout mudou (confira com scripts/compare-runs.py e, se for esperado, regrave):\n{}", erros.join("\n"));
}

#[test]
fn drift_check_catches_a_shifted_box_and_lost_ink() {
    let base = Medida { largura: 100, altura: 200, caixa: [10, 10, 90, 150], tinta: 1000 };
    assert!(comparar("a", &base, &base).is_empty());
    let subiu = Medida { caixa: [10, 20, 90, 160], ..base.clone() };
    assert_eq!(comparar("a", &base, &subiu).len(), 2, "y0 e y1 andaram 10 px");
    let sem_texto = Medida { tinta: 700, ..base.clone() };
    assert_eq!(comparar("a", &base, &sem_texto).len(), 1);
    let outra_pagina = Medida { altura: 220, ..base.clone() };
    assert_eq!(comparar("a", &base, &outra_pagina).len(), 1);
    let pequena = Medida { caixa: [12, 8, 92, 152], tinta: 1050, ..base.clone() };
    assert!(comparar("a", &base, &pequena).is_empty(), "variação pequena é aceita");
}

#[test]
fn expectations_roundtrip() {
    let mut m = BTreeMap::new();
    m.insert("docx/x.docx".to_string(), Medida { largura: 1224, altura: 1584, caixa: [1, 2, 3, 4], tinta: 99 });
    assert_eq!(ler_expectativas(&escrever_expectativas(&m)), m);
}
