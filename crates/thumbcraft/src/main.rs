//! thumbcraft: miniaturas de documentos Word, planilhas e apresentações em um binário só.
//!
//! Com `panic = "abort"` uma falha interna derruba o processo (SIGABRT): quem chama deve usar
//! um processo por arquivo e um timeout.

mod cli;
mod deck;
mod encode;
mod grid;
mod info;
mod open;
mod word;

use std::path::Path;
use std::process::ExitCode;

use thumbcraft_core::{Error, Res, post};

use cli::{Cmd, Job};

fn write(path: &str, bytes: &[u8]) -> Res<()> {
    std::fs::write(path, bytes).map_err(|e| Error::Internal(format!("não foi possível gravar '{path}': {e}")))
}

fn run(job: &Job) -> Res<()> {
    let input = Path::new(&job.input);
    let engine = open::engine_for(input)?;
    let mut doc = open::open(engine, input)?;
    if let Some(scale) = job.native {
        // Modo de teste: PNG nativo do motor, byte a byte como o do mini correspondente.
        let page = job.opts.page as usize;
        if page > doc.pages() {
            return Err(Error::Invalid(format!("o arquivo não tem {} {}: só há {} no total", doc.unit(), page, doc.pages())));
        }
        let png = doc.native_png(page - 1, scale)?;
        return write(&job.output, &png);
    }
    let done = post::process(doc.as_mut(), &job.opts)?;
    let bytes = encode::encode(&done.img, job.format)?;
    write(&job.output, &bytes)
}

fn real_main() -> u8 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = match cli::parse(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("thumbcraft: {e}");
            return e.code();
        }
    };
    match cmd {
        Cmd::Help => {
            print!("{}", cli::USO);
            0
        }
        Cmd::Version => {
            print!("{}", info::version());
            0
        }
        Cmd::Formats => {
            print!("{}", info::formats());
            0
        }
        Cmd::Detect(f) => match open::engine_for(Path::new(&f)) {
            Ok(e) => {
                println!("{}", e.name());
                0
            }
            Err(e) => {
                eprintln!("thumbcraft: {e}");
                e.code()
            }
        },
        Cmd::Run(job) => match run(&job) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("thumbcraft: {e}");
                e.code()
            }
        },
    }
}

fn main() -> ExitCode {
    // Pilha grande: layouts muito aninhados não estouram a pilha padrão de 8 MB.
    let t = std::thread::Builder::new().stack_size(128 << 20).spawn(real_main);
    match t.map(|h| h.join()) {
        Ok(Ok(code)) => ExitCode::from(code),
        _ => {
            eprintln!("thumbcraft: erro interno inesperado");
            ExitCode::from(5)
        }
    }
}
