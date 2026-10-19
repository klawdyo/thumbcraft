# Word baseline (WordCraft versus LibreOffice)

What it is: results of the thumbnail test (page 1 as PNG) over 30 input files (`../../corpus/docx/`), with the WordCraft engine (minimal binary `wc-min`) and with LibreOffice. A reference for comparing future WordCraft versions and the unified binary.

Date: 2026-10-19.

## Binaries and versions

| Item | Version |
|---|---|
| WordCraft | commit `117aea31a8e61bd3e306e9210800cdc42897e3c1` (2026-10-19), clone from the storytold organization |
| `wc-min` | 15 lines in `prototipo-wc-min/main.rs`, inside the WordCraft workspace (`apps/wc-min`), depends only on `wordcraft-engine`, `wordcraft-doc` and `serde_json` |
| Build profile | `opt-level=z`, fat LTO, `codegen-units=1`, `panic=abort`, `strip=symbols` (`CARGO_PROFILE_RELEASE_*` variables) |
| `wc-min` size | 19,763,984 bytes = 19.76 MB = 18.85 MiB |
| Rust | 1.97.0 |
| LibreOffice | 24.2.7.2, `soffice --headless --convert-to png`, a fresh process per file |
| Machine | 2 vCPUs shared with other agents (see `../../docs/ENVIRONMENT.md`) |

## How it was generated

- Rust: `wc-min INPUT OUTPUT.png 1`, page 1, scale 2 (px per pt), one process per file, 30 s limit. The PNG comes out in Letter (1224x1584) with a transparent background. The thumbnails and samples in this directory were flattened onto white.
- LibreOffice: a fresh `soffice` process per file. All 30 runs finished with exit 0, but only 25 produced a PNG.
- Times measured in seconds by wall clock, with a warm disk cache.

## Contents

| Path | What it is |
|---|---|
| `resultados-wordcraft-wc-min.tsv` | per file: exit, seconds, PNG bytes and error message |
| `resultados-libreoffice.tsv` | per file: exit, seconds, PNG bytes (0 when none was produced) |
| `imagens/contact-sheet-wordcraft-30-arquivos.png` | contact sheet of the 30 cases, failures in red (downscaled to 1600 px wide) |
| `imagens/comparacao-wordcraft-vs-libreoffice-6-casos.png` | side by side for cases 03, 06, 08, 09, 11 and 12 (downscaled to 1600 px) |
| `amostras/` | 7 cases as PNG downscaled to 700 px wide, one per engine (`.wordcraft.png` and `.libreoffice.png`) |
| `prototipo-wc-min/` | source of the minimal binary (`Cargo.toml.txt` is the `Cargo.toml`, renamed so cargo does not read it) |

## Summary of the numbers

Full tables are in `../../docs/RESULTS.md`.

| | WordCraft (`wc-min`) | LibreOffice |
|---|---|---|
| PNG produced | 25 of 30 | 25 of 30 (not the same 25) |
| Mean time | 0.124 s over the successes (0.105 s over the 30) | 1.48 s over the 30 (1.52 s over the 25 with a PNG) |
| Maximum time | 0.245 s | 4.09 s |
| Quality (WordCraft) | 18 ok, 7 degraded, 5 failed | not classified file by file |

WordCraft's 5 failures are invalid inputs, with exit 1 and a clean error message. LibreOffice returns exit 0 even without producing a PNG (13, 23, 24, 26 and 29) and produces a PNG for invalid inputs (for example plain text and binary garbage).

## How to repeat

See `../../docs/TESTS.md` (section "Thumbnails Word") and `../../docs/UPDATING.md`.
