# GridCraft: custo de código para gerar PNG, caminhos A, B, C e E, viabilidade de PR

> Arquivo de relatório. Origem: subagente `ac1cbf38ae86decce`, turno 1 (de 3). Sessão de 2026-10-19.
> Tema: Comparação dos caminhos A (trait de canvas com vello_cpu), B (cena nova), C (PDF e depois rasterização com craft-pdf raster) e E (cena simplificada). Estimativas de dias do relatório foram consideradas exageradas pelo usuário.
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

# Custo de código para exportar uma aba do GridCraft para PNG (thumbnail, sem GUI nem GPU)

Fiz só leitura e não rodei build nenhum. Não consegui medir tamanho de binário, então esses números são estimativas e estão marcados como tal.

## 1. WordCraft: o caminho do `file.exportPng` (verificado no código)

| Peça | Onde | Linhas |
|---|---|---|
| Registro | `crates/engine/src/cmd/file.rs:96` (`CommandSpec::new("file.exportPng", …)`) | 1 |
| Comando | mesmo arquivo, `export_png` (341-359) | ~19 |
| Montagem da cena | `Session::export_layout_with` (`crates/engine/src/session.rs:901`) chama `wordcraft_layout::layout(...)`. A lista `Draw` (Glyphs, Fill, Line, Image, Shape e outras) está em `crates/layout/src/display.rs:25` | display.rs 1891; layout inteiro 28k |
| Rasterização | `crates/render/src/lib.rs`: `render_page` (503), `render_region_capped` (516), `draw()` (722-1048) | lib.rs 2031, mais `color_glyph.rs` 439 |
| PNG | `Rendered::to_png` (lib.rs:~55) usa o crate `image` | ~20 |
| Fontes e texto | `Draw::Glyphs` traz glifo, x e baseline já posicionados. O `draw` pega `FontDb::global().outline(face, gid)` e faz `ctx.fill_path` | `crates/fonts` 4916 (fontdb.rs 1980) |
| CLI | `convert x.docx y.png` e `render` (`apps/wordcraft-cli/src/main.rs:405, 453`) | ~15 |
| MCP | `render_page` em `crates/mcp/src/tools.rs` | ~8 |

- O fluxo é: layout produz o display list, `render` desenha cada `Draw` no `vello_cpu` e o PNG é codificado.
- O Word já tem layout e display list prontos. O GridCraft não tem isso, então o modelo só serve até certo ponto.
- Existe um `apps/wc-min` não rastreado no clone do WordCraft (CLI mínimo de PNG) e o `Cargo.lock` está modificado. Não foi obra minha.
- O `wordcraft-cli` release tem 42,6 MB (medido). Isso inclui tudo, então não serve como custo do rasterizador.

## 2. GridCraft: o que dá para reaproveitar (verificado)

- **Não existe display list nem view model neutro.** O desenho do egui está acoplado à UI: `crates/ui-egui/src/grid.rs` (3671 linhas, `paint_panes` na 778, `look()` na 458 duplicando a lógica do motor).
- **O único desenho sem toolkit é a impressão.** Está em `crates/engine/src/cmd/print.rs` (2167 linhas), e as linhas 603-1530 (~930) são desenho. Ele cobre:
  - paginação: `print_range` (152), `layout_sheet` (230), faixas de linhas e colunas (`bands`, 191);
  - fundos, bordas, mesclagem, quebra de linha, transbordo, formato numérico;
  - formatação condicional (`cell_look`, 634, mais `crates/engine/src/cf.rs`);
  - gráficos: o motor emite `Prim`, o `draw_prims` os converte em chamadas de página, e o texto usa `PdfMeasure`;
  - imagens, formas, sparklines, cabeçalhos de linha e coluna, e cabeçalho/rodapé.
- **O destino do desenho é o `gridcraft_pdf::Page`.** Só 8 funções recebem `&mut Page`. As chamadas usam 13 métodos distintos: `save`, `restore`, `clip_rect`, `fill_rect`, `rect_stroke`, `stroke_line`, `stroke_line_dashed`, `polyline`, `polygon`, `circle`, `text`, `text_rotated` e `image_rgba`. Dá para extrair uma trait pequena com isso.
- **Texto no PDF:** só as 14 fontes padrão, com métricas em `crates/pdf/src/metrics.rs`. O `docs/gaps.md` #5 reconhece que não há PDF não latino.
- **`gridcraft-chart::rasterize`** (`crates/chart/src/raster.rs`, 264 linhas, Rust puro, sem dependências) desenha retângulos, linhas, polígonos, setores e círculos com antialiasing. Ele **não desenha texto, não tem clip e não tem imagem** (o cabeçalho do arquivo diz isso). Só serve para gráficos ou para um thumbnail "greeked".
- **Qual é a "página 1":** o `print_range` usa a área de impressão, senão o intervalo usado mais os objetos. `fitToPage` e `range` já são parâmetros do `file.exportPdf`. Para um thumbnail, o natural é `range` limitado (por exemplo A1:Z60) com `fitToPage`, e desligar cabeçalhos e rodapés.

