# PDF no thumbcraft: comparação de bibliotecas no Windows

> **Situação atual (2026-10-19):** o Hayro, pelo `craft-pdf`, já está no binário do thumbcraft. A medição das
> proteções dentro do binário (os mesmos PDFs hostis, pico de memória e tempo) e o custo de tamanho estão em
> `.tasks/08-pdf-engine.md`. Esta comparação fica como foi escrita; o Hayro "direto" medido aqui (1.061 MB e
> 2.055 MB nas bombas) não é o que roda no thumbcraft.

Data: 2026-10-19. Complementa `PDF.md` (medido em Linux com 2 vCPUs compartilhadas). Aqui a mesma pergunta, refeita no Windows e com bibliotecas novas: qual biblioteca gera a miniatura da primeira página de um PDF com o melhor equilíbrio entre velocidade, memória, robustez, qualidade e custo de distribuição, e o que isso significa para um futuro suporte a PDF no binário unificado.

Convenções: MB decimais (1 MB = 1.000.000 bytes). **Medido** é o que foi observado nesta rodada; **estimado** vem sempre marcado. Todos os candidatos renderizam a primeira página na mesma largura (600 px e 1200 px) e gravam PNG RGB.

## 1. Resumo

| Critério | Melhor | Observação |
|---|---|---|
| Velocidade (processo novo, 600 px) | hayro (12,8 ms de mediana) | pdfium 20,8 ms, mutool 22,5 ms, poppler 52,7 ms, pdf-parse 356,1 ms |
| Velocidade (em processo, aquecido, 600 px) | hayro (4,0 ms) | pdfium 5,2 ms, pdf-parse 19,9 ms |
| Memória do pico (aquecido) | hayro (37 MB) e pdfium (49 MB) | pdf-parse chegou a 1.054 MB (600 px) e 2.165 MB (1200 px) |
| Robustez contra bombas | pdfium e mutool | hayro direto alocou 2.055 MB na bomba de imagem; pdf-parse estourou os 30 s |
| Aceita arquivos quebrados | hayro, pdf-parse, poppler e mutool (3 de 3) | o pdfium recusou o xref corrompido |
| Qualidade visual | empate | os cinco são visualmente equivalentes (seção 6) |
| Custo de distribuição | hayro (6,65 MB isolado, Rust puro) | pdfium.dll 7,5 MB (binário nativo à parte), mutool 46,5 MB, Poppler 55 MB, pdf-parse 92 MB de `node_modules` |

**Recomendação.** Para um serviço que gera miniaturas de PDFs de terceiros, o **pdfium** é a escolha mais segura: perto do hayro em velocidade (1,6 vez mais lento no COLD e 1,3 vez no WARM), com memória baixa na bomba de imagem (38 MB) e mantido pelo Chrome, embora também use 1 GB na bomba de fluxo. O **hayro** é o mais rápido e o único candidato em Rust puro, o que o torna o único viável para entrar no binário unificado, mas sem proteção própria contra PDFs hostis quando usado direto (seção 5). Ele só deve entrar com limites de memória e de pixels por fora, como o `craft-pdf` já faz. O **pdf-parse** é o mais lento e o mais pesado; só se justifica por já estar no serviço Node. **Poppler** é lento e tem um defeito com páginas rotacionadas. O **mutool** é rápido e tolerante, mas 46,5 MB e licença copyleft.

## 2. Candidatos

| Candidato | Versão | Como foi chamado |
|---|---|---|
| hayro (direto) | crate `hayro` 0.8.0 (crates.io), Rust puro, só CPU, MIT ou Apache-2.0 | `hayro::render` com escala `largura / largura_da_página`, fundo branco |
| pdfium | `pdfium-render` 0.9.4 com `pdfium.dll` chromium/8100 (pdfium-binaries) | `set_target_width`, fundo branco |
| pdf-parse | 2.4.5 (pdf.js 5.4.296 e `@napi-rs/canvas`), Node 24.19.0 | `getScreenshot({ first: 1, desiredWidth })` |
| mutool | MuPDF 1.28.5 (pacote do scoop) | `mutool draw -q -o SAIDA -w LARGURA -F png ENTRADA 1` |
| poppler | `pdftoppm` 25.07.0 (winget) | `-png -f 1 -l 1 -scale-to-x LARGURA -scale-to-y -1 -singlefile` |
| node vazio | Node 24.19.0 | `node -e 0`, linha de base do custo de iniciar o Node |

