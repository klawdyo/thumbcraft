# GridCraft: protótipo gp-min (Caminho C, PDF e rasterização), papel A3

> Arquivo de relatório. Origem: subagente `a0d9c86c9a242e39e`, turno 1 (de 3). Sessão de 2026-10-19.
> Tema: Primeira versão do protótipo do Caminho C: PDF do GridCraft rasterizado com craft-pdf raster (hayro), A3 com recorte ao conteúdo.
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

# Relatório: protótipo gp-min (Caminho C, PDF e raster)

O protótipo funciona, com um binário só. Os 58 arquivos do corpus passaram por ele em 5,8 s no total.

## Arquitetura
- **Um binário, um processo.** O motor do GridCraft entra como dependência por caminho (`crates/engine`) e o `craft-pdf` com `raster` entra como git. Não foi preciso o plano de dois processos.
- **Fluxo:**
  1. `file.open` abre a planilha.
  2. `pageLayout.pageSetup` configura a página: papel A3, orientação conforme a proporção do intervalo, margens de 0,15", sem cabeçalho, rodapé, títulos repetidos nem cabeçalhos de linha e coluna, e `fitWidth=1`/`fitHeight=1`.
  3. `file.exportPdf` gera o PDF em base64 na memória, com `range` e `fitToPage`.
  4. `craft_pdf::open_slice` abre esse PDF.
  5. `raster::render` rasteriza a página 0 com fundo branco opaco.
  6. O resultado é achatado sobre branco, recortado ao conteúdo (tinta mais 8 px de folga) e gravado como PNG pelo crate `png`.
- **"Página 1":** o intervalo é `--range`, ou o intervalo impresso (área de impressão, ou intervalo usado mais gráficos e imagens) limitado a A1:Z60. Esse intervalo é ajustado para uma única página da aba ativa, e a página 0 do PDF é essa página. Abas não ativas são ignoradas.
- **Opções extras:** `--gridlines` e `--no-crop`. Por padrão não há linhas de grade, o que parece certo para thumbnail.

## Números
- **Código:** 166 linhas em `gp-min/src/main.rs` e 20 em `run-corpus.sh`.
- **Binário:** 11.409.736 bytes (10,88 MiB, 11,4 MB decimais). É release com `opt z`, LTO fat, `codegen-units=1`, `panic=abort` e `strip`. Liga só a libc, libm e libgcc_s.
- **O que o binário carrega:** o motor inteiro (xlsx, fórmulas, funções, gráficos, PDF próprio), o Hayro (interpretação e raster em Rust puro, fontes padrão 14 embutidas) e o `png`.
- **Build:** 7 min 23 s para o `cargo check` frio e 5 min 36 s para o release.
- **Tempo por arquivo:** cerca de 70 a 110 ms nos casos comuns. Gráficos levam 110 a 160 ms, a planilha de 130 colunas 247 ms e a de 10.000 linhas 178 ms. O maior foi 301 ms (`12-paineis-congelados`).
- **Corpus:** 55 de 58 geraram PNG com código 0. Os 3 de código 2 (`49-vazio-0bytes`, `50-xlsx-truncado`, `51-xlsx-bytes-corrompidos`) falharam na abertura, o que é esperado. O resultado está em `S/grid-png/b-pdfraster/out/` (PNGs e `results.tsv`).
- **Formatos:** CSV, TSV, XLS e ODS abriram, inclusive os de extensão trocada.

## Verificação visual
Abri 8 PNGs do corpus do outro agente. Além deles, o `MANIFEST.tsv` do corpus também virou PNG, porque o script processa tudo que está na pasta. Não criei planilhas próprias: usei o corpus para a conferência visual.

- **Funcionou:**
  - acentos em português;
  - cores de fundo, bordas e negrito;
  - mescladas;
  - escala de cor da formatação condicional;
  - gráfico de barras com legenda;
  - imagens PNG e JPEG;
  - planilha larga (Z cabe em 2318 px de largura).
- **Fidelidade:**
  - **Texto não latino e emoji viram `?`.** É o limite do PDF do projeto, que só usa as fontes padrão 14.
  - **Texto cortado na célula.** O motor não deixa texto transbordar sobre vizinha ocupada, então "Centro-Oeste" sai como "Centro-Oe" e há cortes em `01-texto-acentos`.
  - **Fonte substituída.** Fontes ficam mapeadas para Helvetica, Times ou Courier.
  - **Não confirmei** visualmente painéis congelados, sparklines, rotação de texto, RTL, comentários e hiperlinks; só olhei os PNGs listados acima. O `35-comentarios-hiperlinks` deu um PNG de 2.713 bytes, quase vazio, então vale conferir esse.

## Conflitos de dependência
1. **O `c3487320` não serve para raster.** Ele só tem o writer do `craft-pdf` (`read`, `interpret` e `raster` não existem nele). A feature `raster` foi adicionada no commit `c07b3fa` (e `ca50d63`), que fica no `main`. Usei `ae2f796b673f29cf3161f36eb6a8933f0a3fe028`, a ponta do `main`. Em consequência, WordCraft e DeckCraft precisariam subir de revisão para ganhar raster. Não testei se isso quebra o `krilla` deles.
2. **O Hayro com patch vem do craft-libs.** O `craft-pdf` usa versões do Hayro com patch que ficam em `vendor/` no craft-libs, por `[patch.crates-io]`. Esse patch é ignorado em dependências, então o crate raiz precisa repeti-lo (`hayro`, `hayro-interpret`, `hayro-syntax`, `hayro-jbig2`, todos no mesmo rev). Foi o que fiz no `Cargo.toml` do `gp-min`.
3. **O GridCraft fixa o craft-libs em `d27646a5`, e isso não colidiu.** Esse rev só entra por `craft-ui`, que o engine não puxa. O Cargo.lock do GridCraft foi copiado para o `gp-min` e o cargo só adicionou o que faltava.
4. **Edition 2024 e Rust 1.97** foram aceitos sem problema.
5. **O original não foi alterado:** `git status` do GridCraft sai vazio.

## Caminhos
- **Binário:** `S/grid-png/target/release/gp-min`
- **Script:** `S/grid-png/b-pdfraster/run-corpus.sh` (aceita `GP_MIN` para trocar o binário)
- **Fonte:** `S/grid-png/b-pdfraster/gp-min/src/main.rs`

## Para a memória
O `craft-pdf` com `raster` só existe a partir do `main` do craft-libs (commit `c07b3fa` em diante). O `c3487320` é só writer.
