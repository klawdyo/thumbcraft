# Testes realizados

Registro de cada teste da sessão de 2026-10-19: objetivo, entradas, ferramentas e versões, parâmetros exatos, métricas e como repetir. Resultados consolidados em `RESULTS.md`. Os dados brutos estão em `../baselines/`, as entradas em `../corpus/` e os relatórios originais em `../reports/`.

Convenções: tempos medidos com relógio de parede em uma máquina de 2 vCPUs compartilhadas com outros agentes que compilavam, e com cache de disco quente (ver `ENVIRONMENT.md`). Os motores são sempre executados em um processo novo por arquivo, com timeout. Tamanhos em MB decimais com MiB entre parênteses.

## Como os corpora foram gerados

Os scripts geradores eram em Python e não são versionados (regra do projeto). Ficam aqui as ferramentas e os casos; os arquivos gerados estão em `../corpus/`.

### Corpus Word (`../corpus/docx/`, 30 arquivos)

| Faixa | Ferramenta | Casos |
|---|---|---|
| 01 a 08 (.docx) | python-docx 1.2.0 (com lxml 6.1.3) | texto com acentos; títulos e listas; tabelas com e sem borda; imagens PNG e JPEG; 40 capítulos; fontes Calibri, Times New Roman, Arial, Courier New e Cambria; negrito, itálico, sublinhado e cores; cabeçalho e rodapé |
| 09, 10, 20 (.doc 97) | LibreOffice 24.2.7.2 convertendo docx do python-docx | imagem e tabela; títulos e listas; multipágina |
| 11, 12, 13, 15, 18 (.odt, .rtf, .html, .tex) | pandoc 3.1.3 a partir de `fontes-geracao/rich.md` e `multi.md` | formatação rica; HTML com imagem em data URI; multipágina |
| 14, 16, 17, 19 (.md, .txt) | escritos à mão ou por shell | Markdown com tabela e imagem relativa; texto simples; texto longo (mais de 10 páginas) |
| 21 a 30 (inválidos) | shell | vazio; truncado (5000 bytes do 03 e 3000 bytes do 09); extensão trocada (cópias dos 16, 01, 03, 14 e 13); 20000 bytes sem estrutura |

### Corpus PowerPoint (`../corpus/pptx/`, 46 arquivos)

| Faixa | Ferramenta | Casos |
|---|---|---|
| 01 a 32 | python-pptx 1.0.2 com Pillow 12.3.0 (imagens) e edição direta de XML | título; bullets; imagens; formas; gradientes; sombra e transparência; tabelas; gráficos (barras, linhas, pizza); SmartArt montado à mão (com e sem drawing); texto formatado; fontes; fundos; mestre com logotipo; notas; transições; 60 slides; overflow; texto vertical e rotacionado; 4:3 e 16:9; vídeo placeholder; hiperlinks; primeiro slide oculto; RTL, CJK e emoji; imagem em tela cheia com recorte; vários gráficos (barras empilhadas, área e rosca) |
| 33, 34 | zipfile, trocando o content type do 03 | .ppsx e .potx |
| 35 a 39 | LibreOffice 24.2.7.2 a partir de pptx | .ppt e .odp |
| 40 a 46 | zipfile e bytes | vazio; truncado na metade (do 09); 400 bytes sobrescritos no terço do arquivo (do 09); texto puro; XML do slide 1 cortado ao meio (do 21); extensões trocadas (cópias do 01) |
| `mutacao-48/` | zipfile com semente 7 | ver T05 |

### Corpus Excel (`../corpus/xlsx/`, 57 arquivos)

XlsxWriter 3.2.9 e openpyxl 3.1.5 para os 42 válidos (casos em `MANIFEST.tsv`), o .xls (43) e o .ods (44) convertidos do 34 pelo LibreOffice, as imagens do 21 geradas com ImageMagick (`convert`, gradiente e plasma), csv e tsv em UTF-8 e Latin-1, e inválidos por manipulação de bytes (50 é o 03 cortado na metade, 51 é o 03 com bytes sobrescritos). Detalhes em `../reports/17-corpus-xlsx-CORPUS.md`. As 16 planilhas de `regressao-amostras/` foram geradas por outro script Python (cobertura descrita no README da pasta).