**Hayro, o que é.** Rasterizador de PDF em Rust, sem GPU, que lê e desenha PDFs (sem suporte a PDF criptografado com senha). Os projetos *craft usam o Hayro dentro do crate `craft-pdf` (`craft-libs`), na feature `raster`, mas só a feature `write` (escrever PDF) está compilada nos binários atuais. Por isso aqui ele entra "direto", pelo crate público, sem o `craft-pdf` no meio. O `pdf-min` do `PDF.md` era o Hayro via `craft-pdf`.

**Fora da comparação: `mupdf` (crate Rust).** O crate `mupdf` 0.8.0 não compilou no Windows: o `mupdf-sys` usa MSBuild e exige fontes de `zxing-cpp` que o pacote não traz (erros `C1083`, depois de ajustar `MUPDF_MSVC_PLATFORM_TOOLSET=v145` por só haver o toolset do Visual Studio 2026). O MuPDF entra pelo `mutool` oficial, o mesmo motor, como no `PDF.md`.

## 3. Método

- **COLD:** um processo novo por execução. Um runner em Rust (`runner`) dispara cada comando, mede o tempo de relógio e lê `PeakWorkingSetSize` do processo filho (pico de memória física). 32 PDFs válidos (30 válidos mais os 2 digitalizados, mesma definição do `PDF.md`; ficam fora o de zero páginas, os criptografados e os quebrados), 5 repetições intercaladas entre os candidatos, larguras 600 px e 1200 px. "Mediana" é a mediana das medianas por arquivo; "p95" é o percentil 95 de todas as amostras; "disp." é a amplitude interquartil sobre a mediana.
- **WARM:** um processo por candidato e largura, 3 execuções de aquecimento e 20 medidas por arquivo, em processo. Vale para hayro, pdfium e pdf-parse. Poppler e mutool só existem como processo externo, então o COLD é o número deles. A memória é o pico do processo inteiro durante os 32 arquivos (não por chamada).
- **Robustez:** os 45 PDFs de `corpus/pdf/` (inclui 78, um JPEG com extensão `.pdf`) e 3 PDFs hostis gerados à parte, 1 execução por candidato, timeout de 30 s.
- **Qualidade:** saídas de 600 px dos cinco, comparadas com a saída do pdfium (referência escolhida por ser o motor do Chrome) pelo erro absoluto médio por pixel (MAD, de 0 a 255) sobre a área comum, mais 6 folhas de contato vistas à mão.
- **PNG:** os candidatos Rust usam o mesmo codificador (`png` 0.18, compressão balanced); os demais usam o próprio.

### Ambiente

Windows 11 Home, Intel Core i5-13450HX (16 processadores lógicos), 15,7 GB de RAM, SSD local, Rust 1.99.1 (perfil de release do harness: `opt-level = 3`, LTO thin), Node 24.19.0. Máquina de uso normal, sem outros builds iniciados por mim durante as medições, mas com o Windows Defender ativo; não medi a carga de fundo. O custo de criar processo no Windows é maior que no Linux: o `node -e 0` leva 41 ms.

## 4. Desempenho

### COLD (processo novo)

600 px, 32 arquivos, 160 amostras por candidato:

| Candidato | Mediana (ms) | p95 (ms) | Mín e máx (ms) | Disp. | Pico de memória (MB) | PNG mediano (KB) |
|---|---|---|---|---|---|---|
| node vazio | 41,0 | n/d | 37,6 a 41,9 | n/d | 32,8 | n/d |
| pdf-parse | 356,1 | 555,9 | 279,0 a 659,2 | 14% | 86,0 | 24,6 |
| hayro | 12,8 | 53,7 | 8,7 a 117,8 | 120% | 8,7 | 22,3 |
| pdfium | 20,8 | 69,6 | 15,8 a 134,7 | 26% | 14,3 | 21,1 |
| mutool | 22,5 | 89,2 | 14,5 a 270,1 | 26% | 10,3 | 18,8 |
| poppler | 52,7 | 708,6 | 28,4 a 994,1 | 68% | 13,3 | 19,4 |

1200 px, 32 arquivos:

