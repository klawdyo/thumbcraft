# Anexos da análise do binário combinado (all-min)

Material de apoio do relatório `../../16-binario-combinado-relatorio-detalhado.md` (e do 05). Montado em 2026-10-19 por um subagente que compilou um binário experimental `all-min` com três features (`word`, `ppt`, `xlsx`) para medir o quanto a unificação economiza.

| Arquivo | O que é |
|---|---|
| `main.rs` | fonte do `all-min` (escolhe o motor pela extensão: .doc e .docx pelo WordCraft, .pptx pelo DeckCraft, .xlsx só exporta PDF pelo GridCraft, que na época não tinha PNG) |
| `Cargo.toml.txt` | o `Cargo.toml` do `all-min` (renomeado para o cargo não o ler); os caminhos de dependência apontam para clones da sessão, que não existem mais |
| `build-all.sh` | script que compilou as cinco combinações de features com `opt-level=z`, LTO fat, `codegen-units=1`, `panic=abort` e `strip=symbols` |
| `progress.txt` | resultado dos cinco builds (código, segundos, bytes) |
| `ext-word.txt`, `ext-ppt.txt`, `ext-xlsx.txt`, `ext-all.txt` | crates externos de cada fecho de dependências (`cargo tree`) |
| `closure-*.txt` | fecho de dependências completo de cada conjunto |
| `common3.txt` | pacotes externos comuns aos três locks completos |

Os binários compilados (`bin/`) não foram copiados. Os tamanhos medidos estão no relatório 16: combinado Word, Excel e PowerPoint com 26.600.832 bytes (26,60 MB, 25,37 MiB), contra a soma de 32.865.688 bytes (32,87 MB, 31,34 MiB) dos três mínimos.
