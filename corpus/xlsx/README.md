# Corpus Excel (GridCraft e afins)

Conjunto de 57 arquivos de entrada, mais `MANIFEST.tsv`, usado para testar a geração de thumbnail (primeira página da primeira aba visível em PNG) e comparar o GridCraft (original e patchado) com o LibreOffice. Montado em 2026-10-19.

- Tamanho total: 984 KB (inclui `regressao-amostras/`, 176 KB).
- Índice: `MANIFEST.tsv` (arquivo, caso, categoria, suportado_gridcraft, nota_thumbnail_esperada). Foi gerado pelo agente que montou o corpus e é a fonte da verdade da coluna `suportado_gridcraft`.
- Relatório de montagem do corpus (como cada caso foi criado, como o LibreOffice define a "página 1"): `../../reports/17-corpus-xlsx-CORPUS.md` e `../../reports/09-corpus-xlsx-e-baseline-libreoffice.md`.
- Resultados: `../../baselines/excel/`.

## Composição

| Categoria | Arquivos | Faixa |
|---|---|---|
| Válidos (.xlsx) | 42 | 01 a 42 |
| Formato alternativo (.xls, .ods, .csv e .tsv em UTF-8 e Latin-1) | 6 | 43 a 48 |
| Inválidos (0 bytes, truncado, corrompido, texto puro, extensões trocadas) | 9 | 49 a 57 |

Não há .xlsb, porque nenhuma ferramenta local grava esse formato.

## Atenção

- Os scripts geradores eram em Python (XlsxWriter 3.2.9 e openpyxl 3.1.5) e não são versionados por regra do projeto. Os arquivos gerados estão aqui, e `../../docs/TESTS.md` descreve as ferramentas e os casos.
- Dois casos dependem de ferramentas externas: o 43 (.xls) e o 44 (.ods) foram convertidos do 34 pelo LibreOffice, e as imagens do 21 foram geradas com ImageMagick.
- A planilha de 10 mil linhas (`24-planilha-comprida-10000-linhas.xlsx`) tem só 303 KB e está incluída. Nada foi omitido por tamanho.
- Os arquivos 50 e 51 derivam do 03: o 50 é o 03 cortado na metade e o 51 é o 03 com bytes sobrescritos. Os 54, 55 e 57 são cópias do 02 com extensão trocada, e o 56 é cópia do 43.

## regressao-amostras/

16 planilhas pequenas (`01_values` a `16_text_misc`) usadas apenas pelo teste de regressão do PDF (`../../baselines/gridcraft-pdf-regression/`), junto com as 48 planilhas .xlsx deste corpus. Foram geradas por um script Python não versionado, cobrindo: valores, estilos, bordas, mesclagem, formatação condicional, gráficos, imagens, quebra de linha, paginação com quebras manuais e títulos repetidos, cabeçalho e rodapé, escala, A3, paisagem, fit to page, cabeçalhos de linha e coluna, sparklines, texto rotacionado e transbordo.
