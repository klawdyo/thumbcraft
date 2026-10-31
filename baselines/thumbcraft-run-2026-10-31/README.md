# thumbcraft run, 2026-10-31 (engines bumped to the 2026-10-31 heads)

Before/after comparison of the whole corpus (`corpus/`, 4 types, subfolders included) for the update of the
four engines. Nothing in `baselines/thumbcraft-run-2026-10-19*` was touched.

| | Before | After |
|---|---|---|
| thumbcraft | 0.18.0 (`main`, `a83ae98`) | 0.18.0 + engine bump |
| WordCraft | `117aea3` (0.8.0) | `e6a5cd0` (0.13.2) |
| GridCraft | `66ce6be` (0.8.0) | `67e39fa` (0.14.2) |
| DeckCraft | `010941e` (0.8.0) | `a3fbf02` (0.13.1) |
| craft-libs | `ae2f796` | `c39e435` |
| Release binary | 25,669,632 bytes | 27,220,480 bytes (+1,550,848, +6.0%) |

Both binaries were built from source with `cargo build --release --locked -p thumbcraft` (rustc 1.99.1, Windows).
Run with `scripts/run-corpus.sh` (`RECURSIVE=1`, 60 s timeout), three modes per type: `native2`
(`--native-scale 2`, no post-processing; PDF has no native mode), `contain480` (`-W 480 -H 480 --fit contain`) and
`cover120` (`-W 120 -H 120`). Compared pixel by pixel with `scripts/compare-runs.py` (threshold 16/255 per channel).

## Result

- No file went from success to failure, no exit code changed, none was fixed (`NOVO_ERRO`, `CODIGO` and
  `CORRIGIDO` are empty in every set).
- **DeckCraft: identical** (all 86 pptx files byte for byte in `native2`). This includes the `skrifa` 0.47 to 0.46
  alignment done by the patches, so that alignment does not change rendering.
- **PDF (craft-pdf, Hayro): identical** in every mode where it runs (39 of 45 files; the other 6 are invalid
  inputs that fail with code 4 on both sides).
- **WordCraft: 7 of 27 renderable files changed** (`comparacao/docx-native2.md`). The page body sits a few pixels
  higher (tighter vertical spacing), which moved "Seção 10" of `19_md_multipagina.md` onto page 1.
- **GridCraft: 5 of 70 renderable files changed** (`comparacao/xlsx-native2.md`). Banner text of
  `34-painel-composto.xlsx` and `regressao-amostras/10_mixed.xlsx` is clipped at the top, charts are smaller, and
  non-Latin text (`31-texto-nao-latino-emoji.xlsx`, `36-planilha-rtl.xlsx`) is drawn as `?` with wider spacing.
- `pares/` has before | after | difference images (red = pixels that differ) of the most different files.

## Where the differences come from

- **GridCraft:** not from our PNG patch. `pdf-gridcraft-com-vs-sem-patch/resultado.tsv`: for the 68 valid
  spreadsheets, the PDF exported by the patched `gridcraft-cli` is byte for byte identical to the PDF of the
  unpatched upstream commit (the 3 files that fail are the invalid inputs, on both sides). The unpatched upstream PDF
  already clips the banner title, so those are GridCraft 0.14.2 behaviours.
- **WordCraft:** upstream layout change between 0.8.0 and 0.13.2. Our patches there only swap the PDF writer for a stub,
  align `miniz_oxide` and `skrifa`, and do not touch layout. Not isolated by a build without them.

## Layout

```
depois/<type>-<mode>.tsv       results.tsv of the "after" run (file, engine, exit code, ms, bytes, dimensions)
comparacao/<type>-<mode>.tsv   one row per file: class, codes, dimensions, % of different pixels, mean difference
comparacao/<type>-<mode>.md    counts per class and the files that changed
pares/                         before | after | difference
pdf-gridcraft-com-vs-sem-patch/resultado.tsv
```

The "before" run was not stored (it is reproducible from `main` and the old SHAs). The guard that keeps these
differences from drifting again is `crates/thumbcraft/tests/layout.rs` (see `docs/UPDATING.md`).
