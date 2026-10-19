# WordCraft: display list Draw compartilhada por PDF e PNG

> Arquivo de relatório. Origem: subagente `ac1cbf38ae86decce`, turno 3 (de 3). Sessão de 2026-10-19.
> Tema: Verificação de que o PDF e o PNG do WordCraft partem da mesma lista `Draw`.
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

# O PDF do WordCraft e o PNG consomem o mesmo display list `Draw`

Sim, os dois backends partem da mesma lista `Draw`. Tudo abaixo foi lido no código (`S/repo/crates/`). As contagens de linhas de código são aproximadas, e as comparações de fidelidade são estimativa quando indicado.

## 1. Registro e chamada do `file.exportPdf` (verificado)

- **Registro:** `engine/src/cmd/file.rs:82` registra `file.exportPdf` com um fechamento. O fechamento só chama `crate::io::save_path_with(path, &s.doc, None, &opts)`, sem tocar em layout ou desenho.
- **Despacho por extensão:** `engine/src/io_ext.rs:33-36`, no ramo `"pdf"`, chama `wordcraft_pdf::export(doc, &PdfOptions)`.
- **PNG pela mesma via:** o ramo `"png"` (`io_ext.rs:32`) chama `render_png` (43-48), que faz `layout`, `wordcraft_render::render_page` e `to_png`. Por isso `convert x.docx y.png` e `file.exportPng` compartilham o motor de layout com o PDF.
- **Impressão:** `file.printPdf` (`file.rs:106`) vai para `engine/src/print.rs:162-171`, que chama `wordcraft_pdf::export_layout(&doc, &lay, &opts)` com um layout já pronto.

## 2. De onde o PDF vem (verificado)

- **O PDF é gerado a partir de `Draw`.** O cabeçalho de `pdf/src/lib.rs:3-4` diz que desenha a lista de `page_display`. Em `draws()` (878-896) isso é feito de fato, chamando `page_display(self.doc, &one, dopts)`.
- **O crate é `wordcraft-pdf`** (`crates/pdf/src/lib.rs`, 1757 linhas, mais 1862 de testes). Ele usa `craft-pdf` (com a feature `write`) e `krilla` para o desenho.
- **O conversor `Draw` para PDF é o `Exporter::draw`** (`pdf/src/lib.rs:938-1124+`). Os métodos auxiliares são:
  - `page` (795);
  - `line` (1126);
  - `picture` (1235);
  - `glyphs` (1333).
- **Tamanho:** o `draw` principal tem ~190 linhas. Com os auxiliares, o desenho (sem tags, metadados e fontes) fica em ~700 a 900 linhas. O restante de 1757 trata fontes subsetadas, mapas Unicode, tagging acessível, metadados, links e favoritos.
- **O PNG é `render/src/lib.rs:722-1048`**, ~330 linhas no `draw`, mais `color_glyph.rs` (439) e imagens e vetores.

## 3. Ponto único onde o layout termina (verificado)

- **Layout:** `wordcraft_layout::layout(doc: &Document, cache: &mut LayoutCache, opts: &LayoutOptions) -> DocLayout` (`layout/src/lib.rs:2124`). `DocLayout` (491) contém `pages: Vec<Page>`, e cada `Page` (463) tem `items: Vec<Placed>`, `header` e `footer`.
- **Display list:** `page_display(doc: &Document, page: &Page, opts: &DisplayOptions) -> Vec<Draw>` (`layout/src/display.rs:410`).
- **Os dois backends partem daí:** o PNG chama `page_display` em `render_region_capped` (`render/src/lib.rs:~530`) e o PDF em `Exporter::draws` (`pdf/src/lib.rs:895`). Há ainda cache por item no PDF (monta uma `Page` com um único `Placed` por vez) para ter tags.
- **Observação:** o layout não devolve `Vec<Draw>` direto. A conversão `Placed` em `Draw` (`display.rs`, 1891 linhas) acontece por página, na hora do backend.

## 4. Tratamento das variantes de `Draw` (verificado nos `match`)

A lista tem 14 variantes: `Glyphs`, `Fill`, `Line`, `Image`, `Shape`, `Ink`, `Path`, `Figure`, `Mark`, `Turned`, `Rotated`, `Clip`, `Art` e `MarkText` (`display.rs:25-112`).

| Variante | PDF (`pdf/src/lib.rs`) | PNG (`render/src/lib.rs`) |
|---|---|---|
| `Glyphs` | 945, fonte embutida e subsetada, texto selecionável | 777, contornos preenchidos via `FontDb::outline`, mais glifos coloridos (`color_glyph.rs`) |
| `Fill`, `Line`, `Image`, `Shape`, `Path`, `Ink`, `Art` | 951, 959, 960, 1009, 961, 979, 1042 | 729, 738, 861, 957, 900, 946, 918 |
| `Clip`, `Turned`, `Rotated`, `Figure` | 1093, 1101, 1112, 940 (com tags de acessibilidade) | 1018, 1031, 1038, 724 |
| `Mark`, `MarkText` | ignoradas (`1092`, marcas de edição na tela) | desenhadas (824, 842) |

- **Marcas de edição:** só o PNG desenha `Mark` e `MarkText`. No PDF elas são deliberadamente ignoradas.
- **Opção de exibição:** o PDF usa `DisplayOptions` próprias. O PNG zera `dim_header` em `file.rs:354`.
- **Fidelidade (estimativa, não comparei saídas):**
  - O PDF mantém texto como texto (busca e acessibilidade) e tem fallback para contornos quando o krilla não consegue subsetar a fonte (`export_layout_inner`).
  - O PNG não tem tags nem links.
  - Não verifiquei se o PDF trata glifos coloridos como o PNG.

## 5. Conclusão

- **A arquitetura "layout produz display list, backends consomem" está completa e verificada.** Há duas ressalvas: a lista é produzida por página (`page_display`), e o PDF a recorta por item para marcar tags.
- **Custo de cada backend:**
  - PNG: ~330 linhas de desenho (`draw`), ~2000 no `render` inteiro (com cache de imagens, vetores, glifos coloridos).
  - PDF: ~190 no `draw`, ~700 a 900 com os auxiliares de desenho, 1757 no crate.
  - Display list: 1891 linhas em `display.rs`, mais ~28k de layout.
- **O que o GridCraft precisa copiar é só o conceito:**
  1. Uma representação intermediária neutra de página (enum de poucas operações: preencher retângulo, linha, polígono, texto já posicionado e medido, imagem, clip e save/restore).
  2. Uma função única "planilha e intervalo viram operações por página" que concentre a resolução de estilo, o formato numérico, a quebra de linha, o transbordo e a mesclagem, hoje intercalados em `print.rs`.
  3. Backends que só mapeiam operações: PDF (já existe, via `gridcraft_pdf::Page`) e PNG.
  4. O ponto de partida de tamanho é o do WordCraft: PNG ~330 linhas e conversor PDF ~190, mas o GridCraft precisaria primeiro extrair o item 2 do `draw_block` e do `draw_text` (~930 linhas), o que é a parte cara.
- **A regra do `AGENTS.md` do GridCraft** ("repos don't share code") permite replicar essa forma de trabalho sem copiar código. Reimplementar do zero é mais seguro.

Arquivos principais:
- `S/repo/crates/layout/src/display.rs`
- `S/repo/crates/pdf/src/lib.rs`
- `S/repo/crates/render/src/lib.rs`
- `S/repo/crates/engine/src/io_ext.rs`