| Candidato | Mediana (ms) | p95 (ms) | Mín e máx (ms) | Disp. | Pico de memória (MB) | PNG mediano (KB) |
|---|---|---|---|---|---|---|
| pdf-parse | 382,5 | 740,6 | 310,8 a 798,1 | 14% | 90,9 | 59,7 |
| hayro | 20,4 | 128,3 | 12,1 a 153,4 | 40% | 18,0 | 52,6 |
| pdfium | 29,5 | 147,7 | 18,3 a 238,6 | 35% | 29,0 | 50,5 |
| mutool | 46,6 | 266,6 | 25,0 a 510,8 | 29% | 18,3 | 50,0 |
| poppler | 118,1 | 2.124,4 | 52,7 a 3.111,5 | 87% | 18,8 | 43,9 |

A dispersão alta do hayro a 600 px vem do conjunto misturar arquivos de 10 ms com o vetorial pesado (100 ms): a medida de dispersão relativa é grande porque a mediana é pequena. O pdf-parse paga cerca de 40 ms de partida do Node mais o carregamento do pdf.js e do canvas (86 MB já antes de renderizar). O Poppler sofre com imagens grandes (digitalizados: 713 e 963 ms).

### WARM (em processo, aquecido)

| Largura | Candidato | Mediana (ms) | Típicos (ms) | Pesados (ms) | Mín e máx (ms) | PNG (KB) | Pico do processo (MB) |
|---|---|---|---|---|---|---|---|
| 600 | pdf-parse | 19,9 | 18,4 | 194,4 | 8,7 a 265,6 | 24,6 | 1.054 |
| 600 | hayro | 4,0 | 3,9 | 45,3 | 1,2 a 110,2 | 22,3 | 37 |
| 600 | pdfium | 5,2 | 5,1 | 48,3 | 1,5 a 130,6 | 21,1 | 49 |
| 1200 | pdf-parse | 55,4 | 54,7 | 375,3 | 23,6 a 432,3 | 59,7 | 2.165 |
| 1200 | hayro | 11,3 | 11,3 | 113,2 | 4,4 a 144,3 | 52,6 | 42 |
| 1200 | pdfium | 13,8 | 13,2 | 120,2 | 5,1 a 217,6 | 50,5 | 61 |

"Pesados" são o vetorial (48) e os dois digitalizados (60 e 61); "típicos" são os outros 29. O pdf-parse acumula memória entre chamadas (o canvas nativo não devolve o que usou): o pico do processo passou de 1 GB a 600 px e de 2 GB a 1200 px depois de 640 chamadas. Não investiguei se é cache do Skia ou coleta tardia.

### Por arquivo (COLD, 600 px, mediana em ms)

| Arquivo | pdf-parse | hayro | pdfium | mutool | poppler |
|---|---|---|---|---|---|
| 10_word_simples_acentos | 365,7 | 14,4 | 19,9 | 20,7 | 54,8 |
| 13_word_multipagina | 356,9 | 19,1 | 24,3 | 25,8 | 90,3 |
| 23_excel_planilha_larga | 356,1 | 15,8 | 20,6 | 22,4 | 60,0 |
| 30_ppt_formas_gradiente | 413,9 | 26,2 | 42,4 | 38,3 | 114,7 |
| 34_ppt_fundo_imagem | 390,2 | 16,4 | 23,3 | 22,6 | 100,0 |
| 35_ppt_rtl_cjk_emoji | 356,1 | 12,2 | 19,0 | 20,3 | 34,9 |
| 42_pagina_vazia | 339,7 | 11,2 | 18,3 | 19,7 | 29,8 |
| 46_pagina_gigante_A0 | 365,5 | 12,9 | 21,3 | 26,4 | 71,4 |
| 48_vetorial_pesado_mapa | 602,0 | 100,8 | 131,3 | 263,2 | 311,2 |
| 60_escaneado_so_imagem | 477,4 | 43,6 | 52,2 | 89,2 | 712,7 |
| 61_escaneado_cor_300dpi_4pag | 557,4 | 53,7 | 71,8 | 73,0 | 962,9 |

## 5. Robustez

Gera um PNG de 600 px (1 execução por candidato e arquivo, timeout de 30 s). O sucesso aqui quer dizer "devolveu imagem", não "a imagem está correta".

