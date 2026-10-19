# thumbcraft

A single command-line binary that takes a Word document, a spreadsheet or a presentation and writes a thumbnail image (PNG, JPEG or WebP). It bundles the rendering engines of three Rust projects from the [storytold](https://github.com/storytold) organization, with no dependency on LibreOffice, a GPU or any external service:

| Engine | Project | Input |
|---|---|---|
| WordCraft | https://github.com/storytold/wordcraft | Word and similar documents |
| GridCraft | https://github.com/storytold/gridcraft | Spreadsheets (Excel) |
| DeckCraft | https://github.com/storytold/deckcraft | Presentations (PowerPoint) |

The engine is chosen from the file content (and, when the content is ambiguous, the extension), so a file with the wrong extension still goes to the right engine. thumbcraft is meant to be called from a file-upload pipeline, one process per file.

> Most project documentation under `docs/` is written in Portuguese. This README and the code identifiers are in English.

## Supported formats

| Engine | Reads | Does not read |
|---|---|---|
| WordCraft | `.docx`, `.doc` (Word 97-2003), `.odt`, `.rtf`, `.html`, `.md`, `.tex`, `.txt` (plus docm, dotx, dotm, dot, WordML) | |
| GridCraft (with the PNG patch) | `.xlsx`, `.xlsm`, `.xls` (data, formulas and formats only), `.ods` and `.xlsb` (data only), `.csv`, `.tsv` | |
| DeckCraft | `.pptx`, `.potx`, `.ppsx` | **`.ppt`, `.odp`** |
| craft-pdf (Hayro) | `.pdf` (first page, or the page given by `--page`; password-protected, empty and truncated files fail with code 4) | |

PDF is rendered by Hayro through `craft-pdf`, inside the same binary; the comparison with other PDF libraries is in `docs/PDF.md` (Linux) and `docs/PDF-WINDOWS.md` (Windows). Known limitations per engine are in `docs/PITFALLS.md`. Every imported project is pinned to one revision, and each modification of their code is a small patch registered in `docs/UPSTREAM-CHANGES.md`; the binary carries one version of each package.

## Usage

```
thumbcraft INPUT OUTPUT [options]
thumbcraft --formats | --version | --help | --detect FILE
```

| Option | Meaning |
|---|---|
| `-W, --width <px>` / `-H, --height <px>` | Final size; one alone keeps the aspect ratio |
| `--fit <cover\|contain>` | With both sizes: fill and crop (default) or fit inside the box |
| `--gravity <pos>` | `top`, `bottom`, `left`, `right`, `center`, `top-left`, `top-right`, `bottom-left`, `bottom-right` (default: top for cover, center for contain) |
| `--background <color>` | Background for contain and for transparency: `white` (default), `transparent` or `#RRGGBB` |
| `--trim`, `--trim-pad <px>` | Remove blank or transparent margins, keeping a margin in native pixels (default 8) |
| `--crop x,y,w,h` | Crop rectangle as fractions (0 to 1) of the rendered page, applied before resizing |
| `--render-scale <auto\|px/pt>` | Native render scale (`auto` renders at about twice the target) |
| `--filter <lanczos\|area>` | Downscale filter (default `lanczos`) |
| `--page <N>` | Page, slide or print page, starting at 1 (default 1) |
| `--format <png\|jpeg\|webp>`, `--quality <1-100>` | Output format (default: from the output extension; WebP is lossless) and JPEG quality (default 85) |
| `--native-scale <S>` | Test mode: write the engine's native render at S px/pt, with no post-processing |

Example:

```sh
thumbcraft report.docx thumb.png -W 120 -H 120 --fit cover --gravity top
```

Rendering is done at high resolution first (the engines render natively and thumbcraft scales the result down), and only then cropped and resized.

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 2 | Invalid usage (bad option or value out of range) |
| 3 | Unsupported format (`.ppt`, `.odp`, unknown extension) |
| 4 | Invalid file, or failure to open or render it (includes a missing page and a damaged first slide) |
| 5 | Internal error (includes failing to write the output) |

Error messages are printed to stderr in Portuguese. The release profile uses `panic = "abort"`, so an internal engine failure kills the process (no message, status 134 on Linux): always run one process per file, with a timeout, and treat any status outside 0 to 5 as a failure.

## Why this project exists: a bridge

The craft engines keep evolving, and thumbcraft is a bridge in the meantime. For example, GridCraft cannot write PNG yet, so thumbcraft applies a patch (`patches/gridcraft/`) that adds `file.exportPng`; if GridCraft gains native PNG output, the patch goes away. The repository therefore keeps what was done, what was tested, with which sample files, and the comparative results (time, size, success rate and visual quality) of the original engines against LibreOffice and against alternatives, so that upgrading the pinned versions can be re-evaluated (`docs/UPDATING.md`).

## Building

See `docs/BUILD.md`. In short:

```sh
scripts/build.sh          # fetches the three upstream projects at pinned SHAs, applies the patch, builds in release
```

Requirements: Rust 1.95 or newer (edition 2024), git, and access to GitHub and crates.io. The release profile optimizes for binary size (`opt-level = "z"`, fat LTO, `panic = "abort"`, stripped symbols). The release binary is about 27 MB on Linux (26,992,128 bytes) and 24 MB on Windows (23,995,392 bytes). A clean build takes several minutes. Pinned versions are in `UPSTREAM.md`; size details in `docs/SIZES.md`.

Build output goes to `outputs/` (see `.cargo/config.toml`), which git ignores.

### Tests

```sh
cargo test --locked                # core crate: fast, uses a fake engine, no third-party code needed to run
cargo test --workspace --locked    # also the encoders and the end-to-end CLI tests over the corpus (builds the engines)
bash scripts/check-encoding.sh     # UTF-8 without BOM, LF line endings, no mojibake
```

The workspace needs `third_party/` to resolve, so run `scripts/fetch-upstream.sh` once first (it needs `core.autocrlf=false` on Windows).

## Repository layout

| Path | Contents |
|---|---|
| `crates/core/` | Engine-independent core: the `Render` port, routing, scale choice, crop/trim/fit math, resampling, the post-processing pipeline |
| `crates/thumbcraft/` | The binary: CLI, engine adapters, encoders |
| `scripts/` | Build, upstream fetch, corpus runs, native comparison, encoding check |
| `patches/gridcraft/` | PNG export patch for GridCraft (see `docs/PR-GRIDCRAFT.md`) |
| `third_party/` | Upstream projects at pinned SHAs (generated, not versioned) |
| `corpus/` | Test inputs (docx, xlsx, pptx, pdf) with a `MANIFEST.tsv` each |
| `baselines/`, `reports/` | Results and reports of the 2026-10-19 evaluation |
| `docs/` | Architecture, build, decisions, results, pitfalls, the PDF evaluations, the npm distribution plan; `docs/rules/` holds the working rules |
| `.tasks/` | Task backlog (`00-manifest.md`) |

## Status and roadmap

- Working: the three engines behind one CLI, with trim, crop, resize, fit and PNG, JPEG and WebP output; unit, pipeline and end-to-end tests pass on Windows.
- PDF input: supported (Hayro through `craft-pdf`); memory and size measurements are in `.tasks/08-pdf-engine.md`.
- Planned: npm packages (a TypeScript wrapper plus prebuilt binaries per platform) and a release pipeline. The design is in `docs/DISTRIBUTION_NPM.md`; none of it is implemented yet.
- Supported platforms: Windows x64 and Linux x64 (glibc), both built and tested. macOS, ARM and musl are out of scope.

## Notes on the stored data

The corpus and baselines date from 2026-10-19 (Linux x86_64, 2 shared vCPUs, LibreOffice 24.2.7.2, Rust 1.97.0), over the commits `117aea31` (WordCraft), `66ce6be4` (GridCraft) and `010941ec` (DeckCraft). Timings are noisy. The repository does not store binaries, full-size PNGs, golden PDFs or clones of the craft projects, only hashes, recipes and reduced samples. Sizes are in decimal MB (1 MB = 1,000,000 bytes).

## License

Apache License 2.0 (see `LICENSE`). The craft projects fetched into `third_party/` have their own licenses (MIT or Apache-2.0).
