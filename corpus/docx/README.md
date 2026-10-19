# Corpus Word (WordCraft e afins)

Conjunto de 30 arquivos de entrada usado para testar a geração de thumbnail (página 1 em PNG) com o motor do WordCraft e comparar com o LibreOffice. Montado em 2026-10-19.

- Tamanho total: 836 KB (30 arquivos, mais `fontes-geracao/`).
- Índice: `MANIFEST.tsv` (arquivo, caso, categoria, como foi gerado, resultado no WordCraft, se o LibreOffice gerou PNG).
- Resultados por arquivo: `../../baselines/word/`.
- Como cada arquivo foi gerado: `../../docs/TESTS.md`, seção "Corpus Word".

## Composição

| Categoria | Arquivos | Faixa |
|---|---|---|
| Válidos em .docx | 8 | 01 a 08 |
| Formatos alternativos (.doc 97, .odt, .rtf, .html, .md, .tex, .txt) | 12 | 09 a 20 |
| Inválidos (vazio, truncado, lixo, extensão trocada) | 10 | 21 a 30 |

## Observações

- Os arquivos 25 a 29 são cópias byte a byte de outros arquivos do corpus com a extensão trocada (ver coluna `geracao` do manifesto).
- Todos os documentos válidos são sintéticos e simples (python-docx, pandoc e LibreOffice). Não há documento real do Word com formas, WordArt, EMF/WMF, campos ou OLE. Isso é uma limitação conhecida do corpus.
- O corpus não foi gerado com scripts commitados: os scripts eram em Python e, por regra do projeto, não são versionados. A pasta `fontes-geracao/` guarda os insumos não Python (Markdown de origem e imagens) para refazer os arquivos com pandoc e LibreOffice.
- A imagem do `14_md_rico.md` referencia `img.png` por caminho relativo. No teste original o arquivo não estava ao lado do .md, então a imagem não apareceu (limitação do teste, não do motor). Para repetir com imagem, copie `fontes-geracao/img.png` para a mesma pasta.
