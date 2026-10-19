# GridCraft: fase 2, PR de exportação PNG, viabilidade de thumbnails e notas de unificação

> Arquivo de relatório. Origem: subagente `ab4ce65aee885f788`, turno 2 (de 3). Sessão de 2026-10-19.
> Tema: Exportador de PNG com vello_cpu 0.2.0, `file.exportPng`, crate raster, gc-min, corpus, comparação das três implementações.
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

## Relatório da fase 2: PR de exportação PNG, viabilidade de thumbnails e notas de unificação

Os três blocos abaixo estão separados como pedido.

## 1. PR (refatoração e exportador PNG)

**Veredito de regressão do PDF: 325 de 325 PASS e 306 de 306 PDFs idênticos byte a byte aos goldens do código original.** Os 19 casos restantes são erros: o texto de saída e o código de saída são idênticos, e não geram PDF. Rodei a checagem 3 vezes: depois de mover os tipos (antes de qualquer trabalho de PNG), depois de ligar o `file.exportPng`, e no final. O `file.exportPdf` não ganhou nenhum padrão ou parâmetro novo.

**Ajuste de rota.** O backend de raster é `vello_cpu` 0.2.0, como o WordCraft e o DeckCraft. O esboço em `tiny-skia` ficou só em `S/grid-png/a-refactor/phase2-stash/`, fora do PR. Os padrões de thumbnail (A4 forçado, recorte, escala 2) saíram do comando e foram para o `gc-min`. O `gc-min` e o `run-corpus.sh` saíram da árvore do repositório.

**Onde ficaram `Op`, `Rec` e `replay`.** Estão em `crates/pdf/src/display.rs`. O `print.rs` do GridCraft só os usa, e o backend PNG, em outro crate, os consome. O crate `pdf` é camada 0, sem dependências, e já é dono de `Font`, `Rgb` e `Page`. Nada de `Page` precisou ser exposto, e o `Page` não foi alterado.

**Arquivos do PR** (`changes-pr.patch`: 32 arquivos, +1043 e -33 linhas; sem fontes, lock e docs gerados, 16 arquivos com +678 e -24):

| Arquivo | Mudança |
|---|---|
| `crates/pdf/src/display.rs` (novo) | +106 |
| `crates/pdf/src/lib.rs` | +3 |
| `crates/raster/src/lib.rs` (novo) | +314 |
| `crates/raster/src/text.rs` (novo) | +88 |
| `crates/raster/Cargo.toml` (novo) | +17 |
| `crates/raster/fonts/` (novo) | 12 TTF e 3 textos OFL |
| `crates/engine/src/cmd/print.rs` | +109 e -14, líquido |
| `crates/engine/src/catalog.rs`, `Cargo.toml`, `tests/command_sequence_fuzz.rs` | +1 cada |
| `apps/gridcraft-cli/src/main.rs` | +31 |
| `docs/cli.md`, `docs/parity-checklist.md` (gerado por `cargo xtask parity`), `xtask/src/layers.rs` | pequenos |
| `Cargo.toml`, `Cargo.lock`, `ATTRIBUTION.md`, `NOTICE` | dependências e licenças |

- **`file.exportPng`:** espelha o `exportPdf`. Parâmetros: `path`, `page` (1-based, contado em todas as folhas, como a impressão), `sheets`, `range`, `fitToPage`, `scale` (px por pt; padrão 1; vai até 16). Usa o page setup da própria planilha e, sem `path`, devolve base64. Fundo branco opaco.
- **CLI:** `convert in.xlsx out.png [--range] [--page] [--scale] [--fit]`.
- **Teste novo:** gera PNG válido e confere assinatura e dimensões A4 (1191x1684 a escala 2). Também confere os erros e que o PDF não muda.
- **Funções que mudaram de assinatura:** 8 funções de desenho trocaram `&mut Page` por `&mut Rec`. O `render_pdf` manteve a assinatura.

