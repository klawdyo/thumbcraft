# Appendices of the combined binary analysis (all-min)

Supporting material for the report `../../16-binario-combinado-relatorio-detalhado.md` (and for 05). Assembled on 2026-10-19 by a subagent that built an experimental `all-min` binary with three features (`word`, `ppt`, `xlsx`) to measure how much the unification saves.

| File | What it is |
|---|---|
| `main.rs` | source of `all-min` (chooses the engine by extension: .doc and .docx go to WordCraft, .pptx to DeckCraft, .xlsx only exports PDF through GridCraft, which had no PNG at the time) |
| `Cargo.toml.txt` | the `all-min` `Cargo.toml` (renamed so cargo does not read it); the dependency paths point to session clones that no longer exist |
| `build-all.sh` | script that built the five feature combinations with `opt-level=z`, fat LTO, `codegen-units=1`, `panic=abort` and `strip=symbols` |
| `progress.txt` | result of the five builds (code, seconds, bytes) |
| `ext-word.txt`, `ext-ppt.txt`, `ext-xlsx.txt`, `ext-all.txt` | external crates of each dependency closure (`cargo tree`) |
| `closure-*.txt` | full dependency closure of each set |
| `common3.txt` | external packages common to the three full locks |

The compiled binaries (`bin/`) were not copied. The measured sizes are in report 16: combined Word, Excel and PowerPoint at 26,600,832 bytes (26.60 MB, 25.37 MiB), against the sum of 32,865,688 bytes (32.87 MB, 31.34 MiB) of the three minimal binaries.