## 3. `craft-libs` (verificado, clonei em `…/png-cost/craft-libs`)

- **Não há crate de scene nem backend `vello_cpu` compartilhado.** Os crates são: ai-core, atomic, cms, docpath, i18n, pdf, print, quit, settings e ui.
- **O único item de renderização é o `craft-pdf`** (features `read`, `interpret`, `raster`, `write`). O `raster` usa hayro, que embute as fontes padrão 14 Foxit (BSD-3, já listadas no NOTICE e no ATTRIBUTION do craft-libs).
- **A API é `raster::render(page, cache, settings, Options{scale, size, offset, background})`** e devolve RGBA premultiplicado.
- **O render do WordCraft é local** (`wordcraft-render`, com `vello_cpu` 0.2). Cada app tem o seu.
- **Revisões pinadas:**
  - GridCraft: só `d27646a5` (craft-ui e mais um crate).
  - WordCraft: `d27646a5` (craft-ui) e `c3487320` (craft-pdf e krilla).
  - O `d27646a5` é ancestral do `c3487320`, com 56 commits de diferença (12 a 13 de outubro de 2026).
  - No `d27646a5` o crate `pdf` não existe. Para usar o `craft-pdf`, o GridCraft precisa de uma segunda revisão (como o WordCraft já faz) ou de subir o craft-ui.
- **Versões de `vello_cpu`:** o GridCraft já tem 0.1.0 no lock, via epaint 0.36.2 (GUI). O WordCraft usa 0.1.0 e 0.2.0. O Caminho A com `vello_cpu` 0.2 duplicaria a dependência; com 0.1 a API pode diferir (não verifiquei).
- **O `AGENTS.md` do GridCraft diz "Never code: repos don't share code".** Copiar o `wordcraft-render` não é aceitável. O compartilhamento é pelo `craft-libs`.

## 4. Caminhos

| | A: trait `Canvas` + backend vello_cpu | B: cena nova do modelo | C: PDF então `craft-pdf::raster` | E: trait `Canvas` + `rasterize` (texto como barras) |
|---|---|---|---|---|
| Arquivos | `pdf` (trait), `print.rs` (assinaturas e ~48 chamadas), crate novo de raster, `file.rs`/`print.rs` (comando), CLI, catalog, docs, testes | `print.rs` duplicado, mais o resto de A | `engine/Cargo.toml`, `cmd/print.rs` (comando), CLI (~10 linhas), catalog, docs, testes | `pdf` ou `print.rs` (trait), módulo novo, CLI, catalog, docs, testes |
| Linhas novas | 600-900 (base: WordCraft `draw()` ~330 e glifos/fontes, mais ~80 de trait) | 1200-1800 (reescreve ~930 de `print.rs`) | 80-150 de produção, mais ~80 de teste e docs | 250-400 mais ~100 de teste e docs |
| Dependências novas | vello_cpu, kurbo, peniko, skrifa e leitura de fontes de sistema | as de A | `craft-pdf` (hayro, hayro-interpret, hayro-syntax, vello_cpu), bytemuck | nenhuma |
| Binário (estimativa, não medido) | +1,5 a 3 MB, e fontes de sistema | igual a A | +2 a 4 MB (inclui fontes Foxit e cmaps embutidos) | ~0 |
| Texto | fontes de sistema (frágil em container) | idem | fontes padrão embutidas, igual ao PDF | só barras cinza |
| Riscos | casar métricas Helvetica com a fonte real; manter dois backends sincronizados | divergir da impressão e do PDF | segunda revisão do craft-libs; wasm (`cargo xtask wasm` exige que o engine L5 compile em wasm32); recorte da área útil e escala (DPI) | clip precisa ser feito à mão; sem imagem nem rotação fiel |
| Esforço | 6 a 9 dias | 10 a 15 dias | 1 a 2 dias | 3 a 4 dias |

- **Gráficos (D):** reaproveitar o `rasterize` só vale dentro de E. Sozinho ele não cobre a aba.
- **Riscos que valem para todos:**
  - Mesclagem, formatação condicional, imagens e gráficos já são tratados em `print.rs`, então herdam os mesmos bugs e limites do PDF.
  - Texto não latino não funciona, por limite do PDF.
  - Planilhas largas ou compridas: sem `range` o `bands` pagina e a "página 1" é só um pedaço. O thumbnail precisa de `range` e `fitToPage` por padrão.
  - O PDF tem papel e margem. Convém ajustar o Page Setup para o thumbnail ou recortar o bitmap ao conteúdo.

## 5. Recomendação

Recomendo o **Caminho C**. É o menor diff, reaproveita 100% do desenho validado do PDF e usa o código compartilhado do `craft-libs`, que o `AGENTS.md` do projeto prefere. As fontes embutidas dão texto legível sem fonte de sistema.

Se o limite for peso de binário e não fidelidade, use o **E**. O A só compensa se o objetivo evoluir para um renderizador de verdade, com texto não latino.