**Qualidade.**
- Testes de `gridcraft-pdf`, `gridcraft-engine` e `gridcraft-raster`: 347 passaram, 0 falharam.
- `fmt --check` limpo. Clippy `-D warnings` limpo em todos os crates exceto UI, app desktop e web, que não toquei.
- `cargo xtask layers`: 17 crates, 0 violações.
- `cargo xtask assets`: 46 de 46 atribuídos.
- `cargo xtask wasm`: ok, incluindo `gridcraft-engine`, `gridcraft-raster` e `gridcraft-web`. Instalei o alvo `wasm32-unknown-unknown`.
- `parity` regenerado: 275 de 291.
- Não rodei `cargo xtask ci` inteiro.

**Dependências novas e licenças.**
- Crate novo `gridcraft-raster`, camada 0, depende só de `gridcraft-pdf`, `vello_cpu` 0.2 (sem a feature `text`) e `skrifa` 0.47.
- No lock entram `vello_cpu` e `vello_common` 0.2.0, `skrifa` 0.47.0, `read-fonts` 0.44.0 e `glifo` 0.3.0. As versões 0.1.0 e 0.44 que o egui já usa continuam, e a GUI não foi tocada.
- Fontes: Arimo, Tinos e Cousine (12 TTF, 239 KB no total, subconjunto Latin/WinAnsi sem hinting). Licença SIL OFL 1.1, com os textos ao lado e linhas em `ATTRIBUTION.md` e `NOTICE`.
- Como baixei as fontes: o shell só alcançou npm, e o GitHub da craft-fonts deu 403. Usei o pacote npm `@expo-google-fonts/{arimo,tinos,cousine}` (MIT e OFL-1.1), que embala as fontes do Google.

**Decisão para o mantenedor.** O `AGENTS.md` do GridCraft diz que fontes ficam em `storytold/craft-fonts` e nunca no repositório. Embuti as fontes porque o PNG precisa delas para casar com as métricas AFM. Para o PR, a alternativa é mover os 12 TTF para o `craft-fonts` e carregá-los via `CRAFT_FONTS_DIR`, como o projeto faz com as demais fontes.

**Limitações do PNG.**
- Texto não latino sai como `?`. É a mesma limitação do PDF, porque a medição e o layout são WinAnsi.
- Sem recorte e sem formato JPEG.
- Máximo de 65535 px por lado e 64 milhões de pixels.
- Não implementei emoji, RTL real nem fontes não padrão.

## 2. Viabilidade (thumbnails)

Tudo está fora do repositório, em `S/grid-png/a-refactor/feasibility/`.

**`gc-min`.** O padrão de thumbnail está só aqui, passado como parâmetros e page setup ao `file.exportPng`: primeira aba visível, A4, orientação, margens, área de impressão, escala e quebras da própria planilha, sem ajuste para caber (a não ser `--fit`), página 1, página inteira sobre branco, `--scale` padrão 2.

**`--crop` não foi implementado.** O comando do PR não tem recorte e o `gc-min` só depende do motor.

**Tamanho e build do `gc-min`** (mesmos parâmetros pedidos, via `[profile.release]` do próprio `Cargo.toml`: `opt-level="z"`, LTO fat, 1 codegen unit, `panic=abort`, strip de símbolos):

| Binário | Bytes | MB |
|---|---|---|
| `gc-min` | 8.388.368 | 8,00 |
| `gridcraft-cli` release padrão, original | 15.783.648 | 15,05 |
| `gridcraft-cli` release padrão, com PNG | 18.156.800 | 17,32 |
| `gridcraft-cli` com PNG, depois de `strip` | 14.530.104 | 13,86 |

[Nota de correção do arquivista: a coluna "MB" desta tabela está em MiB (o próprio agente confirmou isso no relatório 15). Em MB decimais: 8.388.368 bytes = 8,39 MB (8,00 MiB); 15.783.648 = 15,78 MB (15,05 MiB); 18.156.800 = 18,16 MB (17,32 MiB); 14.530.104 = 14,53 MB (13,86 MiB).]

- O CLI cresce 2,37 MB com o PNG, incluindo as fontes.

