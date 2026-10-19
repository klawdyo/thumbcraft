# Resultados consolidados

Data: 2026-10-19. Ambiente: Ubuntu 24.04.5, 2 vCPUs Xeon 2,80 GHz compartilhadas com outros agentes que compilavam, 8 GB de RAM, rustc 1.97.0, LibreOffice 24.2.7.2 (ver `ENVIRONMENT.md`). Cache de disco quente, um processo por arquivo. Cada configuração foi medida uma vez, então os tempos têm ruído. Tamanhos em MB decimais (1 MB = 1.000.000 bytes) com MiB entre parênteses.

Fontes: `../baselines/*/` (dados brutos) e `../reports/` (relatórios). Metodologia: `TESTS.md`.

## 1. Thumbnail por motor, código original dos *craft contra LibreOffice contra protótipos

"Média (todos)" é sobre todos os arquivos processados, inclusive falhas rápidas. "Média (sucessos)" só sobre os que geraram PNG.

### Word (30 arquivos, página 1)

| Motor | PNG gerado | Média (todos) | Média (sucessos) | Máximo | PNG médio | Tamanho do PNG |
|---|---|---|---|---|---|---|
| WordCraft `wc-min`, código original | 25 de 30 | 0,105 s | 0,124 s | 0,245 s | 275 KB | 1224x1584 (Letter, escala 2, fundo transparente) |
| LibreOffice 24.2.7.2 | 25 de 30 (outros 25) | 1,477 s | 1,518 s | 4,09 s | 112 KB | 817x1057 |

Qualidade (WordCraft): 18 ok, 7 degradados (06 fontes substituídas, 09 tabela do .doc sem bordas, 14 imagem relativa ausente, 15 imagem do tex vira texto, 26 docx como .txt vira texto binário, 28 md como .pdf em texto bruto, 22 txt vazio em branco), 5 falhas (21, 23, 24, 25 e 30, todas entradas inválidas, exit 1 com mensagem limpa). Sem panic nem timeout. O WordCraft é cerca de 12 vezes mais rápido.

### PowerPoint (46 arquivos, slide 1)

| Motor | PNG gerado | Média (todos) | Média (sucessos) | Máximo | ok, degradado, falhou | PNG médio | Tamanho do PNG |
|---|---|---|---|---|---|---|---|
| DeckCraft `dc-min`, código original | 38 de 46 | 83 ms | 98 ms | 970 ms | 34, 4, 8 | 63 KB | 960x540 (escala 1) |
| LibreOffice 24.2.7.2 | 45 de 46 | 1792 ms | 1792 ms | 2977 ms | 40, 3, 3 | 67 KB | 1280x720 |

O DeckCraft é cerca de 20 vezes mais rápido; em 45 dos 46 arquivos levou menos de 220 ms (o pico de 970 ms foi o arquivo com árabe, CJK e emoji, por causa da varredura de fontes do sistema). Das 8 falhas do DeckCraft, 5 são .ppt e .odp (formato não suportado, o LibreOffice abre) e 3 são entradas inválidas rejeitadas com exit 1; o LibreOffice deu exit 0 com PNG em branco ou de texto minúsculo no arquivo de 0 bytes e no texto puro (falso sucesso). Teste de mutação: 48 de 48 com PNG, sem panic (46 idênticos ao original).

### Excel (57 arquivos, primeira página A4 da primeira aba visível)

| Motor | PNG gerado | Média (todos) | Média (sucessos) | Máximo | PNG médio | Tamanho do PNG |
|---|---|---|---|---|---|---|
| GridCraft original | não gera PNG | | | | | |
| LibreOffice 24.2.7.2 | 54 de 57 | 1653 ms | 1653 ms (todos exit 0; 3 sem PNG) | 2101 ms | 34 KB | 795x1124 |
| `gp-min` A3 recortado ao conteúdo, escala 2 (PDF e raster hayro) | 54 de 57 | 91 ms | 95 ms | 301 ms | 50 KB | variável (recorte) |
| `gp-min` A4 recortado ao conteúdo, escala 2 | 54 de 57 | 106 ms | 111 ms | 456 ms | 57 KB | variável (recorte) |
| `gp-min` A4 sem zoom, escala 2 | 54 de 57 | 129 ms | 136 ms | 478 ms | 48 KB | 1190x1683 |
| `gp-min` A4 sem zoom, escala 1 | 54 de 57 | 43 ms | 45 ms | 433 ms | 16 KB | 595x842 |
| `gc-min` (GridCraft com patch, `vello_cpu`), escala 2 | 52 de 55 | 134 ms | 141 ms | 393 ms | 48 KB | 1191x1684 |
| `gc-min`, escala 1 | 52 de 55 | 45 ms | 47 ms | 191 ms | 19 KB | cerca de um quarto dos pixels |
| `gc-min`, escala 2 com `--crop` | 52 de 55 | 164 ms | 173 ms | 431 ms | 33 KB | recortado ao conteúdo mais 8 px |