## T01 WordCraft: premissa da CLI e tamanhos de binário

- Objetivo: confirmar que o WordCraft converte doc em PNG pela CLI só com o core, e medir o binário mínimo.
- Entradas: um docx feito com pandoc e um .doc feito com o LibreOffice, com título, negrito, itálico, acentos e tabela.
- Ferramentas: rustc 1.97.0, WordCraft `117aea31`.
- Parâmetros: `cargo build --release -p wordcraft-cli` (padrão) e com `CARGO_PROFILE_RELEASE_OPT_LEVEL=z`, `LTO=fat`, `CODEGEN_UNITS=1`, `PANIC=abort`, `STRIP=symbols` para `-p wordcraft-cli -p wc-min`. `wc-min` em `../baselines/word/prototipo-wc-min/`.
- Métricas: tamanhos (RESULTS.md), PNG de 1224x1584 (Letter, escala 2) com texto, acentos e negrito corretos e tabela sem bordas; `wc-min` e `wordcraft-cli` geram PNG byte a byte idêntico para o mesmo .doc; `ldd` lista só `libc`, `libm` e `libgcc_s`; build limpo leva 7 a 8 minutos em 2 vCPUs.
- Como repetir: `../baselines/word/README.md`.

## T02 WordCraft: thumbnails em 30 arquivos contra o LibreOffice

- Objetivo: taxa de sucesso, tempo e qualidade visual por formato.
- Entradas: `../corpus/docx/` (30 arquivos).
- Parâmetros: `wc-min ENTRADA SAIDA.png 1` (página 1, escala 2), um processo por arquivo, timeout de 30 s. LibreOffice: `soffice --headless --convert-to png`, um processo novo por arquivo.
- Métricas: PNG gerado, exit, segundos, bytes do PNG, classificação visual (ok, degradado, falhou) e comparação lado a lado em 6 casos (03, 06, 08, 09, 11 e 12).
- Resultados: `../baselines/word/`. Observação: o fundo do PNG do WordCraft é transparente e foi achatado em branco para as miniaturas.
- Verificação visual: contact sheet dos 30 casos (falhas em vermelho) e `comparacao-wordcraft-vs-libreoffice-6-casos.png`.

## T03 GridCraft: existe PNG pela CLI?

- Objetivo: descobrir se o irmão de Excel exporta PNG.
- Método: leitura de `docs/cli.md`, de `apps/gridcraft-cli/src/main.rs` e do crate do motor, com busca por "png". Nada foi compilado.
- Resultado: não existe. O PNG só existe como screenshot da janela do app (`ui.screenshot` do MCP) e no exemplo `snapshot` do `ui-egui` (wgpu). Relatório 03.

## T04 DeckCraft: thumbnails em 46 arquivos contra o LibreOffice

- Objetivo: taxa de sucesso, tempo e qualidade.
- Entradas: `../corpus/pptx/` (46 arquivos).
- Ferramentas: DeckCraft `010941ec` (0.8.0), `dc-min` (`../baselines/powerpoint/prototipo-dc-min/`), LibreOffice 24.2.7.2.
- Parâmetros: `dc-min ENTRADA SAIDA.png 1` (escala 1, slide 0, PNG de 960x540), timeout de 30 s, um processo por arquivo. LibreOffice com perfil dedicado e `pkill` ao fim de cada arquivo (script `../baselines/powerpoint/ferramentas/run.sh`).
- Verificações de equivalência: `dc-min` contra o CLI completo, byte a byte, nos 36 arquivos válidos (idênticos); CLI com perfil padrão contra CLI com perfil de tamanho, 36 pares (idênticos); `dc-min` contra CLI otimizado nos 36 arquivos (idênticos).
- Builds medidos: CLI padrão 753 s, CLI otimizada 770 s, mínimo 523 s (inflados, porque houve dois builds simultâneos e outro agente compilava). Logs em `../baselines/powerpoint/logs-de-build/`.
- Verificação visual: contact sheet e comparação de 12 casos (`../baselines/powerpoint/imagens/`).