| Categoria | pdf-parse | hayro | pdfium | mutool | poppler |
|---|---|---|---|---|---|
| Válidos e digitalizados (32) | 32 | 32 | 32 | 32 | 32 |
| Criptografado sem senha de abertura (51 e 52) | 2 de 2 | 2 de 2 | 2 de 2 | 2 de 2 | 2 de 2 |
| Criptografado com senha de abertura (50) | falha | falha | falha | falha | falha |
| Truncado, 50% e 90% (70 e 71) | 0 de 2 | 0 de 2 | 0 de 2 | 0 de 2 | 0 de 2 |
| Corrompido: meio, xref, lixo antes do cabeçalho (72, 73 e 74) | 3 de 3 | 3 de 3 | 2 de 3 (recusou o xref) | 3 de 3 | 3 de 3 |
| Arquivo vazio (75) | falha | falha | falha | falha | falha |
| Texto, HTML e JPEG com extensão `.pdf` (76, 77 e 78) | 0 de 3 | 0 de 3 | 0 de 3 | 3 de 3 | 0 de 3 |
| Zero páginas (43) | falha | falha | falha | falha | falha (código 99) |

Observações:

- O `mutool` abre por conteúdo e renderiza texto puro, HTML e imagem mesmo com extensão `.pdf`. É útil para um serviço permissivo e ruim para um que precisa rejeitar não-PDF; é preciso validar o cabeçalho `%PDF` antes.
- O pdfium rejeitou o PDF de xref corrompido (73), que os outros quatro leem (reconstruindo a tabela). Entre os quatro que o aceitam, nenhum mostrou erro visível na miniatura.
- Nenhum candidato desenha PDF com senha de abertura; é erro limpo em todos.

### Bombas e limites (3 PDFs hostis gerados à parte, fora do git)

| Caso | pdf-parse | hayro | pdfium | mutool | poppler |
|---|---|---|---|---|---|
| Bomba de fluxo (conteúdo da página de 512 MB em 0,52 MB comprimidos) | ok, 3.095 ms, 850 MB | ok, 740 ms, **1.061 MB** | ok, 521 ms, **1.034 MB** | ok, 960 ms, 8 MB | ok, 2.818 ms, 12 MB |
| Bomba de imagem (cinza 40000 x 40000, 1,6 GB descomprimidos em 1,56 MB) | **estourou os 30 s** | ok, 1.911 ms, **2.055 MB** | ok, 628 ms, 38 MB | ok, 1.592 ms, 12 MB | ok, 6.379 ms, 13 MB |
| Página de 14400 x 14400 pt | ok, 322 ms, 76 MB | ok, 11 ms, 7 MB | ok, 18 ms, 12 MB | ok, 21 ms, 8 MB | ok, 135 ms, 12 MB |

Leitura: o **hayro direto não se protege** contra a bomba de fluxo (1 GB) nem contra a de imagem (2 GB): usado cru, um PDF de 1,5 MB pode custar 2 GB de memória. A avaliação em Linux (`PDF.md`) mediu o `pdf-min` com 10 a 12 MB nesses mesmos casos; a diferença é que ali o Hayro vinha dentro do `craft-pdf`, que impõe orçamentos de decodificação e de pixels. Essa proteção não vem no crate `hayro` sozinho. O pdfium ficou com 1 GB na bomba de fluxo mas só 38 MB na de imagem. `mutool` e Poppler mantiveram a memória em 8 a 13 MB nos três casos (o Poppler pagou em tempo: 6,4 s na bomba de imagem). Não havia limite de memória do processo (no Windows seria um Job Object, que não foi usado), então os números são o que cada biblioteca decidiu alocar.

## 6. Qualidade visual

Comparação com o pdfium como referência (MAD por pixel de 0 a 255, 32 PDFs válidos, 600 px, largura e altura comuns):

| Candidato | MAD mediano | MAD máximo | Maior diferença de largura e altura (px) |
|---|---|---|---|
| pdf-parse | 0,82 | 14,82 (48 vetorial pesado) | 0 e 1 |
| hayro | 0,80 | 11,76 (48 vetorial pesado) | 0 e 1 |
| mutool | 0,76 | 31,65 (48 vetorial pesado) | 0 e 1 |
| poppler | 1,05 | 25,31 (48 vetorial pesado) | 177 e 136 (arquivo 49, rotacionado) |

