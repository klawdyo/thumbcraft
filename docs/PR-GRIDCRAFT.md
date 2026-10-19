# PR de exportação PNG no GridCraft

Conteúdo, verificações e pendências do patch que dá ao GridCraft a exportação PNG. O patch está em `../patches/gridcraft/0001-exportador-png.patch` (mantido pelo construtor) e é aplicado por `../scripts/fetch-upstream.sh` sobre o GridCraft `66ce6be4743f4227047f76f56815520f824e6d81`. Se o upstream passar a gerar PNG nativamente, o patch sai (ver `UPDATING.md`). Relatórios de origem: `../reports/13`, `14`, `15`, `06` e `07`.

## Resumo

- 32 arquivos, +1043 e -33 linhas. Sem fontes, lock e docs gerados são 16 arquivos com +678 e -24.
- Dois passos: a refatoração do `print.rs` (fase 1, provada sem regressão) e o exportador PNG (fase 2).
- Escrito no padrão dos *craft e não sob medida para o binário unificado. Os padrões de thumbnail (A4, escala 2, crop) ficam fora do PR, no `gc-min` (`../baselines/excel/prototipos/gc-min/`).

## O que muda

| Arquivo | Mudança |
|---|---|
| `crates/pdf/src/display.rs` (novo) | +106: `Op`, `Rec` e `replay` (display list de 13 operações: `Save`, `Restore`, `ClipRect`, `FillRect`, `RectStroke`, `StrokeLine`, `StrokeLineDashed`, `Polyline`, `Polygon`, `Circle`, `Text`, `TextRotated`, `ImageRgba`) |
| `crates/pdf/src/lib.rs` | +3, exporta o módulo; o `Page` não foi alterado |
| `crates/raster/src/lib.rs` (novo) | +314, backend `vello_cpu` |
| `crates/raster/src/text.rs` (novo) | +88 |
| `crates/raster/Cargo.toml` (novo) | +17 |
| `crates/raster/fonts/` (novo) | 12 TTF (Arimo, Tinos, Cousine) e 3 textos OFL |
| `crates/engine/src/cmd/print.rs` | +109 e -14 líquido: 8 funções trocaram `&mut Page` por `&mut Rec`; `render_pdf` monta uma lista por página e a reproduz em `doc.add_page`; novo comando `file.exportPng` |
| `crates/engine/src/catalog.rs`, `Cargo.toml`, `tests/command_sequence_fuzz.rs` | +1 cada |
| `apps/gridcraft-cli/src/main.rs` | +31: `.png` no `convert` |
| `docs/cli.md`, `docs/parity-checklist.md` (gerado por `cargo xtask parity`), `xtask/src/layers.rs` | pequenos |
| `Cargo.toml`, `Cargo.lock`, `ATTRIBUTION.md`, `NOTICE` | dependências e licenças |

Fase 1, sozinha: só `print.rs`, +118 e -14 (`../baselines/gridcraft-pdf-regression/fase1-print-rs.patch`). O `render_pdf` manteve a assinatura pública, e a pilha de estado, o registro de fontes e as imagens do `Page` se reconstroem no replay.

## `file.exportPng`

- Espelha o `file.exportPdf`. Parâmetros: `path`, `page` (1-based, contado em todas as folhas, como a impressão), `sheets`, `range`, `fitToPage`, `scale` (px por pt; padrão 1; até 16). Usa o page setup da própria planilha. Sem `path`, devolve base64. Fundo branco opaco.
- CLI: `convert in.xlsx out.png [--range] [--page] [--scale] [--fit]`.
- Limites: 65535 px por lado e 64 milhões de pixels. Texto não latino sai como `?` (mesma limitação do PDF: medição e layout em WinAnsi, fontes padrão 14 por AFM). Sem recorte, sem JPEG, sem emoji, sem RTL real e sem fontes não padrão.
- Teste novo: gera PNG válido, confere a assinatura e as dimensões A4 (1191x1684 a escala 2), os erros e que o PDF não muda.

## Dependências e licenças

