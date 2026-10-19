# Reports of the 2026-10-19 session

Final reports of the subagents that produced the project's experiments, kept in the logical order of the investigation. Each file carries at the top the agent id, the turn and the topic. The content is faithful to the original, with three changes: a no-dash convention, correction notes in square brackets where a report called a value in MiB "MB", and the session path prefix (`/tmp/claude-0/.../scratchpad`) abbreviated to `S`. The reports themselves are in Portuguese.

Unit convention of this repository: decimal MB (1 MB = 1,000,000 bytes) with MiB in parentheses. Old reports that quote MiB as MB carry a correction note (01, 04 and 14).

## Index

| No. | Report | Topic | Agent (turn) |
|---|---|---|---|
| 01 | [01-wordcraft-binario-minimo.md](01-wordcraft-binario-minimo.md) | Does WordCraft turn a document into PNG through the CLI with only the core? A 19.76 MB minimal binary | `a9c8d6827203c05f5` (1) |
| 02 | [02-wordcraft-teste-30-arquivos.md](02-wordcraft-teste-30-arquivos.md) | Thumbnails with 30 files: 25 of 30, mean 0.12 s against LibreOffice's 1.48 s | `a9c8d6827203c05f5` (2) |
| 03 | [03-gridcraft-descoberta-sem-png-na-cli.md](03-gridcraft-descoberta-sem-png-na-cli.md) | GridCraft (Excel) cannot write PNG through the CLI | `a361871d26ddb3c94` (1) |
| 04 | [04-deckcraft-teste-46-arquivos.md](04-deckcraft-teste-46-arquivos.md) | DeckCraft (PowerPoint), 46 files: 38 of 46, 83 ms against 1792 ms, 6.67 MB mini, mutation test | `a50f015b868bc6891` (final) |
| 05 | [05-analise-codigo-compartilhado-binario-combinado.md](05-analise-codigo-compartilhado-binario-combinado.md) | Shared code and the unified binary: 26.60 MB against 32.87 MB | `ab39820d4829f5884` (1) |
| 06 | [06-gridcraft-custo-de-png-e-viabilidade-de-pr.md](06-gridcraft-custo-de-png-e-viabilidade-de-pr.md) | Cost of PNG in GridCraft, paths A, B, C and E, PR feasibility | `ac1cbf38ae86decce` (1) |
| 07 | [07-gridcraft-print-rs-modo-imediato.md](07-gridcraft-print-rs-modo-imediato.md) | Intermediate PDF step: `print.rs` draws in immediate mode | `ac1cbf38ae86decce` (2) |
| 08 | [08-wordcraft-display-list-draw-pdf-e-png.md](08-wordcraft-display-list-draw-pdf-e-png.md) | Word architecture: the `Draw` display list shared by PDF and PNG | `ac1cbf38ae86decce` (3) |
| 09 | [09-corpus-xlsx-e-baseline-libreoffice.md](09-corpus-xlsx-e-baseline-libreoffice.md) | 57-file xlsx corpus and the LibreOffice baseline | `a81d8604ffdad44fb` (final) |
| 10 | [10-gridcraft-gp-min-caminho-c-a3.md](10-gridcraft-gp-min-caminho-c-a3.md) | `gp-min` prototype (PDF and rasterization), A3 | `a0d9c86c9a242e39e` (1) |
| 11 | [11-gridcraft-gp-min-a4.md](11-gridcraft-gp-min-a4.md) | `gp-min` at A4 (A3 against A4) | `a0d9c86c9a242e39e` (2) |
| 12 | [12-gridcraft-gp-min-a4-sem-zoom.md](12-gridcraft-gp-min-a4-sem-zoom.md) | `gp-min` at A4 with no zoom, like LibreOffice | `a0d9c86c9a242e39e` (3) |
| 13 | [13-gridcraft-refatoracao-fase-1-regressao-pdf.md](13-gridcraft-refatoracao-fase-1-regressao-pdf.md) | Refactor phase 1 (only `print.rs`), PDF regression 325 of 325 | `ab4ce65aee885f788` (1) |
| 14 | [14-gridcraft-refatoracao-fase-2-pr-png.md](14-gridcraft-refatoracao-fase-2-pr-png.md) | Phase 2: PNG exporter with `vello_cpu`, PR, `gc-min`, final comparison | `ab4ce65aee885f788` (2) |
| 15 | [15-gridcraft-gc-min-crop-e-quebra-de-tamanho.md](15-gridcraft-gc-min-crop-e-quebra-de-tamanho.md) | Crop in `gc-min` (+0.019 MB) and size breakdown (engine 6.43 MB, PNG 1.95 MB) | `ab4ce65aee885f788` (3) |
| 16 | [16-binario-combinado-relatorio-detalhado.md](16-binario-combinado-relatorio-detalhado.md) | Detailed report of the combined binary analysis (`RELATORIO.md`) | `ab39820d4829f5884` |
| 17 | [17-corpus-xlsx-CORPUS.md](17-corpus-xlsx-CORPUS.md) | The xlsx corpus document (`CORPUS.md`) | `a81d8604ffdad44fb` |

The short intermediate agent turns ("still running" notices) were ignored. Each agent's report was extracted from the last assistant text of each turn in the session's JSONL file.

## Where the artifacts cited in the reports are today

In the reports, `S` is the session's working directory, which no longer exists.

| Cited as | Is in |
|---|---|
| `S/thumbs-test/in`, `results.tsv`, `lo.tsv`, `contact_sheet.png`, `compare.png` | `../corpus/docx/`, `../baselines/word/` |
| `S/repo/apps/wc-min` | `../baselines/word/prototipo-wc-min/` |
| `S/pptx-test/in`, `results.tsv`, `results-raw.tsv`, `thumbs/*.png` | `../corpus/pptx/`, `../baselines/powerpoint/` |
| `S/pptx-test/fuzz` | `../corpus/pptx/mutacao-48/` |
| `S/pptx-test/dc-min` | `../baselines/powerpoint/prototipo-dc-min/` |
| `S/grid-png/c-corpus/in` | `../corpus/xlsx/` |
| `S/grid-png/c-corpus/out-lo`, `tools` | `../baselines/excel/resultados/libreoffice.tsv`, `../baselines/excel/ferramentas/` |
| `S/grid-png/b-pdfraster` (`gp-min`, `out*`, `compare-*.png`) | `../baselines/excel/` (prototipos, resultados, imagens) |
| `S/grid-png/a-refactor` (`regress-pdf.sh`, tables, `golden-pdf/ORIGIN.txt`, `SHA256SUMS`, `changes-phase1.patch`) | `../baselines/gridcraft-pdf-regression/` |
| `S/grid-png/a-refactor/changes-pr.patch` | `../patches/gridcraft/` (maintained by the builder) |
| `S/grid-png/a-refactor/feasibility` (`gc-min`, `bloat-crates.txt`, `build-variants.log`) | `../baselines/excel/prototipos/gc-min/`, `../baselines/excel/tamanhos/` |
| `S/grid-png/compare-final` | `../baselines/excel/imagens/` and `../baselines/excel/resultados/comparacao-final-tres-colunas.tsv` |
| `S/combined-analysis` (no binaries) | `16-...md` and `anexos/combinado-all-min/` |
| Full-size PNGs, golden PDFs, binaries, clones and `target/` | not kept (large or regenerable; see `../docs/UPDATING.md` and the baseline READMEs) |
| `S/grid-png/a-refactor/phase2-stash` (sketch of the raster crate with `tiny-skia`, discarded) | not kept: it was left out of the PR by decision (see `../docs/DECISIONS.md`) |