- As diferenças são de suavização de bordas e de renderização de texto reduzido; as folhas de contato (acentos, tabelas, imagens, gradientes, transparência, formulário, A0, tamanhos mistos, rotação, vetorial, digitalizados) não mostram texto cortado, cor errada nem elemento faltando em nenhum candidato. O MAD alto do vetorial pesado (12.000 curvas sobrepostas) vem de pequenas diferenças de antialiasing em milhares de bordas.
- **Defeito do Poppler com página rotacionada:** no arquivo 49 (`/Rotate 90`) o `pdftoppm -scale-to-x 600` entregou 777 x 600 em vez de 600 x 464 como os demais: a largura pedida é aplicada ao lado errado antes de girar. Quem usar Poppler precisa tratar `/Rotate` à parte.
- O Poppler também devolve 1 px a mais de altura que os demais em páginas A4 e Letter (arredondamento diferente), sem importância visual.
- O fundo é branco opaco em todos (pedido explicitamente para os Rust e para o mutool; o pdf-parse e o Poppler já entregam fundo opaco).

## 7. Custo de distribuição e de integração

| Candidato | Tamanho | Observação |
|---|---|---|
| hayro | 6.645.248 bytes (6,65 MB) como binário isolado com o perfil de tamanho do thumbcraft (`opt-level = z`, LTO fat, `panic = abort`, strip); `rlibs.exe` de teste (hayro e pdfium-render juntos, perfil de velocidade) tem 7,87 MB | Rust puro, sem DLL |
| pdfium | `pdfium.dll` 7.512.064 bytes (7,51 MB) mais o binário da aplicação | biblioteca nativa carregada em tempo de execução; é preciso distribuir a DLL por plataforma (Windows e Linux, cada uma) |
| mutool | `mutool.exe` 46.476.800 bytes (46,5 MB) | pacote completo do MuPDF 1.28.5 tem mais executáveis |
| poppler | 55 MB (pasta do pacote do winget, com DLLs) | |
| pdf-parse | 92 MB de `node_modules` | exige Node |

**Hayro dentro do binário unificado (estimado).** O thumbcraft tem hoje 23.995.392 bytes no Windows (medido: build de release de 4 min 51 s). O binário isolado do Hayro tem 6,65 MB. Parte do código seria compartilhada (`kurbo` 0.13.1, `peniko` 0.6.1 e `png` estão nas mesmas versões nos dois `Cargo.lock`), mas o `hayro` 0.8.0 exige `vello_cpu` 0.3.0 enquanto o thumbcraft usa `vello_cpu` 0.2.0, então o rasterizador entraria duas vezes; o `skrifa` também tem versões diferentes. Faixa estimada de custo incremental: de 4 a 6,6 MB (o limite superior é o binário isolado inteiro). **Não foi medido dentro do thumbcraft**; a medição certa é adicionar a dependência atrás de uma feature e comparar o tamanho de release. O ponto da incompatibilidade de `vello_cpu` é o mesmo que o `PDF.md` já registrava para o `craft-pdf`.

## 8. Recomendação detalhada

1. **Para o serviço de upload (Node) hoje:** trocar o pdf-parse por processo externo é o que traz ganho real. Pdfium (por `pdfium-render` ou binário próprio) tem o melhor equilíbrio: 20,8 ms de COLD, memória baixa na maioria dos casos hostis e mantido pelo Chrome. O pdf-parse custa 17 vezes mais tempo de COLD e acumula mais de 1 GB de memória em processo longo.
2. **Para PDF no binário unificado:** o hayro é o único em Rust puro e o mais rápido (12,8 ms COLD, 4,0 ms aquecido), mas sem proteção própria contra PDFs hostis (1 a 2 GB nas bombas). Só entra com limites externos: tamanho máximo do arquivo, limite de pixels da página, timeout e, se possível, limite de memória do processo. O `craft-pdf` já traz parte disso; ele deveria ser o caminho, não o crate `hayro` cru.
3. **Se o requisito é rejeitar o que não é PDF:** validar o cabeçalho `%PDF` antes de qualquer biblioteca, por causa do `mutool`.
4. **Poppler e mutool:** só como referência. O Poppler é o mais lento (53 ms COLD, 963 ms em imagem grande) e erra páginas rotacionadas; o mutool é rápido mas pesado (46,5 MB) e copyleft. A licença do Poppler (GPL) e do MuPDF (AGPL) é conhecimento geral, não verificado nesta rodada; confira antes de distribuir.