- Crate novo `gridcraft-raster`, camada 0, depende de `gridcraft-pdf`, `vello_cpu` 0.2 (sem a feature `text`) e `skrifa` 0.47. No lock entram `vello_cpu` e `vello_common` 0.2.0, `skrifa` 0.47.0, `read-fonts` 0.44.0 e `glifo` 0.3.0. As versões do egui (0.1.0 e 0.44) continuam e a GUI não foi tocada.
- Não usa `craft-libs`, então não exige segunda revisão nem subir o `craft-ui` (era a objeção do Caminho C).
- Fontes: 12 TTF, 239 KB (238.924 bytes), subconjunto Latin/WinAnsi sem hinting, SIL OFL 1.1, com textos ao lado e linhas em `ATTRIBUTION.md` e `NOTICE`. Baixadas do pacote npm `@expo-google-fonts/{arimo,tinos,cousine}`, porque o GitHub do `craft-fonts` deu 403.

## Verificações feitas

| Verificação | Resultado |
|---|---|
| Regressão do PDF contra o código original | 325 de 325 PASS, 306 de 306 PDFs idênticos byte a byte, 0 pixels diferentes (rodada 3 vezes na fase 2: depois de mover os tipos, depois de ligar `file.exportPng`, no final) |
| `cargo test` de `gridcraft-pdf`, `gridcraft-engine` e `gridcraft-raster` | 347 passaram, 0 falharam (346 na fase 1) |
| `cargo fmt --check` | limpo |
| `cargo clippy -D warnings` | limpo, exceto UI, app desktop e web (não tocados) |
| `cargo xtask layers` | 17 crates, 0 violações |
| `cargo xtask assets` | 46 de 46 atribuídos |
| `cargo xtask wasm` | ok (alvo `wasm32-unknown-unknown`; incluindo `gridcraft-engine`, `gridcraft-raster` e `gridcraft-web`) |
| `cargo xtask parity` | regenerado, 275 de 291 |
| `cargo xtask ci` completo | não rodado |
| Desempenho do PDF (10 mil linhas) | sem diferença mensurável |

Tamanho: o backend PNG soma 1,95 MB ao binário do GridCraft (1.949.432 bytes). O CLI padrão cresce 2,37 MB (2.373.152 bytes) com o PNG, incluindo fontes.

## Pendências para os mantenedores

1. Fontes embutidas contra a regra do `AGENTS.md` ("fontes ficam em `storytold/craft-fonts`, nunca no repositório"). Alternativa: mover os 12 TTF para o `craft-fonts` e carregá-los por `CRAFT_FONTS_DIR`, como o projeto faz com as outras.
2. Feature para não arrastar o backend PNG para quem usa só o motor: o motor com `file.exportPng` registrado já puxa o backend inteiro, mesmo sem chamá-lo (o `main` do `gc-min` soma só 3.456 bytes).
3. Abrir uma issue antes: duas decisões são dos mantenedores (`file.exportPng` no motor ou só no CLI; fontes embutidas ou no `craft-fonts`), e a fila é grande.
4. Texto não latino sai como `?` (limite do PDF, `docs/gaps.md` #5).
5. Revisão automática do CodeRabbit e `cargo xtask ci` verde com fmt de largura 150, sem `unwrap`, `expect` nem `panic` fora de testes, teste que sobreviva a `every_command_survives_empty_params`, limite de tamanho de imagem contra entrada hostil.
6. O `AGENTS.md` pede id de tarefa no commit (padrão visto: `docs: ...(#1860)`). O clone usado era raso (1 commit).

## Viabilidade de PR no GridCraft

- Aceita contribuições externas: sim, sem política escrita. 40 PRs abertos e 93 fechados, de muitos autores, e 70 issues abertas. O `AGENTS.md` traz instruções para agentes que trabalham para um contribuidor. Licença MIT ou Apache-2.0. Não há `CONTRIBUTING`, `CODE_OF_CONDUCT`, template de PR ou issue, CLA nem DCO.
- Nenhum PR ou issue visível (25 de cada) trata de PNG ou thumbnail. `ROADMAP.md`, `gaps.md` e `cli.md` não mencionam PNG como planejado. A busca por "png" nas issues foi bloqueada.
- Recomendação: abrir a issue antes, propondo o escopo ("exportação PNG da primeira página pelo mesmo desenho do PDF, fontes padrão, sem texto não latino") e anexando o protótipo.