## T05 DeckCraft: teste de mutação de 48 variantes

- Objetivo: robustez contra XML alterado, truncado e com números absurdos.
- Entradas: `../corpus/pptx/mutacao-48/`: 6 sementes (09, 12, 18, 05, 03 e 23) com 8 mutantes cada. Semente do gerador aleatório: 7.
- Mutação (k de 0 a 7, modo k mod 4): 0 sobrescreve 5 bytes aleatórios; 1 trunca o arquivo em ponto aleatório; 2 remove de 1 a 49 bytes; 3 troca `val="` por `val="-9999999999` e o primeiro `cx="` por `cx="99999999999`. Altera um único XML do pacote (slide, gráfico, diagrama, mestre ou layout).
- Métricas: PNG gerado, panic, travamento. Resultado: 48 de 48 geraram PNG, stderr vazio, sem panic. Aviso do arquivista: em 46 dos 48 o PNG é byte a byte idêntico ao do original (só `12_..._m1` e `18_..._m1`, ambos truncamentos, mudaram), então a mutação tem pouco efeito sobre o resultado; o teste prova ausência de panic, não resistência a todo tipo de corrupção. Resultados em `../corpus/pptx/mutacao-48/RESULTADOS.tsv`.

## T06 Binário combinado (all-min)

- Objetivo: tamanho do binário único contra a soma dos minis.
- Entradas: binário experimental `all-min` com features `word`, `ppt` e `xlsx` (o Excel só abre e exporta PDF). Fonte em `../reports/anexos/combinado-all-min/`.
- Parâmetros: cinco combinações de features (`ppt`, `xlsx`, `word,ppt`, `word,ppt,xlsx`, `word`), todas com `opt-level=z`, LTO fat, `codegen-units=1`, `panic=abort`, `strip=symbols`. Tempos de build: 348, 203, 814, 319 e 164 s (concorrência com outro build).
- Métricas: bytes, fecho de dependências (`cargo tree`: 157 crates externos no Word, 90 no PowerPoint, 57 no Excel, 304 de soma, 160 na união, 41 comuns aos três), compressão com xz -9e (soma dos minis 10,56 MB, combinado 8,43 MB), linhas de código próprio (Word 154 mil, PowerPoint 53 mil, Excel 76 mil).
- Sanidade: o combinado gera PNG idêntico ao do Word+PPT para docx e pptx, abre .doc, gera PDF válido para .xlsx, e extensão desconhecida sai com exit 2.
- Resultado: 26,60 MB contra 32,87 MB.

## T07 GridCraft: estudos de código (sem execução)

- Custo de PNG por caminho A, B, C e E (relatório 06), fronteira layout e desenho no `print.rs` (07) e display list `Draw` do WordCraft (08). Método: leitura de código com contagem de linhas e funções. Estimativas marcadas como tais nos relatórios.

## T08 GridCraft: corpus xlsx e linha de base do LibreOffice

- Objetivo: criar a linha de base de planilhas.
- Entradas: `../corpus/xlsx/` (57 arquivos).
- Parâmetros: `soffice -env:UserInstallation=file:///tmp/lo-profile-baseline --headless --convert-to png --outdir out-lo ARQUIVO`, um processo por arquivo, `timeout 60`, `nice -n 10`, script `../baselines/excel/ferramentas/run-lo.sh`.
- Métricas: exit, ms, bytes do PNG. Para descobrir como o LibreOffice define a página 1, o PDF do mesmo arquivo foi gerado e contadas as páginas, colunas e linhas (23: 13 páginas, só Col1 a Col10 na primeira; 24: 209 páginas, cerca de 49 linhas na primeira; 29: só C5:F12; 39: só A1; 40: aba visível).
- Resultado: 54 de 57 com PNG, todos exit 0, todos 795x1124 px.

