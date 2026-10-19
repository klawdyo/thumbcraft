# Relatórios da sessão de 2026-10-19

Relatórios finais dos subagentes que produziram os experimentos do projeto, guardados na ordem lógica da investigação. Cada arquivo traz no topo o id do agente, o turno e o tema. O conteúdo é fiel ao original, com três alterações: convenção sem travessões, notas de correção entre colchetes onde um relatório chamou de MB um valor em MiB, e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`.

Convenção de unidades deste repositório: MB decimais (1 MB = 1.000.000 bytes) com MiB entre parênteses. Relatórios antigos que citam MiB chamando de MB têm nota de correção (01, 04 e 14).

## Índice

| Nº | Relatório | Tema | Agente (turno) |
|---|---|---|---|
| 01 | [01-wordcraft-binario-minimo.md](01-wordcraft-binario-minimo.md) | WordCraft converte doc em PNG pela CLI só com o core? Binário mínimo de 19,76 MB | `a9c8d6827203c05f5` (1) |
| 02 | [02-wordcraft-teste-30-arquivos.md](02-wordcraft-teste-30-arquivos.md) | Thumbnails com 30 arquivos: 25 de 30, média 0,12 s contra 1,48 s do LibreOffice | `a9c8d6827203c05f5` (2) |
| 03 | [03-gridcraft-descoberta-sem-png-na-cli.md](03-gridcraft-descoberta-sem-png-na-cli.md) | GridCraft (Excel) não gera PNG pela CLI | `a361871d26ddb3c94` (1) |
| 04 | [04-deckcraft-teste-46-arquivos.md](04-deckcraft-teste-46-arquivos.md) | DeckCraft (PowerPoint), 46 arquivos: 38 de 46, 83 ms contra 1792 ms, mini de 6,67 MB, teste de mutação | `a50f015b868bc6891` (final) |
| 05 | [05-analise-codigo-compartilhado-binario-combinado.md](05-analise-codigo-compartilhado-binario-combinado.md) | Código compartilhado e binário unificado: 26,60 MB contra 32,87 MB | `ab39820d4829f5884` (1) |
| 06 | [06-gridcraft-custo-de-png-e-viabilidade-de-pr.md](06-gridcraft-custo-de-png-e-viabilidade-de-pr.md) | Custo de PNG no GridCraft, caminhos A, B, C e E, viabilidade de PR | `ac1cbf38ae86decce` (1) |
| 07 | [07-gridcraft-print-rs-modo-imediato.md](07-gridcraft-print-rs-modo-imediato.md) | Etapa intermediária do PDF: o `print.rs` desenha em modo imediato | `ac1cbf38ae86decce` (2) |
| 08 | [08-wordcraft-display-list-draw-pdf-e-png.md](08-wordcraft-display-list-draw-pdf-e-png.md) | Arquitetura do Word: display list `Draw` compartilhada por PDF e PNG | `ac1cbf38ae86decce` (3) |
| 09 | [09-corpus-xlsx-e-baseline-libreoffice.md](09-corpus-xlsx-e-baseline-libreoffice.md) | Corpus xlsx de 57 arquivos e baseline do LibreOffice | `a81d8604ffdad44fb` (final) |
| 10 | [10-gridcraft-gp-min-caminho-c-a3.md](10-gridcraft-gp-min-caminho-c-a3.md) | Protótipo `gp-min` (PDF e rasterização), A3 | `a0d9c86c9a242e39e` (1) |
| 11 | [11-gridcraft-gp-min-a4.md](11-gridcraft-gp-min-a4.md) | `gp-min` em A4 (A3 contra A4) | `a0d9c86c9a242e39e` (2) |
| 12 | [12-gridcraft-gp-min-a4-sem-zoom.md](12-gridcraft-gp-min-a4-sem-zoom.md) | `gp-min` em A4 sem zoom, como o LibreOffice | `a0d9c86c9a242e39e` (3) |
| 13 | [13-gridcraft-refatoracao-fase-1-regressao-pdf.md](13-gridcraft-refatoracao-fase-1-regressao-pdf.md) | Refatoração fase 1 (só `print.rs`), regressão do PDF 325 de 325 | `ab4ce65aee885f788` (1) |
| 14 | [14-gridcraft-refatoracao-fase-2-pr-png.md](14-gridcraft-refatoracao-fase-2-pr-png.md) | Fase 2: exportador PNG com `vello_cpu`, PR, `gc-min`, comparação final | `ab4ce65aee885f788` (2) |
| 15 | [15-gridcraft-gc-min-crop-e-quebra-de-tamanho.md](15-gridcraft-gc-min-crop-e-quebra-de-tamanho.md) | Crop no `gc-min` (+0,019 MB) e quebra do tamanho (motor 6,43 MB, PNG 1,95 MB) | `ab4ce65aee885f788` (3) |
| 16 | [16-binario-combinado-relatorio-detalhado.md](16-binario-combinado-relatorio-detalhado.md) | Relatório detalhado da análise do binário combinado (`RELATORIO.md`) | `ab39820d4829f5884` |
| 17 | [17-corpus-xlsx-CORPUS.md](17-corpus-xlsx-CORPUS.md) | Documento do corpus xlsx (`CORPUS.md`) | `a81d8604ffdad44fb` |

Os turnos intermediários curtos dos agentes (avisos de "ainda rodando") foram ignorados. O relatório de cada agente foi extraído do último texto do assistente em cada turno do arquivo JSONL da sessão.

## Onde estão hoje os artefatos citados nos relatórios

Nos relatórios, `S` é o diretório de trabalho da sessão, que não existe mais.

| Citado como | Está em |
|---|---|
| `S/thumbs-test/in`, `results.tsv`, `lo.tsv`, `contact_sheet.png`, `compare.png` | `../corpus/docx/`, `../baselines/word/` |
| `S/repo/apps/wc-min` | `../baselines/word/prototipo-wc-min/` |
| `S/pptx-test/in`, `results.tsv`, `results-raw.tsv`, `thumbs/*.png` | `../corpus/pptx/`, `../baselines/powerpoint/` |
| `S/pptx-test/fuzz` | `../corpus/pptx/mutacao-48/` |
| `S/pptx-test/dc-min` | `../baselines/powerpoint/prototipo-dc-min/` |
| `S/grid-png/c-corpus/in` | `../corpus/xlsx/` |
| `S/grid-png/c-corpus/out-lo`, `tools` | `../baselines/excel/resultados/libreoffice.tsv`, `../baselines/excel/ferramentas/` |
| `S/grid-png/b-pdfraster` (`gp-min`, `out*`, `compare-*.png`) | `../baselines/excel/` (prototipos, resultados, imagens) |
| `S/grid-png/a-refactor` (`regress-pdf.sh`, tabelas, `golden-pdf/ORIGIN.txt`, `SHA256SUMS`, `changes-phase1.patch`) | `../baselines/gridcraft-pdf-regression/` |
| `S/grid-png/a-refactor/changes-pr.patch` | `../patches/gridcraft/` (mantido pelo construtor) |
| `S/grid-png/a-refactor/feasibility` (`gc-min`, `bloat-crates.txt`, `build-variants.log`) | `../baselines/excel/prototipos/gc-min/`, `../baselines/excel/tamanhos/` |
| `S/grid-png/compare-final` | `../baselines/excel/imagens/` e `../baselines/excel/resultados/comparacao-final-tres-colunas.tsv` |
| `S/combined-analysis` (sem binários) | `16-...md` e `anexos/combinado-all-min/` |
| PNGs completos, PDFs golden, binários, clones e `target/` | não guardados (grandes ou regeneráveis; ver `../docs/UPDATING.md` e os README dos baselines) |
| `S/grid-png/a-refactor/phase2-stash` (esboço do crate raster com `tiny-skia`, descartado) | não guardado: ficou fora do PR por decisão (ver `../docs/DECISIONS.md`) |
