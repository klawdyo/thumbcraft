//! Testes de ponta a ponta: executam o binário `thumbcraft` sobre o corpus versionado.
//!
//! Só compilam e rodam com os motores presentes (`scripts/fetch-upstream.sh` antes). Cada arquivo
//! roda em um processo novo com timeout, como pede o `panic = "abort"` (docs/ARCHITECTURE.md).
//! Os códigos esperados vêm de docs/ARCHITECTURE.md (tabela de códigos de saída) e dos MANIFEST.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_thumbcraft");
const TIMEOUT: Duration = Duration::from_secs(60);

struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

fn corpus(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus").join(rel)
}

/// Pasta de saída própria de cada teste, dentro do diretório temporário do sistema.
fn tmp(test: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("thumbcraft-e2e-{}-{test}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn run(args: &[&str]) -> Out {
    let mut child = Command::new(BIN)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
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
            panic!("timeout em: thumbcraft {}", args.join(" "));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let o = child.wait_with_output().unwrap();
    Out {
        // Morte por sinal (panic = abort) não tem código: vira -1 e falha qualquer asserção.
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

fn png_dims(path: &Path) -> (u32, u32) {
    let f = std::fs::File::open(path).unwrap_or_else(|e| panic!("sem saída em {}: {e}", path.display()));
    let r = png::Decoder::new(std::io::BufReader::new(f)).read_info().expect("PNG inválido");
    let i = r.info();
    (i.width, i.height)
}

fn generate(input: &str, out: &Path, extra: &[&str]) -> Out {
    let inp = corpus(input);
    let mut a = vec![inp.to_str().unwrap(), out.to_str().unwrap()];
    a.extend_from_slice(extra);
    run(&a)
}

// ---------- informação e uso ----------

#[test]
fn version_and_formats_go_to_stdout() {
    let v = run(&["--version"]);
    assert_eq!(v.code, 0);
    assert!(v.stdout.starts_with("thumbcraft "), "{}", v.stdout);
    for engine in ["wordcraft", "gridcraft", "deckcraft", "craft-pdf", "hayro"] {
        assert!(v.stdout.contains(engine), "falta {engine}");
    }
    let f = run(&["--formats"]);
    assert_eq!(f.code, 0);
    for ext in ["docx", "xlsx", "pptx", "pdf"] {
        assert!(f.stdout.contains(ext), "falta {ext}");
    }
    assert!(f.stdout.contains("pdf (craft-pdf"), "o PDF deve aparecer como motor: {}", f.stdout);
    assert!(!f.stdout.contains("reservado"), "o PDF não é mais reservado: {}", f.stdout);
    assert!(f.stderr.is_empty());
}

#[test]
fn help_lists_options() {
    let h = run(&["--help"]);
    assert_eq!(h.code, 0);
    for o in ["--width", "--fit", "--trim", "--crop", "--page", "--format", "--native-scale", "--input-name"] {
        assert!(h.stdout.contains(o), "falta {o}");
    }
}

#[test]
fn incorrect_usage_is_code_2_with_message_on_stderr() {
    for args in [&["so-uma-entrada.docx"][..], &["a.docx", "b.png", "--nada"], &["a.docx", "b.png", "-W", "0"], &["a.docx", "b.bmp"]] {
        let r = run(args);
        assert_eq!(r.code, 2, "{args:?}");
        assert!(r.stderr.starts_with("thumbcraft: "), "{}", r.stderr);
        assert!(r.stdout.is_empty());
    }
}

#[test]
fn missing_input_is_code_4() {
    let d = tmp("inexistente");
    let r = run(&["nao-existe.docx", d.join("x.png").to_str().unwrap()]);
    assert_eq!(r.code, 4);
    assert!(!d.join("x.png").exists());
}

// ---------- detecção de motor ----------

#[test]
fn detect_by_extension_and_content() {
    let cases = [
        ("docx/01_docx_simples_acentos.docx", "word"),
        ("docx/11_odt_rico.odt", "word"),
        ("docx/16_txt_simples.txt", "word"),
        // Extensão errada: o conteúdo vence.
        ("docx/25_extensao_errada_txt_como.docx", "word"),
        ("xlsx/02-valores-simples.xlsx", "grid"),
        ("xlsx/45-csv-utf8-acentos.csv", "grid"),
        ("pptx/01_titulo_acentos.pptx", "deck"),
        ("pptx/33_slideshow.ppsx", "deck"),
        ("pdf/10_word_simples_acentos.pdf", "pdf"),
        // Lixo antes do %PDF: sem assinatura no início, vale a extensão.
        ("pdf/74_lixo_antes_do_cabecalho.pdf", "pdf"),
        // Texto com extensão .pdf segue para o WordCraft (docs/BUILD.md, 11:40).
        ("docx/28_extensao_errada_md_como.pdf", "word"),
    ];
    for (file, engine) in cases {
        let r = run(&["--detect", corpus(file).to_str().unwrap()]);
        assert_eq!(r.code, 0, "{file}: {}", r.stderr);
        assert_eq!(r.stdout.trim(), engine, "{file}");
    }
}

#[test]
fn unsupported_formats_are_code_3() {
    for file in ["pptx/35_legado_formas.ppt", "pptx/37_formas_gradiente.odp"] {
        let d = tmp("nao-suportado");
        let r = generate(file, &d.join("x.png"), &[]);
        assert_eq!(r.code, 3, "{file}: {}", r.stderr);
        assert!(!d.join("x.png").exists(), "{file} não devia gravar saída");
    }
}

// ---------- geração por motor ----------

#[test]
fn produces_valid_png_for_each_engine() {
    let d = tmp("motores");
    for file in ["docx/01_docx_simples_acentos.docx", "xlsx/02-valores-simples.xlsx", "pptx/01_titulo_acentos.pptx"] {
        let out = d.join(format!("{}.png", file.replace('/', "_")));
        let r = generate(file, &out, &[]);
        assert_eq!(r.code, 0, "{file}: {}", r.stderr);
        let (w, h) = png_dims(&out);
        assert!(w > 0 && h > 0, "{file}");
    }
}

#[test]
fn width_and_height_give_exact_size() {
    let d = tmp("tamanho");
    for file in ["docx/01_docx_simples_acentos.docx", "xlsx/02-valores-simples.xlsx", "pptx/01_titulo_acentos.pptx"] {
        for fit in ["cover", "contain"] {
            let out = d.join("t.png");
            let r = generate(file, &out, &["-W", "120", "-H", "90", "--fit", fit]);
            assert_eq!(r.code, 0, "{file} {fit}: {}", r.stderr);
            assert_eq!(png_dims(&out), (120, 90), "{file} {fit}");
        }
    }
}

#[test]
fn width_only_keeps_aspect_ratio() {
    let d = tmp("proporcao");
    let out = d.join("p.png");
    let r = generate("pptx/26_proporcao_16x9.pptx", &out, &["-W", "320"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let (w, h) = png_dims(&out);
    assert_eq!(w, 320);
    assert!((h as i64 - 180).abs() <= 1, "16:9 esperado, saiu {w}x{h}");
}

#[test]
fn output_is_deterministic() {
    let d = tmp("deterministico");
    let (a, b) = (d.join("a.png"), d.join("b.png"));
    for o in [&a, &b] {
        let r = generate("docx/02_docx_titulos_listas.docx", o, &["-W", "200", "-H", "200"]);
        assert_eq!(r.code, 0, "{}", r.stderr);
    }
    assert_eq!(std::fs::read(a).unwrap(), std::fs::read(b).unwrap());
}

// ---------- opções ----------

#[test]
fn output_formats_by_extension_and_by_option() {
    let d = tmp("formatos");
    let cases: [(&str, &[&str], &[u8]); 4] = [
        ("a.png", &[], &[0x89, b'P', b'N', b'G']),
        ("a.jpg", &[], &[0xFF, 0xD8, 0xFF]),
        ("a.webp", &[], b"RIFF"),
        // --format vence a extensão.
        ("b.dat", &["--format", "jpeg", "--quality", "50"], &[0xFF, 0xD8, 0xFF]),
    ];
    for (name, extra, magic) in cases {
        let out = d.join(name);
        let mut e = vec!["-W", "100"];
        e.extend_from_slice(extra);
        let r = generate("docx/01_docx_simples_acentos.docx", &out, &e);
        assert_eq!(r.code, 0, "{name}: {}", r.stderr);
        assert!(std::fs::read(&out).unwrap().starts_with(magic), "{name}");
    }
}

#[test]
fn page_picks_the_page_and_missing_page_is_code_4() {
    let d = tmp("page");
    let out = d.join("p.png");
    let ok = generate("pptx/21_apresentacao_60_slides.pptx", &out, &["--page", "60", "-W", "100"]);
    assert_eq!(ok.code, 0, "{}", ok.stderr);
    let missing = generate("pptx/21_apresentacao_60_slides.pptx", &d.join("q.png"), &["--page", "61"]);
    assert_eq!(missing.code, 4);
    assert!(!d.join("q.png").exists());
    let zero = generate("pptx/21_apresentacao_60_slides.pptx", &d.join("z.png"), &["--page", "0"]);
    assert_eq!(zero.code, 2);
}

#[test]
fn trim_and_crop_work_with_real_engine() {
    let d = tmp("trim-crop");
    let out = d.join("t.png");
    let r = generate("docx/01_docx_simples_acentos.docx", &out, &["--trim", "-W", "300"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let (w, h) = png_dims(&out);
    // Letter ou A4 sem trim seria bem mais alta que larga; com trim o texto curto fica mais baixo.
    assert_eq!(w, 300);
    assert!(h < 300 * 11 / 8, "trim não encurtou a página: {w}x{h}");
    let c = generate("docx/01_docx_simples_acentos.docx", &out, &["--crop", "0,0,1,0.25", "-W", "300"]);
    assert_eq!(c.code, 0, "{}", c.stderr);
    let (cw, ch) = png_dims(&out);
    assert_eq!(cw, 300);
    assert!(ch < 300, "crop de 25% da altura deveria dar imagem mais larga que alta: {cw}x{ch}");
}

#[test]
fn native_scale_writes_native_render() {
    let d = tmp("native");
    let out = d.join("n.png");
    let r = generate("pptx/26_proporcao_16x9.pptx", &out, &["--native-scale", "1"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let (w, h) = png_dims(&out);
    // 16:9 a 1 px/pt (DeckCraft usa 960x540 em escala 1, docs/RESULTS.md).
    assert_eq!((w, h), (960, 540));
}

// ---------- entradas inválidas: sempre erro limpo, nunca panic ----------

#[test]
fn invalid_inputs_give_code_4_without_output() {
    let cases = [
        "docx/21_vazio.docx",
        "docx/23_corrompido_docx_truncado.docx",
        "docx/30_lixo_binario.docx",
        "pptx/40_vazio_0bytes.pptx",
        "pptx/41_truncado.pptx",
        // O 42 (bytes corrompidos) o DeckCraft ainda renderiza (MANIFEST: ok), então não entra aqui.
        "pptx/43_texto_puro.pptx",
        "pptx/44_slide1_xml_quebrado.pptx",
        "xlsx/49-vazio-0bytes.xlsx",
        "xlsx/50-xlsx-truncado.xlsx",
        "xlsx/51-xlsx-bytes-corrompidos.xlsx",
    ];
    for file in cases {
        let d = tmp("invalidos");
        let out = d.join("x.png");
        let r = generate(file, &out, &["-W", "100"]);
        assert_eq!(r.code, 4, "{file}: código {} ({})", r.code, r.stderr);
        assert!(r.stderr.starts_with("thumbcraft: "), "{file}: {}", r.stderr);
        assert!(!out.exists(), "{file} não devia gravar saída");
    }
}

/// Falha ao gravar a saída é erro interno (5), não culpa da entrada.
#[test]
fn output_in_missing_folder_is_code_5() {
    let d = tmp("sem-pasta");
    let out = d.join("nao/existe/x.png");
    let r = generate("docx/01_docx_simples_acentos.docx", &out, &["-W", "50"]);
    assert_eq!(r.code, 5, "{}", r.stderr);
}

/// Nenhum arquivo do corpus Word, PowerPoint e Excel pode derrubar o processo (código fora de 0 a 5).
#[test]
fn no_corpus_file_crashes_the_process() {
    let d = tmp("varredura");
    let mut bad = Vec::new();
    for sub in ["docx", "pptx", "xlsx"] {
        let mut files: Vec<_> = std::fs::read_dir(corpus(sub))
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .filter(|p| {
                let n = p.file_name().unwrap().to_string_lossy();
                n != "MANIFEST.tsv" && !n.starts_with("README")
            })
            .collect();
        files.sort();
        for p in files {
            let out = d.join("v.png");
            let _ = std::fs::remove_file(&out);
            let r = run(&[p.to_str().unwrap(), out.to_str().unwrap(), "-W", "120", "-H", "120"]);
            if !(0..=5).contains(&r.code) {
                bad.push(format!("{}: código {}", p.display(), r.code));
            }
            if r.code == 0 {
                assert!(png_dims(&out).0 == 120, "{}", p.display());
            } else {
                assert!(!out.exists(), "{} falhou mas gravou saída", p.display());
            }
        }
    }
    assert!(bad.is_empty(), "processos derrubados:\n{}", bad.join("\n"));
}

// ---------- PDF ----------

/// Monta um PDF mínimo e válido: uma página por tamanho em pontos (largura, altura), todas com o mesmo conteúdo.
fn minimal_pdf(pages: &[(u32, u32)], content: &str, extra_page_entries: &str) -> Vec<u8> {
    let mut objects: Vec<String> = vec!["<< /Type /Catalog /Pages 2 0 R >>".into()];
    let kids: Vec<String> = (0..pages.len()).map(|i| format!("{} 0 R", 3 + 2 * i)).collect();
    objects.push(format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), pages.len()));
    for (i, (w, h)) in pages.iter().enumerate() {
        objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Contents {} 0 R {extra_page_entries} >>", 4 + 2 * i));
        objects.push(format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()));
    }
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).as_bytes());
    out
}

/// Primeiro pixel (RGB) de um PNG gravado pelo thumbcraft.
fn first_pixel(path: &Path) -> [u8; 3] {
    let file = std::fs::File::open(path).unwrap_or_else(|e| panic!("sem saída em {}: {e}", path.display()));
    let mut reader = png::Decoder::new(std::io::BufReader::new(file)).read_info().expect("PNG inválido");
    let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
    reader.next_frame(&mut buffer).unwrap();
    [buffer[0], buffer[1], buffer[2]]
}

#[test]
fn pdf_valid_corpus_files_produce_png_with_the_requested_size() {
    let dir = tmp("pdf-validos");
    let mut count = 0;
    let mut names: Vec<_> = std::fs::read_dir(corpus("pdf")).unwrap().filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    for name in names {
        // Fora: zero páginas (43), com senha de abertura (50), quebrados e com extensão errada (70 a 78).
        if !name.ends_with(".pdf") || name.starts_with("43_") || name.starts_with("50_") || name.starts_with('7') {
            continue;
        }
        let out = dir.join("p.png");
        let r = generate(&format!("pdf/{name}"), &out, &["-W", "120", "-H", "90"]);
        assert_eq!(r.code, 0, "{name}: {}", r.stderr);
        assert_eq!(png_dims(&out), (120, 90), "{name}");
        count += 1;
    }
    assert!(count >= 30, "esperava ao menos 30 PDFs válidos, rodei {count}");
}

#[test]
fn pdf_invalid_files_give_code_4_without_output() {
    for file in [
        "pdf/43_zero_paginas.pdf",
        "pdf/50_cripto_senha_aes256.pdf",
        "pdf/70_truncado_50pct.pdf",
        "pdf/71_truncado_90pct.pdf",
        "pdf/75_zero_bytes.pdf",
        // Imagem com extensão .pdf: vai para o motor de PDF e falha como arquivo inválido (não como formato não suportado).
        "pdf/78_jpeg_com_extensao_pdf.pdf",
    ] {
        let d = tmp("pdf-invalidos");
        let out = d.join("x.png");
        let r = generate(file, &out, &["-W", "100"]);
        assert_eq!(r.code, 4, "{file}: código {} ({})", r.code, r.stderr);
        assert!(r.stderr.starts_with("thumbcraft: "), "{file}: {}", r.stderr);
        assert!(!out.exists(), "{file} não devia gravar saída");
    }
}

#[test]
fn pdf_damaged_but_recoverable_files_still_render() {
    // Corrompido no meio, xref corrompida e 1 KB de lixo antes do cabeçalho: o leitor se recupera.
    for file in ["pdf/72_corrompido_meio.pdf", "pdf/73_corrompido_xref.pdf", "pdf/74_lixo_antes_do_cabecalho.pdf"] {
        let d = tmp("pdf-recuperaveis");
        let out = d.join("x.png");
        let r = generate(file, &out, &["-W", "100"]);
        assert_eq!(r.code, 0, "{file}: {}", r.stderr);
        assert_eq!(png_dims(&out).0, 100, "{file}");
    }
}

#[test]
fn text_and_html_with_pdf_extension_still_go_to_word() {
    for file in ["pdf/76_texto_puro_com_extensao_pdf.pdf", "pdf/77_html_com_extensao_pdf.pdf"] {
        let d = tmp("pdf-extensao-errada");
        let r = generate(file, &d.join("x.png"), &["-W", "100"]);
        assert_eq!(r.code, 0, "{file}: {}", r.stderr);
    }
}

#[test]
fn pdf_page_option_picks_the_page() {
    let d = tmp("pdf-page");
    let ok = generate("pdf/38_ppt_60_slides.pdf", &d.join("a.png"), &["--page", "60", "-W", "100"]);
    assert_eq!(ok.code, 0, "{}", ok.stderr);
    let beyond = generate("pdf/38_ppt_60_slides.pdf", &d.join("b.png"), &["--page", "61"]);
    assert_eq!(beyond.code, 4);
    assert!(!d.join("b.png").exists());
}

#[test]
fn pdf_rotated_page_comes_out_landscape() {
    // O arquivo tem /Rotate 90 sobre uma página em retrato; o Poppler erra a largura aqui (docs/PDF-WINDOWS.md).
    let d = tmp("pdf-rotacionado");
    let out = d.join("r.png");
    let r = generate("pdf/49_pagina_rotacionada_90.pdf", &out, &["-W", "200"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let (w, h) = png_dims(&out);
    assert_eq!(w, 200);
    assert!(h < w, "página rotacionada deveria sair em paisagem: {w}x{h}");
}

#[test]
fn pdf_blank_page_is_opaque_white() {
    let d = tmp("pdf-branco");
    let out = d.join("b.png");
    let r = generate("pdf/42_pagina_vazia.pdf", &out, &["-W", "100"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(first_pixel(&out), [255, 255, 255]);
}

#[test]
fn pdf_generated_in_code_renders_and_respects_the_page_size() {
    let d = tmp("pdf-gerado");
    let input = d.join("in.pdf");
    // 200 x 100 pt com um retângulo preto na metade esquerda (a origem do PDF é o canto inferior esquerdo).
    std::fs::write(&input, minimal_pdf(&[(200, 100)], "0 g 0 0 100 100 re f", "")).unwrap();
    let out = d.join("o.png");
    let r = run(&[input.to_str().unwrap(), out.to_str().unwrap(), "-W", "200", "--render-scale", "1"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(png_dims(&out), (200, 100));
    assert_eq!(first_pixel(&out), [0, 0, 0], "o canto superior esquerdo cai dentro do retângulo preto");
}

#[test]
fn pdf_giant_page_stays_within_limits() {
    // 14400 x 14400 pt (200 polegadas): o pipeline escolhe a escala; o resultado precisa caber nos limites.
    let d = tmp("pdf-gigante");
    let input = d.join("g.pdf");
    std::fs::write(&input, minimal_pdf(&[(14400, 14400)], "0 0 1 rg 1000 1000 8000 8000 re f", "")).unwrap();
    let out = d.join("g.png");
    let r = run(&[input.to_str().unwrap(), out.to_str().unwrap(), "-W", "600"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(png_dims(&out), (600, 600));
}

// ---------- stdin e stdout (`-`) ----------

struct RawOut {
    code: i32,
    stdout: Vec<u8>,
    stderr: String,
}

/// Executa o binário com `stdin` real (os bytes dados) e devolve a stdout sem converter para texto.
fn run_raw(args: &[&str], stdin: &[u8]) -> RawOut {
    let mut child = Command::new(BIN).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("não foi possível executar o thumbcraft");
    let mut pipe = child.stdin.take().unwrap();
    let data = stdin.to_vec();
    // Em outra thread: o binário pode terminar sem ler tudo (erro de uso) e o write falharia aqui.
    let writer = std::thread::spawn(move || {
        use std::io::Write;
        let _ = pipe.write_all(&data);
    });
    let o = child.wait_with_output().unwrap();
    let _ = writer.join();
    RawOut { code: o.status.code().unwrap_or(-1), stdout: o.stdout, stderr: String::from_utf8_lossy(&o.stderr).into_owned() }
}

fn png_size(bytes: &[u8]) -> (u32, u32) {
    let i = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().expect("PNG inválido").info().clone();
    (i.width, i.height)
}

const PNG_MAGIC: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

#[test]
fn stdin_to_stdout_gives_only_the_image() {
    let doc = std::fs::read(corpus("docx/01_docx_simples_acentos.docx")).unwrap();
    let r = run_raw(&["-", "-", "-W", "120"], &doc);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.starts_with(PNG_MAGIC), "a stdout deve começar pelo PNG");
    assert_eq!(png_size(&r.stdout).0, 120);
    assert!(r.stderr.is_empty(), "{}", r.stderr);
}

#[test]
fn stdin_and_file_inputs_give_identical_bytes() {
    let d = tmp("stdin_same");
    for file in ["docx/01_docx_simples_acentos.docx", "xlsx/02-valores-simples.xlsx", "pptx/01_titulo_acentos.pptx"] {
        let out = d.join("file.png");
        assert_eq!(generate(file, &out, &["-W", "120"]).code, 0, "{file}");
        let via_stdin = run_raw(&["-", "-", "-W", "120"], &std::fs::read(corpus(file)).unwrap());
        assert_eq!(via_stdin.code, 0, "{file}: {}", via_stdin.stderr);
        assert_eq!(std::fs::read(&out).unwrap(), via_stdin.stdout, "{file}");
    }
}

#[test]
fn stdin_with_file_output_and_file_with_stdout() {
    let d = tmp("stdin_mixed");
    let doc = std::fs::read(corpus("docx/01_docx_simples_acentos.docx")).unwrap();
    let out = d.join("a.png");
    let r = run_raw(&["-", out.to_str().unwrap(), "-W", "90"], &doc);
    assert_eq!((r.code, png_dims(&out).0), (0, 90), "{}", r.stderr);
    assert!(r.stdout.is_empty());
    let r = run_raw(&[corpus("docx/01_docx_simples_acentos.docx").to_str().unwrap(), "-", "-W", "90"], b"");
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(png_size(&r.stdout).0, 90);
}

#[test]
fn stdout_takes_other_formats_by_option() {
    let doc = std::fs::read(corpus("docx/01_docx_simples_acentos.docx")).unwrap();
    let r = run_raw(&["-", "-", "-W", "100", "--format", "jpeg"], &doc);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.starts_with(&[0xFF, 0xD8, 0xFF]));
}

#[test]
fn native_scale_accepts_stdout() {
    let pptx = std::fs::read(corpus("pptx/26_proporcao_16x9.pptx")).unwrap();
    let r = run_raw(&["-", "-", "--native-scale", "1"], &pptx);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.starts_with(PNG_MAGIC));
}

#[test]
fn input_name_routes_when_content_is_silent() {
    let text = b"Ola, mundo\n";
    let without = run_raw(&["-", "-", "-W", "80"], text);
    assert_eq!(without.code, 3, "sem dica e sem sinal no conteúdo");
    assert!(without.stdout.is_empty());
    let with = run_raw(&["-", "-", "-W", "80", "--input-name", "nota.txt"], text);
    assert_eq!(with.code, 0, "{}", with.stderr);
    assert!(with.stdout.starts_with(PNG_MAGIC));
    let detect = run_raw(&["--detect", "-", "--input-name", "nota.txt"], text);
    assert_eq!((detect.code, String::from_utf8_lossy(&detect.stdout).trim()), (0, "word"));
}

#[test]
fn detect_from_stdin_matches_detect_from_file() {
    for file in ["docx/01_docx_simples_acentos.docx", "xlsx/02-valores-simples.xlsx", "pptx/01_titulo_acentos.pptx"] {
        let by_file = run(&["--detect", corpus(file).to_str().unwrap()]);
        let by_stdin = run_raw(&["--detect", "-"], &std::fs::read(corpus(file)).unwrap());
        assert_eq!((by_stdin.code, String::from_utf8_lossy(&by_stdin.stdout).into_owned()), (by_file.code, by_file.stdout), "{file}");
    }
}

#[test]
fn empty_stdin_is_code_4_with_nothing_on_stdout() {
    let r = run_raw(&["-", "-"], b"");
    assert_eq!(r.code, 4);
    assert!(r.stdout.is_empty());
    assert!(r.stderr.starts_with("thumbcraft: "), "{}", r.stderr);
}

#[test]
fn closed_stdout_is_code_5() {
    let doc = std::fs::read(corpus("docx/01_docx_simples_acentos.docx")).unwrap();
    let mut child = Command::new(BIN).args(["-", "-", "-W", "120"]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    drop(child.stdout.take());
    {
        use std::io::Write;
        let mut pipe = child.stdin.take().unwrap();
        let _ = pipe.write_all(&doc);
    }
    let o = child.wait_with_output().unwrap();
    assert_eq!(o.status.code(), Some(5), "{}", String::from_utf8_lossy(&o.stderr));
}
