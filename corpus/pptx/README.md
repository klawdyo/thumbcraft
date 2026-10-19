# Corpus PowerPoint (DeckCraft e afins)

Conjunto de 46 arquivos de entrada usado para testar a geração de thumbnail (slide 1 em PNG) com o motor do DeckCraft e comparar com o LibreOffice. Montado em 2026-10-19.

- Tamanho total: 5,4 MB decimais (46 arquivos de entrada 3,3 MB, mais `mutacao-48/` 2,2 MB).
- Índice: `MANIFEST.tsv` (arquivo, formato, caso, categoria, como foi gerado, resultado no DeckCraft e no LibreOffice).
- Resultados por arquivo: `../../baselines/powerpoint/`.
- Como cada arquivo foi gerado: `../../docs/TESTS.md`, seção "Corpus PowerPoint".

## Composição

| Categoria | Arquivos |
|---|---|
| Válidos em .pptx (01 a 32, incluindo 12b) | 33 |
| Formatos alternativos OOXML (.ppsx e .potx, 33 e 34) | 2 |
| Formatos que o DeckCraft não lê (.ppt e .odp, 35 a 39) | 5 |
| Inválidos (0 bytes, truncado, bytes corrompidos, texto puro, XML quebrado, extensão trocada, 40 a 46) | 7 |

O número 24 não existe: o caso de texto vertical e o de texto rotacionado ficaram juntos no 23.

## Atenção ao interpretar

- Os arquivos .ppsx e .potx (33 e 34) são o 03 com o content type trocado, não arquivos feitos pelo PowerPoint.
- Os .ppt e .odp (35 a 39) foram gerados pelo próprio LibreOffice a partir de pptx. O 35 perdeu formas na reimportação do LibreOffice, então não prova nada sobre .ppt feito pelo PowerPoint 97.
- O SmartArt 12 e 12b foi montado à mão, não veio de um PowerPoint real.
- O corpus não tem nenhum .pptx produzido pelo PowerPoint de verdade.

## mutacao-48/

48 variantes mutadas de 6 pptx válidos (09, 12, 18, 05, 03 e 23), 8 por semente, usadas no teste de robustez do DeckCraft. Os modos, em ordem cíclica pelo índice da mutação k (k mod 4): 0 sobrescreve 5 bytes aleatórios, 1 trunca o arquivo, 2 remove um trecho de 1 a 49 bytes, 3 troca valores `val="` e `cx="` por números absurdos. Em cada mutante um único arquivo XML do pacote (slide, gráfico, diagrama, mestre ou layout) foi alterado. Semente do gerador aleatório: 7. Resultado por arquivo em `mutacao-48/RESULTADOS.tsv`.