Notas: as 3 falhas de PNG são sempre os arquivos 49 (0 bytes), 50 (truncado) e 51 (bytes corrompidos), com erro limpo (exit 2 no `gp-min`, exit 1 no `gc-min`). O `gc-min` processa 55 arquivos (ignora os 2 .tsv e o manifesto). O LibreOffice tem 3 arquivos sem PNG diferentes (50, 51 e 54) e gera PNG em branco para o 49. Planilhas grandes mostram só a primeira página (130 colunas mostra 10 colunas, 10 mil linhas mostra cerca de 48 linhas), igual nos dois. Diferenças visuais: o LibreOffice usa fonte com serifa leve contra Helvetica no PDF; texto não latino e emoji viram `?` nos protótipos; texto fica cortado na célula quando a vizinha está ocupada; números decimais saem arredondados nos protótipos.

### Equivalência entre os dois protótipos de Excel

Contra o `gp-min` A4 sem zoom, o `gc-min` ficou com erro de cor (RMSE) de 0,6% a 1,5% nos 7 pares testados, e 1 px a mais de largura e altura.

## 2. Tamanho dos binários

Todos x86_64 Linux, rustc 1.97.0. "Perfil de tamanho" é `opt-level=z`, LTO fat, `codegen-units=1`, `panic=abort`, `strip=symbols`.

| Binário | Perfil | Bytes | MB | MiB |
|---|---|---|---|---|
| `wordcraft-cli` | padrão do repositório (thin LTO, cgu 16) | 42.603.432 | 42,60 | 40,63 |
| `wordcraft-cli` | padrão com strip | 35.087.928 | 35,09 | 33,46 |
| `wordcraft-cli` | tamanho | 19.985.520 | 19,99 | 19,06 |
| `wc-min` (mini Word) | tamanho | 19.763.984 | 19,76 | 18,85 |
| `wc-min` refeito no estudo de unificação | tamanho | 19.770.832 | 19,77 | 18,85 |
| `deckcraft-cli` | padrão do repositório | 35.296.984 | 35,30 | 33,66 |
| `deckcraft-cli` | tamanho | 14.504.104 | 14,50 | 13,83 |
| `dc-min` (mini PowerPoint) | tamanho | 6.673.360 | 6,67 | 6,36 |
| mini PowerPoint refeito no estudo de unificação | tamanho | 6.673.136 | 6,67 | 6,36 |
| `gridcraft-cli` original | padrão do repositório | 15.783.648 | 15,78 | 15,05 |
| `gridcraft-cli` com PNG | padrão do repositório | 18.156.800 | 18,16 | 17,32 |
| `gridcraft-cli` com PNG, strip | padrão com strip | 14.530.104 | 14,53 | 13,86 |
| mini Excel (abre xlsx e exporta PDF, sem PNG) | tamanho | 6.428.568 | 6,43 | 6,13 |
| GridCraft original mínimo (variante `orig`) | tamanho | 6.432.920 | 6,43 | 6,13 |
| GridCraft fase 1 (variante `phase1`) | tamanho | 6.435.480 | 6,44 | 6,14 |
| GridCraft com PR (variante `pr`) | tamanho | 8.384.912 | 8,38 | 8,00 |
| `gc-min` | tamanho | 8.388.368 | 8,39 | 8,00 |
| `gc-min` com `--crop` | tamanho | 8.407.856 | 8,41 | 8,02 |
| `gp-min` A3 | tamanho | 11.409.736 | 11,41 | 10,88 |
| `gp-min` A4 sem zoom | tamanho | 11.407.912 | 11,41 | 10,88 |
| Word e PowerPoint combinado | tamanho | 22.472.480 | 22,47 | 21,43 |
| Word, PowerPoint e Excel combinado (Excel só PDF) | tamanho | 26.600.832 | 26,60 | 25,37 |
| Soma dos três minis (Word, PowerPoint, Excel só PDF) | tamanho | 32.865.688 | 32,87 | 31,34 |
| LibreOffice instalado (`/usr/lib/libreoffice`) | pacotes dpkg | 320.967.844 | 320,97 | 306,10 |

