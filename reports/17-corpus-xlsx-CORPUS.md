# Corpus de teste de thumbnails de planilhas (documento CORPUS.md)

> Arquivo de relatório. Origem: subagente `a81d8604ffdad44fb`, arquivo `grid-png/c-corpus/report/CORPUS.md` (complementa o 09). Sessão de 2026-10-19.
> Tema: Documento de corpus xlsx: composição, linha de base do LibreOffice, como o LibreOffice define a página 1, como rodar a comparação.
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

# Corpus de teste de thumbnails de planilhas

## Conteúdo

Pasta `in/`: 57 arquivos mais `MANIFEST.tsv` (colunas: arquivo, caso, categoria, suportado_gridcraft, nota_thumbnail_esperada).

| Categoria | Arquivos |
|---|---|
| válido (xlsx, 01 a 42) | 42 |
| formato alternativo (xls, ods, csv, tsv, 43 a 48) | 6 |
| inválido (0 bytes, truncado, corrompido, texto puro, extensões trocadas, 49 a 57) | 9 |

Suporte do GridCraft (README e docs/cli.md): xlsx sim; xls e ods parcial (só dados, fórmulas e formatos; sem estilos nem gráficos); csv e tsv sim (Latin-1 marcado como parcial por causa da codificação); inválidos e extensões trocadas marcados como não (a detecção por conteúdo é um comportamento a observar, não um requisito). XLSB não foi incluído porque nenhuma ferramenta local grava XLSB.

As planilhas de várias abas (06, 16, 34) trazem textos como `WRONG`, `OUTRA ABA 2 ZZZ` e `ABA ERRADA 3 QQQ` nas abas 2 e 3, para detectar renderização da aba errada. O caso 40 tem a primeira aba oculta e a segunda ativa.

Os geradores estão em `tools/gen/gen_corpus.py` (XlsxWriter 3.2.9 e openpyxl 3.1.5, usados apenas como instrumento de teste). Os arquivos .xls e .ods foram convertidos do 34 pelo LibreOffice. As imagens de 21 foram geradas com ImageMagick.

## Linha de base do LibreOffice

- Versão: LibreOffice 24.2.7.2 (build 420).
- Comando: `soffice -env:UserInstallation=file:///tmp/lo-profile-baseline --headless --convert-to png --outdir out-lo ARQUIVO`, um processo novo por arquivo, `timeout 60`, `nice`. Script: `tools/run-lo.sh IN OUT`. Resultados em `out-lo/results.tsv` (arquivo, exit, ms, tamanho do PNG em bytes).
- Resultado: 57 execuções, todas com exit 0. 54 geraram PNG, 3 não geraram nada: 50 (truncado), 51 (bytes corrompidos) e 54 (xlsx com extensão .csv). Tempo médio 1653 ms, máximo 2101 ms (perfil já inicializado, máquina de 2 vCPUs compartilhada).
- Todos os PNGs têm 795x1124 px: é uma página A4 retrato, não um intervalo de células.

### Como o LibreOffice define a "página 1"

O filtro de PNG do Calc renderiza a primeira página de impressão da primeira aba visível. Ou seja: respeita a área de impressão, o tamanho de papel A4 retrato, as margens e a quebra de página automática (esquerda para a direita só depois de descer, "para baixo e depois para o lado"). Não é A1:Z60 e a escala é a original da planilha (nenhum ajuste para caber). Verificado com PDF do mesmo arquivo:

- 23 (130 colunas): a primeira página mostra só Col1 a Col10; o documento inteiro tem 13 páginas. Truncado.
- 24 (10 mil linhas): a primeira página mostra cerca de 49 linhas; o documento tem 209 páginas. Truncado.
- 12 (80 linhas, 15 colunas): a primeira página mostra cerca de 10 colunas; 4 páginas no total.
- 29 (área de impressão C5:F12): mostra somente C5:F12, nunca A1.
- 39 (A1 e Z500): a primeira página mostra só A1; Z500 fica em outra página.
- 11: linhas e colunas ocultas não aparecem. 36 (RTL): colunas espelhadas.
- 40 (primeira aba oculta): LibreOffice mostra a aba visível (ABA VISIVEL 2). A "primeira aba" é a primeira visível.
- O zoom da planilha (37) é ignorado.
- Fontes não latinas e emoji (31) dependem das fontes instaladas na máquina.

### Casos estranhos ou de falha

- 49 (0 bytes): exit 0 e PNG em branco (tratado como documento vazio), não é falha.
- 50, 51, 54: exit 0 sem PNG (arquivo recusado em silêncio). 54 é um xlsx válido com extensão .csv, lido como texto/rejeitado.
- 52 e 53 (texto/CSV com extensão .xlsx): abrem como texto, com o conteúdo.
- 55, 56, 57 (formato real diferente da extensão): LibreOffice detecta pelo conteúdo e renderiza normalmente.
- 45 a 48 (csv e tsv): saem com cabeçalho de página e rodapé "Page 1" (impressão de texto); o CSV com ponto e vírgula e o Latin-1 dependem do filtro de importação padrão.
- 04 e 41 (fórmulas sem cache): LibreOffice recalcula ao abrir, o que pode divergir de motores que usam apenas o valor em cache.
- Células vazias: o PNG de 38 é uma página em branco.

## Como rodar a comparação

```
tools/compare.sh DIR_A DIR_B DIR_C OUT_DIR [rótulo_A rótulo_B rótulo_C]
# rótulos padrão: libreoffice pdf-raster refatoracao
BATCH=8 tools/compare.sh ...   # linhas por imagem lado a lado (padrão 7)
```

Requer ImageMagick (`convert`, `montage`, `identify`). Os PNGs nas três pastas devem ter o mesmo nome de base (ex.: `01-texto-acentos.png`). A lista de arquivos é a união dos PNGs e das entradas de `results.tsv` de cada pasta, então um PNG ausente vira tile vermelho "FALHOU". Saídas em OUT_DIR:

- `contact-<rótulo>.png`: contact sheet com 5 colunas e o nome sob cada miniatura.
- `side-by-side-NN.png`: 3 colunas, uma linha por arquivo, rótulo em cada tile.
- `results.tsv`: arquivo, e para cada pasta existe, tamanho em px, ms e exit (lidos de `results.tsv` da pasta, se houver).

Teste de fumaça: `tools/compare.sh out-lo out-lo out-lo report/cmp-test` (cerca de 80 s em 2 vCPUs, 13 arquivos de saída, 9 imagens lado a lado). Para regenerar a linha de base: `tools/run-lo.sh $PWD/in $PWD/out-lo`.

Observação: o tile reduz cada PNG para caber em 360x240, então páginas A4 retrato ficam pequenas; para ver detalhes, abra o PNG original.
