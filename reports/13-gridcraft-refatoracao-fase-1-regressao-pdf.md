# GridCraft: refatoração fase 1 (só print.rs) e regressão do PDF, 325 de 325

> Arquivo de relatório. Origem: subagente `ab4ce65aee885f788`, turno 1 (de 3). Sessão de 2026-10-19.
> Tema: Fase 1: refatorar só a geração de PDF e provar zero regressão contra o código original, em três camadas.
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

## Fase 1: veredito de regressão do PDF

**Sem regressão: 325 de 325 casos PASS e 306 de 306 PDFs idênticos byte a byte.**

| Camada | Resultado |
|---|---|
| A, bytes (`cmp`) | 306 de 306 PDFs idênticos |
| B, streams de conteúdo (`qpdf --qdf`) | só roda quando A falha; não precisou rodar |
| C, pixels (`pdftoppm` a 50 dpi + `compare -metric AE`) | 0 pixels diferentes em todas as páginas dos 306 PDFs |
| Texto de saída e código de saída | 325 de 325 iguais, incluindo os 19 casos sem PDF (erros) |

- **Normalização:** nenhuma. O gravador não emite data, `/ID` nem UUID (conferi nos 306 PDFs). A lista de normalização do script fica vazia de propósito, para não esconder nada.
- **Goldens:** gerados pelo binário do código original, antes de qualquer mudança. Origem: commit `66ce6be4743f4227047f76f56815520f824e6d81`, árvore sem alterações. O binário original tem sha256 `8d22cb4dbcef502d6d204c9d91b1a39d6b593e974e00b0ffd99032ab36eb3c3c`.
- **Desvio do combinado:** o original foi compilado no mesmo `CARGO_TARGET_DIR` compartilhado, não em um `target-orig` separado. Como os dois binários têm o mesmo nome, copiei o original para fora do target logo após compilar. O hash acima é dessa cópia.
- **Cobertura:** 16 planilhas minhas (valores, estilos, bordas, mesclagem, formatação condicional, gráficos, imagens, quebra de linha, paginação com quebras manuais, títulos repetidos, cabeçalho e rodapé, escala, A3, paisagem, fit to page, cabeçalhos de linha e coluna, sparklines, texto rotacionado, transbordo) mais as 58 do corpus em `c-corpus/in/`, inválidas e vazias incluídas. Cada uma sai em 5 variantes: `exportPdf` padrão, com `fitToPage`, com `range`, com `sheets:"all"` e `file.print`. Há ainda 5 casos de erro com parâmetros inválidos. O corpus tem uma planilha de 10 mil linhas, uma de 0 bytes e outras truncadas ou corrompidas.
- **Teste do detector:** deslocar o `x` de todo `FillRect` em 1 unidade deu FAIL em 99 casos (226 de 325 PASS). As três camadas acusaram: bytes diferentes, o diff das operações (`50.4 710.727 87.75 27.273 re f`) e 339 pixels diferentes em `s-02_styles.d`, página 1. Depois reverti, reconstruí e voltou a 325 de 325.

## O que mudou

- **Arquivos:** só `crates/engine/src/cmd/print.rs`. `diff -rq` e `git status --short` mostram 1 arquivo alterado e 0 novos na cópia. `crates/pdf`, `Cargo.toml`, `Cargo.lock`, docs, catálogo, CLI e testes não foram tocados.
- **Diff:** +118 e -14 linhas.
- **Dentro do `print.rs`:**
  - `enum Op` com as 13 operações, sem trait.
  - `struct Rec` com os 13 métodos de mesmo nome e assinatura que `Page`.
  - `fn replay(&mut Page, &[Op])`.
  - `fn assemble_pages`, privada. Contém o corpo antigo do `render_pdf` apontando para `Rec`.
  - `render_pdf` agora monta uma lista por página e a reproduz em `doc.add_page`.
- **Assinaturas:** 8 funções trocaram `&mut Page` por `&mut Rec`: `draw_prims`, `draw_border`, `draw_block`, `draw_cell_picture`, `draw_text`, `draw_objects`, `draw_headings`, `draw_header_footer`. As ~50 chamadas `pg.` ficaram idênticas. `render_pdf` manteve a assinatura pública.
- **Estado de `Page` que não coube nas 13 operações:** nenhum. A pilha de estado, o registro de fontes usadas e as imagens são efeitos colaterais dos 13 métodos e se reconstroem no replay. O fechamento de `save` abertos continua sendo feito em `PdfDoc::finish`. Nada privado de `Page` precisou ser exposto.

## Testes e qualidade

- `cargo test -p gridcraft-pdf -p gridcraft-engine`: 346 passaram, 0 falharam, em 19 suítes. Isso inclui os testes de print e PDF e o fuzz de sequência de comandos.
- `cargo fmt --check` e `cargo clippy -p gridcraft-engine -p gridcraft-pdf --all-targets -- -D warnings`: limpos. Não rodei `cargo xtask ci` completo (layers, wasm, assets).

## Desempenho

Na planilha de 10 mil linhas (`file.exportPdf`, PDF de 2,7 MB, perfil dev otimizado), 6 execuções de cada binário deram 0,20 a 0,23 s com a refatoração e 0,20 a 0,35 s no original, sem diferença mensurável. Havia outro build rodando em paralelo, então é ruído. Cerca de 0,10 s disso é só abrir o arquivo. O custo extra é guardar a lista inteira em memória (cópia dos textos e das imagens), não medido. Há a nuance de que o limite de conteúdo por página do PDF só vale no replay.

## Caminhos

- Patch da fase 1: `S/grid-png/a-refactor/changes.patch`
- Script de regressão: `S/grid-png/a-refactor/regress-pdf.sh` (`golden` regenera os goldens com o binário original; `check` recompila a cópia, regenera e compara; sai com código diferente de zero se houver qualquer FAIL)
- Goldens (PDFs, saídas e `SHA256SUMS`): `S/grid-png/a-refactor/golden-pdf/pdf/`
- Binário e origem dos goldens: `S/grid-png/a-refactor/golden-pdf/bin/gridcraft-cli.orig` e `.../golden-pdf/ORIGIN.txt`
- Tabela final por arquivo: `S/grid-png/a-refactor/regress-table-final.txt`
- Tabela do teste do detector: `S/grid-png/a-refactor/regress-table-sanity.txt`
- Cópia modificada: `S/grid-png/a-refactor/gridcraft/`
- Esboço da fase 2 (fora do patch): `S/grid-png/a-refactor/phase2-stash/`. Tem o crate raster não compilado (tiny-skia mais skrifa, 12 fontes Arimo/Tinos/Cousine em subconjunto de 239 KB, SIL OFL 1.1) e um patch com o WIP, incluindo `file.exportPng` e a CLI `.png`.

## Para a fase 2

A fase 2 vai precisar tirar `Op`/`Rec`/`replay` de `print.rs` para um lugar compartilhado. O backend PNG não consegue consumir tipos privados do `print.rs`, então isso exige mexer em outros arquivos.

Parei aqui, como pedido. Nada da fase 2 foi compilado ou incluído no patch.
