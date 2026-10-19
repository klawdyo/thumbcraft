# Excel baseline (GridCraft, prototypes and LibreOffice)

What it is: results of the thumbnail test (first A4 page of the first visible sheet, as PNG) over the 57 files in `../../corpus/xlsx/`. The original GridCraft CLI cannot write PNG, so this baseline keeps the LibreOffice reference line and the three implementations built during the session, for future comparison: the `gp-min` prototype (GridCraft's PDF rasterized with `craft-pdf` raster), `gc-min` (GridCraft with the PNG export patch, `vello_cpu` 0.2.0 backend) and the final three-way comparison.

Date: 2026-10-19.

## Binaries and versions

| Item | Version |
|---|---|
| GridCraft | commit `66ce6be4743f4227047f76f56815520f824e6d81` (2026-10-18), version 0.8.0 |
| PR patch (refactor, `file.exportPng`, `gridcraft-raster` crate) | 32 files, +1043 and -33 (kept in `../../patches/gridcraft/`, see `../../docs/PR-GRIDCRAFT.md`) |
| `gp-min` | 169 lines (A4, no zoom), `prototipos/gp-min/main.rs`; `craft-pdf` with the `raster` feature at craft-libs revision `ae2f796b673f29cf3161f36eb6a8933f0a3fe028`, plus the hayro patches repeated in the root `Cargo.toml` |
| `gc-min` | `prototipos/gc-min/main.rs`, depends only on the patched engine, `png` 0.18 for the crop |
| Build profile | `opt-level=z`, fat LTO, `codegen-units=1`, `panic=abort`, `strip=symbols` |
| `gp-min` size | A3: 11,409,736 bytes (11.41 MB, 10.88 MiB); A4 no zoom: 11,407,912 bytes (11.41 MB, 10.88 MiB) |
| `gc-min` size | without crop 8,388,368 bytes (8.39 MB, 8.00 MiB); with crop 8,407,856 bytes (8.41 MB, 8.02 MiB) |
| Rust | 1.97.0 |
| LibreOffice | 24.2.7.2, `soffice --headless --convert-to png`, a fresh process per file, `timeout 60`, `nice` |

## Contents

| Path | What it is |
|---|---|
| `resultados/libreoffice.tsv` | LibreOffice reference line (file, exit, ms, PNG bytes). All exit 0, 54 with a PNG |
| `resultados/gp-min-a3-recortado-ao-conteudo.tsv` | First version of path C: A3, range A1:Z60 fitted to one page, cropped to the content, scale 2. It has 58 lines because it also processed `MANIFEST.tsv` (turned into a text PNG), which does not count |
| `resultados/gp-min-a4-recortado-ao-conteudo.tsv` | A4 instead of A3, still fitted and cropped |
| `resultados/gp-min-a4-sem-zoom-escala2.tsv` | final `gp-min` version: whole A4, no zoom, scale 2 (1190x1683 px) |
| `resultados/gp-min-a4-sem-zoom-escala1.tsv` | same, scale 1 (595x842 px) |
| `resultados/gc-min-escala2.tsv` | `gc-min` (patched GridCraft), scale 2 (1191x1684 px), 55 files (ignores the .tsv files and the manifest) |
| `resultados/gc-min-escala1.tsv` | same, scale 1 |
| `resultados/gc-min-escala2-crop.tsv` | same, scale 2 with `--crop` (cropped to the content plus an 8 px margin) |
| `resultados/comparacao-final-tres-colunas.tsv` | output of `compare.sh` with the three columns (see the warning below) |
| `imagens/` | contact sheets of the three implementations (`contact-sheet-*.png`), A3 versus A4 comparison, A4 no zoom versus LibreOffice comparison, and 9 side-by-side images (`lado-a-lado-NN.png`, 7 cases per image, in alphabetical order, columns: LibreOffice, `gp-min` A4 no zoom, patched `gc-min`). All downscaled to at most 1600 px wide |
| `amostras/` | 5 cases as PNG downscaled to 600 px wide, one per engine (15 files) |
| `prototipos/` | sources of `gp-min` and `gc-min` (`Cargo.toml.txt` is the renamed `Cargo.toml`; the session's absolute paths were replaced by `S`) |
| `ferramentas/` | `compare.sh` (3-column side by side, ImageMagick), `run-lo.sh` (LibreOffice reference line), `run-corpus-gp-min.sh` and `run-corpus-gc-min.sh` |
| `tamanhos/` | binary size breakdown: `variantes-de-build.log` (original, phase 1 and PR), `cargo-bloat-por-crate.txt` and the `build-variants.sh` script |

## Warnings about the data

- In `comparacao-final-tres-colunas.tsv`, the columns `ms_pdf-raster` and `exit_pdf-raster` are swapped (the script read the `gp-min` `results.tsv`, whose order is file, ms, code). Read the `exit_pdf-raster` value as milliseconds and the `ms_pdf-raster` value as the exit code.
- In `resultados/gp-min-*.tsv` the column order is file, ms, code, bytes, message. In `gc-min-*.tsv` it is file, exit, ms, bytes.
- In the `gp-min` files the exit code for failures is 2, and in `gc-min` it is 1 (both with a clean error message).
- The LibreOffice reference line shows an A4 page at its original scale, 795x1124 px. The prototypes use another scale in px per pt, so the dimensions differ even with the same pagination.
- The full-size PNGs (about 10 MB) were not kept, only samples. To regenerate them, use `ferramentas/` with the binaries.

## Summary of the numbers

Full tables are in `../../docs/RESULTS.md`.

| | LibreOffice | `gp-min` A4 no zoom (scale 2) | `gc-min` patch (scale 2) |
|---|---|---|---|
| PNG produced | 54 of 57 | 54 of 57 | 52 of 55 (the 2 .tsv files are not processed) |
| Mean time | 1653 ms | 129 ms over the 57 (136 ms over the 54 successes) | 134 ms over the 55 (141 ms over the 52 successes) |
| Maximum time | 2101 ms | 478 ms (10,000-row sheet) | 393 ms (same) |
| Mean PNG | 34 KB | 48 KB | 48 KB |

## How to repeat

See `../../docs/TESTS.md` (section "Thumbnails Excel") and `../../docs/UPDATING.md`.