**Protótipo para validar rápido (~25 linhas, não executado):**
1. Crie um crate fora do repo com `gridcraft-engine` por caminho, `craft-pdf` na revisão `c3487320…` com `features=["raster"]`, e `image`.
2. Abra o xlsx e rode `file.exportPdf` com `{"range":"A1:H30","fitToPage":true}`. Sem `path` ele devolve base64.
3. Faça `craft_pdf::open_slice`, pegue a página 0 e chame `raster::render` com `scale` ≈ 96/72 e fundo branco.
4. Despremultiplique e salve o PNG com `image`.

Isso responde em meia hora de código se o texto aparece e qual é o tamanho do binário.

**Custo de identificar:** 3 a 5 horas para quem parte do zero, já descontado o mapa acima. 1 a 2 horas para confirmar com o protótipo.

**Custo de implementar:** 1 a 2 dias para C, mais 0,5 a 1 dia de acabamento e do PR.

## Viabilidade de PR

**O projeto aceita contribuições externas? Sim (evidência forte, sem política escrita).**
- Há 40 PRs abertos e 93 fechados, de muitos autores diferentes (diegozpy, Zarvinx, Dejan-Port, gyurix e outros), e 70 issues abertas.
- O `AGENTS.md` traz instruções para "agents working for a contributor" que preparam PRs.
- A licença é MIT OR Apache-2.0.
- Não existem `CONTRIBUTING`, `CODE_OF_CONDUCT`, template de PR ou de issue, nem CLA ou DCO. O README só aponta para o `AGENTS.md`.
- Não consegui buscar issues por "png" (a busca do GitHub foi bloqueada por robots). Nas 25 PRs e 25 issues visíveis, nada é sobre PNG ou thumbnail. O `ROADMAP.md`, o `gaps.md` e o `cli.md` não mencionam PNG como planejado.
- O WordCraft tem a mesma estrutura, mas não analisei contribuição lá porque o PNG já existe.

**PR mínimo (Caminho C):**
- `crates/engine/Cargo.toml` e `Cargo.lock`: dependência do `craft-pdf` com `raster`, e `workspace.dependencies` no `Cargo.toml` raiz.
- `crates/engine/src/cmd/print.rs`: comando `file.exportPng` (params `path`, `range`, `scale`, `fitToPage`), na forma do `export_png` do Word.
- `apps/gridcraft-cli/src/main.rs`: extensão `.png` no `convert` (mensagem de erro na linha 275 e uso na 39).
- `crates/engine/src/catalog.rs`: linha `file.exportPng`, depois `cargo xtask parity` para regenerar `docs/parity-checklist.md`.
- `docs/cli.md` (linha 26).
- `crates/engine/tests/command_sequence_fuzz.rs`: adicionar `file.exportPng` à lista de exceções (linha ~59).
- Testes em `crates/engine/src/tests.rs`, perto da linha 2787 (os do `exportPdf`), montando a planilha em código.
- NOTICE e ATTRIBUTION se o `cargo xtask assets` cobrar as fontes Foxit.

**O que os mantenedores provavelmente pedirão:**
- `cargo xtask ci` verde: fmt com largura 150, clippy `-D warnings`, assets, layers e wasm.
- Sem `unwrap`, `expect` nem `panic` fora de testes.
- Teste de regressão que sobreviva a `every_command_survives_empty_params`.
- Limite de tamanho de imagem contra entrada hostil.
- Justificativa do peso da dependência. O `cargo xtask wasm` precisa passar, ou a dependência fica atrás de feature.
- Review automático do CodeRabbit.

**Vale abrir uma issue antes? Sim.**
- Há duas decisões que são dos mantenedores: segunda revisão do `craft-libs` versus subir o craft-ui, e `file.exportPng` no motor versus só no CLI.
- A fila é grande (40 PRs, e o `gaps.md` manda checar PRs abertos antes). Convém confirmar que ninguém está fazendo o mesmo e propor o escopo: "thumbnail via PDF, fontes padrão, sem texto não latino".
- O PR pode ser aberto em seguida com o protótipo anexado.
- O clone local é raso (1 commit), então não confirmei o estilo de commit pelo histórico. O `AGENTS.md` pede id de tarefa no commit, e o último título visível segue o padrão `docs: …(#1860)`.

**Para a memória do usuário (se quiser guardar):** o GridCraft (`storytold/gridcraft`) aceita PRs externos, sem CLA. O `craft-libs` só tem o `craft-pdf` como crate de renderização, e o PNG de planilha no GridCraft seria mais barato via `file.exportPdf` mais `craft-pdf::raster`.

Caminhos principais (todos absolutos, dentro de `S/`):
- `xlsx-test/repo/gridcraft/crates/engine/src/cmd/print.rs`
- `xlsx-test/repo/gridcraft/crates/pdf/src/lib.rs`
- `xlsx-test/repo/gridcraft/crates/chart/src/raster.rs`
- `repo/crates/engine/src/cmd/file.rs`
- `repo/crates/render/src/lib.rs`
- `png-cost/craft-libs/crates/pdf/src/raster.rs`
