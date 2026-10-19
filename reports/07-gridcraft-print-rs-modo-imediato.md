# GridCraft: o print.rs desenha em modo imediato, sem ponto de troca PDF/PNG

> Arquivo de relatório. Origem: subagente `ac1cbf38ae86decce`, turno 2 (de 3). Sessão de 2026-10-19.
> Tema: Onde fica a fronteira entre layout e desenho no `print.rs`; proposta de trait ou gravador de operações (`Op`/`Rec`/`replay`).
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

# Existe um ponto de troca PDF/PNG no `print.rs`? Não: a impressão é em modo imediato

Respondo só pelo código lido, sem rodar nada. Todas as linhas são de `crates/engine/src/cmd/print.rs`, salvo indicação.

## 1. O que as funções de paginação produzem (verificado)

Só geometria de página. Nenhuma devolve conteúdo resolvido por célula.

- **`print_range` (152-177):** devolve `Option<RangeRef>`, ou seja, a área de impressão ou o intervalo usado somado às âncoras de gráficos, imagens e formas.
- **`bands` (191-217):** devolve `Vec<(u32,u32)>`, pares de linha ou coluna inicial e final por faixa. Usa os callbacks `size` e `avail` e os `breaks` manuais.
- **`layout_sheet` (230-348):** devolve `SheetLayout` (95-115). Os campos são:
  - papel e margens: `page_w/h`, `margins`;
  - escala e área: `scale`, `area`;
  - faixas e páginas: `row_bands`, `col_bands`, `pages: Vec<(usize,usize)>`;
  - títulos repetidos: `title_rows/cols`;
  - cabeçalhos de linha e coluna: `heading_w/h`;
  - `manual_breaks`.
- **Quem mais usa a paginação:** `crates/engine/src/paper.rs:59` e `:256` (visão de papel da UI) chamam `layout_sheet`. Esse ponto é estável e fica fora do desenho.

## 2. Onde ficam a resolução de estilo, o formato numérico, a medição e o transbordo (verificado)

Tudo acontece dentro dos laços de desenho, sem estrutura intermediária.

- **`render_pdf` (1463-1526):** para cada página chama `make_block` (1427), que só calcula as posições `xs` e `ys` das colunas e linhas (geometria). Em seguida chama `draw_block` (882), `draw_headings`, `draw_objects` e `draw_header_footer` passando o `&mut Page`.
- **`Block` (811):** guarda referências ao `Workbook` e à `Sheet`, mais `xs`, `ys`, `k` e `scale`. Não guarda células resolvidas.
- **`draw_block` (882-1002)** faz tudo em sequência, em quatro laços:
  - **Seleção de células:** filtra o que precisa ser visitado, incluindo mesclagens (`merges`, 891-912).
  - **Estilo resolvido:** `cell_look` (634) é chamado em 927-930. Ele resolve estilo de tabela e formatação condicional (`CfCache::look`). O resultado `Look` (628) é uma struct local, que vive só dentro do `draw_block`.
  - **Fundos e barras de dados:** `pg.fill_rect` em 955-967.
  - **Bordas:** `draw_border` (860) em 970-990.
  - **Texto:** `draw_text` (1049) em 992-1000.
- **`draw_text` (1049-1230)** mistura tudo e termina em `pg.text`, `pg.text_rotated` e `pg.stroke_line` (sublinhado e tachado):
  - formato numérico: `crate::display::format` e `value_text_as` (1065, 1079-1092);
  - encolher para caber, `###` e quebra de linha: `wrap_lines` (772);
  - alinhamento, rotação e alinhamento vertical;
  - transbordo para células vizinhas vazias: `grow` em 1148-1172, com `empty_for_overflow` (1030) e `center_across_width` (1035);
  - o corte (`clip_l`, `clip_r`) e a chamada `pg.clip_rect` (1176).
- **Outras funções que desenham direto:** `draw_headings` (1360), `draw_header_footer` (1395), `draw_objects` (1250), `draw_cell_picture` (1004).
- **Conclusão do item:** o cálculo e a emissão são intercalados no mesmo laço. Não existe uma lista de células medidas e posicionadas.

## 3. Medição de texto: acoplada ao PDF (verificado)

- **A medição é concreta, não abstrata.** `draw_text` chama `gridcraft_pdf::text_width(font, size, text)` em 8 pontos (1079, 1092, 1096, 1190 e outros), e `wrap_lines` usa a mesma função.
- **As métricas vêm de `crates/pdf/src/metrics.rs`:** tabelas AFM das fontes padrão. A seleção da fonte é `font_for` (877), que usa `Font::family_for` (`crates/pdf/src/lib.rs:106`) e mapeia o nome do estilo para Helvetica, Times ou Courier.
- **Itálico usa a largura do reto e fora do ASCII há aproximação** (`metrics.rs`, comentário do cabeçalho).
- **Consequência para um PNG:** o layout fica atado a Helvetica, Times e Courier. Um PNG que usasse outra fonte teria texto de largura diferente da medida (corte ou espaço sobrando). A coerência só existe se o PNG também usar métricas de fontes padrão.