## T09 GridCraft: protótipo gp-min (Caminho C)

- Objetivo: provar PDF do GridCraft mais raster em um binário só.
- Parâmetros por rodada (resultados em `../baselines/excel/resultados/`):
  1. A3, intervalo A1:Z60 ajustado a uma página, margens de 0,15", sem cabeçalho nem rodapé nem títulos, recortado ao conteúdo (8 px de folga), escala 2, fundo branco.
  2. A4 com o mesmo método.
  3. A4 inteira sem zoom: segue o Page Setup da planilha, só o papel é A4, `fitToPage=false`, escala 2 (1190x1683 px) e escala 1 (595x842 px).
- Binário: 11.409.736 bytes (A3) e 11.407.912 bytes (A4 sem zoom), 7 min 23 s para o `cargo check` frio e 5 min 36 s para o release na primeira vez.
- Verificação visual: 8 PNGs na primeira rodada; 6 casos A3 contra A4; 3 casos (130 colunas, 10 mil linhas, área de impressão) contra o LibreOffice na rodada final, sem contagem de pixels. O `35-comentarios-hiperlinks` deu PNG de 2.713 bytes (quase vazio) e não foi investigado.
- Achados: texto não latino vira `?`, texto cortado na célula (`Centro-Oe`), números decimais arredondados diferente do LibreOffice (`493.693` contra `493.69299`, causa não investigada).

## T10 GridCraft: regressão do PDF (fase 1)

- Objetivo: provar que a refatoração do `print.rs` não altera o PDF.
- Entradas: 64 planilhas (16 de `regressao-amostras/` e 48 .xlsx do corpus) em 5 variantes (`exportPdf` padrão, `fitToPage`, `range` B2:F30, `sheets:"all"`, `file.print`) mais 5 casos de erro: 325 casos, 306 com PDF.
- Ferramentas: `cmp`, `qpdf` 11.9.0, `pdftoppm` 24.02.0 (50 dpi), ImageMagick `compare -metric AE` 6.9.12, GridCraft `66ce6be4` (binário original sha256 `8d22cb4d...`).
- Camadas: A bytes, B streams (`qpdf --qdf`, só se A falhar), C pixels. Mais igualdade do texto e do código de saída.
- Resultado: 325 de 325 PASS, 306 de 306 idênticos byte a byte, 0 pixels diferentes. Repetido 5 vezes ao longo da fase 1 e da fase 2 (depois de mover os tipos, ligar `file.exportPng` e no final). Nenhuma normalização (sem data, `/ID` nem UUID).
- Como repetir: `../baselines/gridcraft-pdf-regression/README.md`.

## T11 Regressão: teste de sanidade do detector

- Objetivo: provar que o detector enxerga um defeito.
- Método: deslocar em 1 unidade o `x` de todo `FillRect` no replay, recompilar e rodar. Esperado: FAIL.
- Resultado: 99 FAIL (226 de 325 PASS, 207 de 306 idênticos). As três camadas acusaram: bytes diferentes, diff `50.4 710.727 87.75 27.273 re f` e 339 pixels diferentes em `s-02_styles.d`, página 1. Revertido, recompilado e o resultado voltou a 325 de 325. Tabela em `../baselines/gridcraft-pdf-regression/tabela-teste-de-sanidade-do-detector.txt`.

## T12 Desempenho da refatoração do PDF

- Planilha de 10 mil linhas, `file.exportPdf` (PDF de 2,7 MB), perfil dev otimizado, 6 execuções de cada binário: 0,20 a 0,23 s com a refatoração e 0,20 a 0,35 s no original, sem diferença mensurável (ruído de outro build). Cerca de 0,10 s é só abrir o arquivo. O custo de guardar a lista em memória não foi medido.

## T13 Qualidade do PR de PNG

