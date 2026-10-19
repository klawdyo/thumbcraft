# GridCraft: gp-min com A4 inteira e sem zoom, como o LibreOffice

> Arquivo de relatório. Origem: subagente `a0d9c86c9a242e39e`, turno 3 (de 3). Sessão de 2026-10-19.
> Tema: Correção de requisito do usuário: o thumbnail é a primeira página A4 sem zoom, mostrando só o que couber.
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

O `gp-min` agora gera a página A4 inteira, sem zoom, como o LibreOffice. O corpus completo passou: 54 de 57 arquivos geraram PNG, e os 3 de erro continuam sendo os corrompidos (`49`, `50`, `51`). O MANIFEST.tsv ficou de fora.

**O que ele faz agora**
- A planilha segue o próprio Page Setup: orientação, margens, escala em %, área de impressão, cabeçalho, rodapé e títulos.
- Só o papel é forçado para A4, e só `--gridlines` liga as linhas de grade.
- Ele ativa a primeira aba visível, sem `range`, e rasteriza a página 0 do PDF.
- `fitToPage=false` é enviado ao motor para anular qualquer ajuste de "caber em páginas" que a planilha traga, então não há zoom.
- Opções, todas desligadas por padrão: `--range`, `--fit`, `--crop` (folga de 8 px) e `--gridlines`.
- Sem `--crop`, o PNG é a página A4 inteira em fundo branco: 1190x1683 px com `--scale 2` e 595x842 px com `--scale 1`, em todo o corpus.

**Binário:** 11.407.912 bytes (10,88 MiB). Foram 169 linhas de Rust e 1 min 49 s de build release, com as mesmas otimizações de antes.

**Tempos nos 54 arquivos que geraram PNG:**

| | Média | Máximo | PNG médio |
|---|---|---|---|
| `--scale 2` | 136 ms | 478 ms (`24-planilha-comprida-10000-linhas`) | 47 KB |
| `--scale 1` | 45 ms | 433 ms (`24-planilha-comprida-10000-linhas`) | 16 KB |

**Conferência visual contra `c-corpus/out-lo/`:** olhei a planilha larga de 130 colunas, a de 10 mil linhas e a de área de impressão, lado a lado com o baseline do LibreOffice (`compare-nozoom-lo.png`). Não olhei os outros PNGs desta rodada, e a comparação foi só visual, sem contagem de pixels.
- **Planilha larga (130 colunas):** as duas mostram 10 colunas e 30 linhas na primeira página, sem zoom, igual.
- **10 mil linhas:** as duas mostram 48 linhas na primeira página, igual.
- **Área de impressão:** as duas mostram o mesmo bloco de 4 colunas por 8 linhas.
- **Tamanho:** o LibreOffice sai com 795x1124 px e o `gp-min` com 1190x1683 px, porque a escala é outra. A paginação é a mesma.
- **Diferenças visuais:**
  - A fonte do LibreOffice é outra (tem serifa leve), enquanto o `gp-min` usa Helvetica padrão do PDF.
  - Os números decimais do `gp-min` saem arredondados (`493.693`), enquanto o LibreOffice mostra o valor completo (`493.69299`). Não investiguei a causa.
  - Ambos mantêm o resto da página em branco.

Fica o ponto de atenção para o thumbnail de 120x120: sem zoom, o conteúdo útil ocupa só o canto superior esquerdo da página A4. Reduzir 1190x1683 para 120x120 deixa o texto ilegível, então vale considerar `--crop` ou `--fit` para esse uso.

**Caminhos:**
- Corpus sem zoom, escala 2: `S/grid-png/b-pdfraster/out-a4-nozoom/`
- Corpus sem zoom, escala 1: `S/grid-png/b-pdfraster/out-a4-nozoom-s1/`
- Imagem de comparação: `S/grid-png/b-pdfraster/compare-nozoom-lo.png`
- Binário: `S/grid-png/target/release/gp-min`

`out/` e `out-a4/` ficaram intactos, e o GridCraft original não foi tocado.
