# GridCraft: corpus xlsx e linha de base do LibreOffice

> Arquivo de relatório. Origem: subagente `a81d8604ffdad44fb`, turno final (o turno intermediário curto foi ignorado). Sessão de 2026-10-19.
> Tema: Montagem do corpus de 57 arquivos e da linha de base do LibreOffice para planilhas.
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

O corpus tem 57 arquivos, a linha de base do LibreOffice está gerada e o `compare.sh` funciona. Nenhuma implementação do GridCraft foi tocada.

**Corpus (`in/`)**
- 42 válidos (xlsx), 6 de formato alternativo (xls, ods, csv e tsv em UTF-8 e Latin-1) e 9 inválidos (0 bytes, truncado, bytes corrompidos, texto puro com extensão .xlsx e extensões trocadas).
- Não incluí XLSB, porque nenhuma ferramenta local grava esse formato.
- A coluna `suportado_gridcraft` do `MANIFEST.tsv` ficou assim: xlsx e csv/tsv UTF-8 como sim; xls, ods e csv/tsv Latin-1 como parcial; inválidos e extensões trocadas como não.
- As planilhas de várias abas têm textos "WRONG" e "ABA ERRADA" nas abas 2 e 3, para denunciar a aba errada.

**Linha de base do LibreOffice (24.2.7.2)**
- Todas as 57 execuções terminaram com exit 0, e 54 geraram PNG (95%).
- O tempo médio foi 1653 ms e o máximo 2101 ms, com um processo novo por arquivo.
- Todos os PNGs têm 795x1124 px, ou seja, uma página A4 retrato.

**Casos de falha ou estranhos**
- Três arquivos saíram com exit 0 e sem PNG: o xlsx truncado, o de bytes corrompidos e o xlsx com extensão .csv.
- O arquivo de 0 bytes gerou um PNG em branco, tratado como documento vazio.
- Texto puro e CSV com extensão .xlsx abrem como texto.
- xlsx com extensão .xls ou .ods e xls com extensão .xlsx renderizam normalmente, porque o LibreOffice detecta o formato pelo conteúdo.
- Os csv e tsv saem com rodapé "Page 1".
- As fórmulas sem valor em cache são recalculadas ao abrir, o que pode divergir de motores que só leem o cache.

**Como o LibreOffice define a "página 1"**
- É a primeira página de impressão da primeira aba visível, em A4 retrato, na escala original e sem ajuste para caber. Não é A1:Z60.
- Respeita a área de impressão: no caso 29 mostra só C5:F12, nunca A1.
- A planilha de 130 colunas mostra só Col1 a Col10 na primeira página, de 13 no total.
- A de 10 mil linhas mostra cerca de 49 linhas, de 209 páginas no total.
- No caso de conteúdo distante (A1 e Z500), só o A1 aparece.
- A planilha com a primeira aba oculta mostra a segunda aba, a primeira visível.
- Linhas e colunas ocultas não aparecem, e o zoom da planilha é ignorado.
- As contagens de páginas e colunas vêm do PDF gerado pelo mesmo LibreOffice para cada caso, não só do PNG.

**Comparação**
- O `compare.sh` gera uma contact sheet por pasta, imagens lado a lado em 3 colunas (7 linhas por imagem, ajustável com `BATCH`) e um `results.tsv` consolidado. PNG ausente vira tile vermelho "FALHOU".
- Testei com `out-lo/` nas três pastas, e os 3 casos sem PNG apareceram como "FALHOU". A execução levou cerca de 80 s nas 2 vCPUs, com resultado em `report/cmp-test/`.

**Caminhos**
- `S/grid-png/c-corpus/in/` (com `MANIFEST.tsv`)
- `S/grid-png/c-corpus/out-lo/` (com `results.tsv`)
- `S/grid-png/c-corpus/tools/compare.sh`
- `S/grid-png/c-corpus/report/CORPUS.md`
- Auxiliares: `tools/run-lo.sh` regenera a linha de base e `tools/gen/gen_corpus.py` gera o corpus.

Usei Python (openpyxl e XlsxWriter) só como instrumento de geração do corpus, como o enunciado permitia. O `compare.sh` e o `run-lo.sh` são shell com ImageMagick, sem Python.
