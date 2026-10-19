# PowerPoint baseline (DeckCraft versus LibreOffice)

What it is: results of the thumbnail test (slide 1 as PNG) over 46 input files (`../../corpus/pptx/`), with the DeckCraft engine (minimal binary `dc-min`) and with LibreOffice. A reference for comparing future DeckCraft versions and the unified binary.

Date: 2026-10-19.

## Binaries and versions

| Item | Version |
|---|---|
| DeckCraft | commit `010941ec60f81f1eece31ec1ea83825c9a396217` (2026-10-18), version 0.8.0 (alpha) |
| `dc-min` | `prototipo-dc-min/main.rs`, depends only on `deckcraft-pptx`, `deckcraft-format`, `deckcraft-render` and `deckcraft-fonts` |
| Build profile | `opt-level=z`, fat LTO, `codegen-units=1`, `panic=abort`, `strip=symbols` |
| `dc-min` size | 6,673,360 bytes = 6.67 MB = 6.36 MiB |
| Other sizes | repository default CLI 35,296,984 bytes (35.30 MB, 33.66 MiB); CLI with the size profile 14,504,104 bytes (14.50 MB, 13.83 MiB) |
| Rust | 1.97.0 |
| LibreOffice | 24.2.7.2, `soffice --headless --convert-to png` |
| Fonts | from the system (Carlito and Caladea as metric equivalents of Calibri and Cambria); the external `craft-fonts` was not used |

## How it was generated

- Rust: `dc-min INPUT OUTPUT.png 1` (scale 1, slide 0), one process per file, 30 s timeout. The PNG is 960x540 px. The script is `ferramentas/run.sh`, which runs both engines in the same loop (a dedicated LibreOffice profile and a `pkill` of the process at the end of each file).
- LibreOffice: a 1280x720 px PNG, a fresh process per file, 30 s timeout.
- Warm disk cache, 2 vCPUs shared with another agent that was compiling (inflated build times).

## Contents

| Path | What it is |
|---|---|
| `resultados-deckcraft-vs-libreoffice.tsv` | one line per file, with exit, ms, PNG bytes and class (ok, degraded, failed) for both engines, and notes |
| `resultados-brutos.tsv` | two lines per file (one per engine), with the truncated stderr; the session's absolute paths still appear in the stderr column |
| `imagens/contact-sheet-deckcraft-46-arquivos.png` | contact sheet of the 46 cases (downscaled to 1600 px) |
| `imagens/comparacao-deckcraft-vs-libreoffice-12-casos.png` | 12 cases side by side (downscaled to 1600 px) |
| `amostras/` | 7 cases downscaled to 800 px wide, one per engine |
| `prototipo-dc-min/` | source of the minimal binary |
| `ferramentas/` | `run.sh` and the build scripts (`build-default.sh`, `build-opt.sh`) |
| `logs-de-build/` | logs of the three builds (default CLI, optimized CLI and minimal) |

The folder `../../corpus/pptx/mutacao-48/` holds the robustness test with 48 mutated variants, with its results.

## Summary of the numbers

Full tables are in `../../docs/RESULTS.md`.

| | DeckCraft (`dc-min`) | LibreOffice |
|---|---|---|
| PNG produced | 38 of 46 | 45 of 46 |
| ok, degraded, failed | 34, 4, 8 | 40, 3, 3 |
| Mean time over the 46 | 83 ms (98 ms over the 38 successes only) | 1792 ms |
| Maximum time | 970 ms (file with Arabic, CJK and emoji) | 2977 ms |

Of DeckCraft's 8 failures, 5 are formats it does not read (.ppt and .odp) and 3 are invalid inputs rejected with exit 1. LibreOffice returns exit 0 with a blank PNG or one with tiny text for the 0-byte file and the plain text (a false success).

## How to repeat

See `../../docs/TESTS.md` (section "Thumbnails PowerPoint") and `../../docs/UPDATING.md`.
