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
    for engine in ["wordcraft", "gridcraft", "deckcraft"] {
        assert!(v.stdout.contains(engine), "falta {engine}");
    }
    let f = run(&["--formats"]);
    assert_eq!(f.code, 0);
    for ext in ["docx", "xlsx", "pptx"] {
        assert!(f.stdout.contains(ext), "falta {ext}");
    }
    assert!(f.stderr.is_empty());
}

#[test]
fn help_lists_options() {
    let h = run(&["--help"]);
    assert_eq!(h.code, 0);
    for o in ["--width", "--fit", "--trim", "--crop", "--page", "--format", "--native-scale"] {
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
    for file in ["pptx/35_legado_formas.ppt", "pptx/37_formas_gradiente.odp", "pdf/10_word_simples_acentos.pdf"] {
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