Pontos-chave:

- Combinado contra soma: 6.264.856 bytes a menos (6,26 MB, 19,06%). Word e PowerPoint sozinhos: 3.964.640 bytes (3,96 MB, 15%). Curva: Word 19,77 MB, mais 2,70 MB do PowerPoint, mais 4,13 MB do Excel. Compactado com xz -9e: soma dos minis 10,56 MB, combinado 8,43 MB.
- O combinado medido não inclui o backend PNG do Excel: o Excel entrou só exportando PDF. O backend PNG isolado pesa 1,95 MB (1,86 MiB) no GridCraft, mas no binário unificado `vello_cpu`, `skrifa`, `kurbo`, `peniko` e `png` já existem por causa do Word e do PowerPoint, então o custo incremental esperado é menor (estimativa, não medida). O construtor registra o tamanho real do binário final em `SIZES.md`.
- Quebra do `gc-min` de 8,39 MB: motor e PDF 6,43 MB (6,13 MiB), backend PNG 1,95 MB (1,86 MiB), `Rec` e `replay` 2.560 bytes, `main` e lock cerca de 7.800 bytes. Código do PNG por crate (`.text`, cargo-bloat): `vello_cpu` 1,0 MiB, `vello_common` 151 KiB, `skrifa` 146 KiB, `gridcraft_raster` 57 KiB, `png` 42 KiB. Código do motor: `gridcraft_engine` 1,7 MiB, `gridcraft_xlsx` 514 KiB, `std` 501 KiB, `gridcraft_functions` 373 KiB.
- Fontes embutidas: WordCraft 13 TTFs, 4.305.160 bytes (4,31 MB, 4,11 MiB), mais 617 KB de hifenização e ortografia; DeckCraft e GridCraft (motor) nenhuma; patch do GridCraft 12 TTFs, 238.924 bytes (0,24 MB).
- Dependências dos minis: 157 crates externos (Word), 90 (PowerPoint), 57 (Excel), 304 de soma, 160 na união, 41 comuns aos três e 87 comuns entre Word e PowerPoint.

## 3. Regressão do PDF do GridCraft (fase 1)

| Camada | Resultado |
|---|---|
| A, bytes (`cmp`) | 306 de 306 PDFs idênticos |
| B, streams (`qpdf --qdf`) | não precisou rodar |
| C, pixels (50 dpi, `compare -metric AE`) | 0 pixels diferentes em todas as páginas |
| Texto e código de saída | 325 de 325 iguais, incluindo 19 casos sem PDF |
| Total | 325 de 325 PASS |
| Teste de sanidade do detector (defeito injetado) | 99 FAIL, 226 de 325 PASS, as três camadas acusaram |

Desempenho do PDF com a refatoração (10 mil linhas): 0,20 a 0,23 s contra 0,20 a 0,35 s no original, sem diferença mensurável.

## 4. Build

Tempos com 2 vCPUs compartilhadas: WordCraft limpo 7 a 8 min; DeckCraft CLI padrão 753 s, otimizada 770 s, mínimo 523 s (dois builds em paralelo); GridCraft `gc-min` 3 min 33 s, depois 1 min 48 s; `gp-min` release 5 min 36 s (cargo check frio 7 min 23 s), rebuild A4 2 min 27 s, A4 sem zoom 1 min 49 s; combinados 164 a 814 s (ver `TESTS.md`, T06).

## 5. Tabela comparativa por formato de entrada (original contra LibreOffice)

| Formato | WordCraft e afins | LibreOffice |
|---|---|---|
| .docx, .odt, .rtf, .doc 97 | lê, layout próximo do LibreOffice; .doc com tabela sem bordas | lê |
| .html, .md, .tex, .txt | lê (md sem interpretar quando a extensão engana; imagens relativas não carregam) | .html e .doc que contém html não gerou PNG |
| .pptx, .ppsx, .potx | DeckCraft lê | lê |
| .ppt, .odp | DeckCraft não lê | lê |
| .xlsx | GridCraft só com patch | lê |
| .xls, .ods | GridCraft importa só dados (sem estilos nem gráficos) | lê tudo |
| .csv, .tsv | GridCraft lê | lê (rodapé "Page 1") |
| .xlsb | GridCraft importa só dados (não testado: nenhuma ferramenta local grava XLSB) | não testado |
| PDF | nenhum dos três motores renderiza PDF por enquanto (avaliação em `PDF.md`) | |