## 9. Limitações

- **Uma máquina, um sistema.** Só Windows 11 em um i5-13450HX; nada de Linux nem macOS nesta rodada. Os números absolutos não se comparam com os do `PDF.md` (outra máquina, outro sistema, outros candidatos); a ordem entre candidatos é a parte comparável.
- **Repetições:** 5 no COLD, 20 no WARM, 1 na robustez e nas bombas. A dispersão entre processos no WARM não foi medida.
- **Corpus pequeno e gerado** (PDFs do LibreOffice, do qpdf e escritos à mão): não inclui PDFs reais de usuários (digitalizados reais, JBIG2, JPEG 2000, XFA). As diferenças entre motores costumam aparecer justamente aí. Os dois PDFs grandes do `PDF.md` (1126 páginas, 33 MB) não foram refeitos.
- **Bloqueio do Windows durante a medição.** O Acesso Controlado a Pastas do Windows Defender estava ativo e bloqueou gravações de executáveis novos em pastas protegidas até cada um ser aprovado. As primeiras execuções de robustez (e só elas) tiveram falhas por esse motivo e foram refeitas depois das aprovações; as medições de tempo do COLD gravaram as imagens em uma pasta temporária, fora da proteção.
- **Sem limite de memória** (`ulimit` não existe no Windows; Job Objects não foram usados).
- **mupdf-rs** não compilou (seção 2), então o MuPDF foi medido pelo `mutool`, que inclui o custo de criar processo.
- **Hayro direto, sem as proteções do `craft-pdf`:** o resultado de robustez não vale para o Hayro dentro do `craft-pdf`, que não foi medido nesta rodada.
- **Qualidade:** medida contra o pdfium (não contra um gabarito) e vista à mão em 6 folhas de contato; sem métrica perceptual.
- **Tamanho no binário unificado:** estimativa, não medição (seção 7).
- **Poppler no WARM:** não se aplica (só existe como processo externo).

## 10. Como refazer

O código do benchmark **não é versionado** (regra do projeto: `docs/STATUS.md`). Ele fica em `benchmarks/pdf/`, pasta ignorada pelo git, com a estrutura:

- `rlibs/`: crate Rust com os modos `one` (renderiza um arquivo) e `bench` (em processo, aquecido) para hayro e pdfium; `runner/`: executa jobs e mede tempo de relógio e pico de memória do filho; `hmin/`: Hayro mínimo para medir tamanho.
- `node/pp.mjs`: o mesmo para o pdf-parse. `jobs.mjs`: gera a lista de jobs. `hostile.mjs`: gera os 3 PDFs hostis. `analyze.py`, `rob.py` e `quality.py`: agregam tempos, robustez e qualidade.
- `env.sh`: caminhos do pdfium, do mutool e do pdftoppm.

Instalar: Rust 1.95 ou mais novo; `winget install Rustlang.Rustup`; `scoop install mupdf`; `winget install oschwartz10612.Poppler`; `npm install pdf-parse@2.4.5`; `pdfium-win-x64.tgz` da release chromium/8100 de `bblanchon/pdfium-binaries` extraído em `benchmarks/pdf/pdfium/`. Medir, a partir de `benchmarks/pdf` em Git Bash:

```sh
source env.sh && cargo build --release
mkdir -p bin && cp target/release/rlibs.exe target/release/runner.exe bin/
node jobs.mjs 600 out_cold valid > jobs.tsv && bin/runner.exe 5 60 jobs.tsv > res/cold_600.tsv
python analyze.py res/cold_600.tsv
```

Fora da pasta ignorada, o build precisa que o Acesso Controlado a Pastas do Windows permita os executáveis recém-compilados (cada script de build é um executável novo).

## 11. O que foi medido e o que foi estimado

| Item | Situação |
|---|---|
| Tempos COLD e WARM, memória, robustez, bombas, qualidade (MAD e inspeção) | medido |
| Tamanhos de `pdfium.dll`, `mutool.exe`, `node_modules`, pasta do Poppler e do binário isolado do Hayro | medido |
| Custo do Hayro dentro do thumbcraft (4 a 6,6 MB) | estimado |
| Licenças de Poppler e MuPDF | conhecimento geral, não verificado |
| Hayro dentro do `craft-pdf` no Windows | não medido |