## 4. Gráficos: uma representação intermediária real (verificado)

- `gridcraft_chart::render` (`crates/chart/src/render.rs:224`) devolve `Vec<Prim>`. `render_sparkline` faz o mesmo.
- **Tipos de `Prim`** (`crates/chart/src/lib.rs:53`): `Rect` (com preenchimento, contorno e raio), `Line` (polilinha com cor, largura e tracejado), `Polygon`, `Wedge` (setores de pizza e rosca), `Circle` e `Text` (com alinhamento, alinhamento vertical e rotação).
- **`Measure`** (trait em `crates/chart/src/lib.rs`, ~105): o motor passa `PdfMeasure` (684), que repete o acoplamento do item 3.
- **`draw_prims` (710-770):** converte cada `Prim` em chamadas de `Page`. A fronteira entre geração e emissão é limpa para gráficos e sparklines.
- **`rasterize` (`crates/chart/src/raster.rs:198`):** desenha esses mesmos `Prim`, sem texto.

## 5. Existe um ponto de troca limpo? Não. Menor refatoração (estimativa)

Existe um ponto de troca parcial: o `Vec<Prim>` dos gráficos. Para as células, o único ponto de troca é a interface do `Page`, e ela é grande demais para substituir só o último passo.

**Refatoração mínima: trait ou gravador de operações.**
- Os métodos `Page` que o `print.rs` usa são 13: `save`, `restore`, `clip_rect`, `fill_rect`, `rect_stroke`, `stroke_line`, `stroke_line_dashed`, `polyline`, `polygon`, `circle`, `text`, `text_rotated` e `image_rgba`. Há 57 ocorrências de `pg.` em `print.rs`.
- **Passo 1:** definir `enum Op` (13 variantes) e `trait Canvas` com esses 13 métodos (~60 a 80 linhas, em `crates/pdf` ou num módulo pequeno do engine).
- **Passo 2:** `impl Canvas for Page` para o PDF, encaminhando as chamadas, sem mudar a saída do PDF. Mudança nas ~8 funções: `pg: &mut Page` vira `pg: &mut dyn Canvas` ou genérico (~50 linhas tocadas, edição mecânica).
- **Passo 3:** um `Recorder` que acumula `Vec<Op>` por página, e `Op` mapeado para PDF (reprodução) ou PNG (~120 linhas, mais o rasterizador de PNG).
- **Passo 4 (opcional):** evitar o `&mut Page` dentro de `draw_text` e outros. Não é preciso, porque a mudança do passo 2 já cobre.
- **Total estimado:** 250 a 350 linhas de refatoração e adaptador, fora o backend PNG em si.

**Risco de regressão nos testes existentes (estimativa):**
- **Baixo.** A saída do PDF não muda se `impl Canvas for Page` apenas encaminha. As dependências do PDF por teste: `crates/pdf/src/tests.rs` (279 linhas, testam `Page` e `PdfDoc` diretamente), os testes em `engine/src/tests.rs:2781-2795` (procuram `/Subtype /Image` e `(#VALUE!) Tj` no PDF) e os do módulo `print::tests` (1748 em diante, não li o conteúdo). Os de `ui-egui/tests/file_menu.rs` e `native_print.rs` acionam `file.exportPdf`.
- **Atrito pontual:** `Page` tem estado interno (`fonts`, `images`, `depth`). O `Recorder` não precisa disso, mas `text_rotated` e `clip_rect` dependem da pilha de estado (`save/restore`). O reprodutor de PNG precisa respeitar a pilha de clip.
- **Risco de desempenho:** gravar 13 tipos de operação por célula em `Vec<Op>` aloca mais que escrever string. O `Op::Text` guarda `String` por chamada. Em planilhas grandes é perceptível, mas o thumbnail usa um intervalo pequeno.

**Resumo das alternativas:**
- **Caminho C (PDF e depois `craft-pdf::raster`):** continua sendo o mais barato (~100 linhas), porque evita refatorar e mantém o acoplamento ao PDF.
- **Refatoração `Op`/`Canvas`:** só compensa se o PNG precisar de fonte diferente das 14, ou se outros backends (SVG, por exemplo) forem desejados. Custo: 1 a 2 dias a mais que o C, mais o backend.

**O que foi estimado e não verificado:** todas as contagens de linhas novas e de risco, o conteúdo dos testes de `print::tests`, e o comportamento de `Page` com o `Recorder`.

Caminho principal: `S/xlsx-test/repo/gridcraft/crates/engine/src/cmd/print.rs`.