- Objetivo: checagens que os mantenedores pedem.
- Resultado na fase 2: `cargo test -p gridcraft-pdf -p gridcraft-engine -p gridcraft-raster` com 347 testes passando (346 na fase 1, mais o novo teste de PNG); `cargo fmt --check` limpo; clippy `-D warnings` limpo em todos os crates menos UI, app desktop e web (não tocados); `cargo xtask layers` com 17 crates e 0 violações; `cargo xtask assets` com 46 de 46 atribuídos; `cargo xtask wasm` ok (incluindo `gridcraft-engine`, `gridcraft-raster` e `gridcraft-web`, com o alvo `wasm32-unknown-unknown`); `cargo xtask parity` regenerado (275 de 291). O `cargo xtask ci` inteiro não foi rodado. Teste novo: PNG válido com assinatura e dimensões A4 (1191x1684 a escala 2) e erros, e o PDF não muda.

## T14 GridCraft: corpus com o `gc-min` e comparação das três implementações

- Entradas: 55 arquivos suportados (.xlsx, .xlsm, .xls, .ods e .csv; os 2 .tsv e o manifesto são ignorados).
- Parâmetros: `gc-min ENTRADA SAIDA.png [--scale N] [--crop] [--fit] [--range A1:Z60]`, escala padrão 2, timeout de 30 s (`../baselines/excel/ferramentas/run-corpus-gc-min.sh`). Rodadas: escala 2, escala 1 e escala 2 com `--crop` (8 px de folga).
- Resultado: 52 de 55 com PNG, 3 falhas esperadas (49, 50 e 51, exit 1 com erro limpo).
- Verificação visual: cerca de 14 PNGs (bordas, gráficos, texto rotacionado, imagens, mesclagem, formatação condicional, quebra e transbordo, primeira aba oculta, área de impressão): sem defeito de desenho; texto não latino vira `?`. Crop: 6 PNGs conferidos, planilha vazia mantém a página inteira.
- Comparação com o `gp-min` em A4 sem zoom: RMSE de 0,6% a 1,5% nos 7 pares testados; tamanho difere de 1 px (1191 contra 1190).
- Comparação final em três colunas com `compare.sh` (`../baselines/excel/ferramentas/compare.sh`, 3 colunas por 7 linhas por imagem) em `../baselines/excel/imagens/`.

## T15 Quebra do tamanho do binário do GridCraft

- Objetivo: de onde vêm os 8,39 MB do `gc-min`.
- Método: três variantes mínimas (`orig`, `phase1`, `pr`) em `opt-level=z`, LTO fat, `codegen-units=1`, `panic=abort`, strip, mais `cargo-bloat` 0.12.1 em build com símbolos (`../baselines/excel/tamanhos/`).
- Resultado: orig 6.432.920 bytes, fase 1 6.435.480, PR 8.384.912, `gc-min` 8.388.368 e com crop 8.407.856. Backend PNG: 1.949.432 bytes (1,95 MB), dos quais 1,51 MB de código, 0,24 MB de fontes e cerca de 0,20 MB de outras seções. O mini Excel do estudo de unificação (6.428.568 bytes) difere do orig por 4.352 bytes (`main` e lock).

## T16 Conflitos de dependência encontrados

- `c3487320` do craft-libs só tem o writer do `craft-pdf`; `raster` existe a partir de `c07b3fa` e `ca50d63`, no `main`. O `gp-min` usou `ae2f796b673f29cf3161f36eb6a8933f0a3fe028`. Não foi testado se subir a revisão no WordCraft e no DeckCraft quebra o `krilla`.
- Os patches do hayro ficam em `vendor/` do craft-libs por `[patch.crates-io]`, que é ignorado em dependências: o crate raiz precisa repetir `hayro`, `hayro-interpret`, `hayro-syntax` e `hayro-jbig2` no mesmo rev.
- Rust 1.97 e edition 2024 foram aceitos sem problema. O lock do GridCraft copiado para o `gp-min` só ganhou o que faltava.