[Nota de correção do arquivista: este 2,37 MB está em MB decimais (2.373.152 bytes, que são 2,26 MiB), ao contrário da tabela acima, que usa MiB. A diferença entre as duas linhas da tabela lida em MiB seria 2,27, não 2,37.]
- O build do `gc-min` levou 3m33.
- Ele arrasta o motor inteiro (calc, fórmulas, xlsx, chart), que é a maior parte do tamanho, mais `vello_cpu`/`vello_common`, `skrifa`, `png` e as fontes. A UI não entra.
- Não medi quanto cada dependência pesa separadamente.

**Corpus.** 55 arquivos suportados, 52 com código de saída 0, 3 falhas esperadas (`49-vazio-0bytes`, `50-xlsx-truncado`, `51-xlsx-bytes-corrompidos`, erro limpo).

| Escala | Média | Máximo |
|---|---|---|
| 2 (padrão) | 134 ms | 393 ms (planilha de 10 mil linhas) |
| 1 | 45 ms | 191 ms |

No padrão, a saída é de 1191x1684 px por arquivo; a escala 1 gera uma imagem com cerca de um quarto dos pixels.

**Conferência visual** em cerca de 14 PNGs, incluindo bordas, gráficos, texto rotacionado, imagens, mesclagem, formatação condicional, quebra e transbordo, primeira aba oculta e área de impressão. Clip, rotação, tracejados e imagens estão corretos, e não encontrei defeito de desenho a corrigir. O texto não latino aparece como `?`, igual ao PDF.

**Comparação com o protótipo.** Contra o protótipo de rasterização de PDF em A4 sem zoom, o erro de cor (RMSE) ficou entre 0,6% e 1,5% nos 7 pares testados. Os PNGs diferem por 1 px de tamanho (1191 contra 1190). A comparação das três implementações está em `S/grid-png/compare-final/`, gerada com o `compare.sh` original, sem ajuste.

**Desempenho do passo extra (montar e reproduzir) no PDF.** Na planilha de 10 mil linhas, 6 execuções de cada binário deram 0,20 a 0,23 s com a refatoração e 0,20 a 0,35 s no original, sem diferença mensurável. O custo é guardar a lista inteira em memória, e esse não medi.

## 3. Notas para a unificação (só fatos medidos)

- O WordCraft e o DeckCraft pinam as mesmas versões de `vello_cpu` (0.2.0), `kurbo` (0.13.1), `peniko` (0.6.1) e `skrifa` (0.47.0). Usei exatamente essas no GridCraft.
- No GridCraft, o `vello_cpu` 0.2.0 convive com o 0.1.0 do `epaint`. A dependência nova entra só no crate `raster` e vem para o motor por ele, e a GUI continua com `vello_cpu` 0.1.0. Num binário que junte só o motor dos três, o `vello_cpu` 0.2.0 deduplica. Em um binário com a GUI do GridCraft, haveria duas versões.
- O lock do GridCraft não tinha `fearless_simd` 1.x. O WordCraft tem `fearless_simd` 0.4.1 e 1.1.0. O GridCraft e o DeckCraft só têm a 0.4.1.

## Caminhos

- Patch do PR: `S/grid-png/a-refactor/changes-pr.patch`
- Patch da fase 1, intacto: `S/grid-png/a-refactor/changes-phase1.patch`
- Script de regressão do PDF: `S/grid-png/a-refactor/regress-pdf.sh`
- Tabela final da regressão: `S/grid-png/a-refactor/regress-table-final.txt`
- Goldens: `S/grid-png/a-refactor/golden-pdf/`
- `gc-min`: `S/grid-png/a-refactor/feasibility/target/release/gc-min`
- `run-corpus.sh`: `S/grid-png/a-refactor/feasibility/run-corpus.sh`
- PNGs do corpus (escala 2): `S/grid-png/a-refactor/out/`, com `results.tsv`. A escala 1 está em `.../a-refactor/out-s1/`.
- Comparação final: `S/grid-png/compare-final/`
- Cópia modificada do repositório: `S/grid-png/a-refactor/gridcraft/`

Para a memória do usuário: o shell só alcança o npm para baixar as fontes, e o GitHub da craft-fonts deu 403. Nada foi pendurado ou adiado para depois do relatório.
