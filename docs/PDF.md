# PDF no thumbcraft: avaliação e benchmark

Data: 2026-10-19. Pergunta do usuário: o código atual do serviço Node de upload, que gera a miniatura da primeira página de um PDF com a biblioteca npm `pdf-parse`, é mais ou menos performático do que usar os projetos *craft, e o suporte a PDF deve ficar fora ou dentro do binário unificado?

Convenções: MB decimais (1 MB = 1.000.000 bytes), MiB entre parênteses quando relevante. **Medido** quer dizer observado nesta sessão; **estimado** quer dizer deduzido (a estimativa vem sempre marcada). Os tempos foram medidos em uma máquina de 2 vCPUs compartilhada com outros agentes que compilavam (ver "Limitações").

## 1. Respostas objetivas às duas dúvidas

**(a) O binário unificado já suporta PDF? Não, como entrada.** O `thumbcraft` detecta o cabeçalho `%PDF` e responde "PDF ainda não é suportado" (erro 3, PDF "reservado"; ver `crates/core/src/route.rs` e a saída de `thumbcraft info`). Os três motores só **escrevem** PDF (WordCraft e DeckCraft via `craft-pdf` com `krilla`, GridCraft com escritor próprio). Nenhum **lê** ou rasteriza PDF. Verificação (medido): nos binários `thumbcraft` (26.992.128 bytes), `wc-min` e `dc-min` e `wordcraft-cli`, a contagem de ocorrências da palavra `hayro` é 0 (e `krilla` aparece nos que têm Word); no `pdf-min` desta avaliação a contagem de `hayro` é 15.

**(b) Existe um "pdfcraft" incluído nos binários atuais? Não. "PdfCraft" é um aplicativo separado.** Ele aparece na tabela "The Crafting Apps" dos READMEs do GridCraft e do DeckCraft ("Reading, organizing and protecting PDFs", repositório `github.com/storytold/pdfcraft`, Rust, alpha, MIT ou Apache-2.0; o `ROADMAP` do DeckCraft registra, na linha de 2026-10-07, que o PrintCraft foi renomeado para PdfCraft). Segundo a página do repositório (lida por `WebFetch`), ele é um leitor e editor de PDF com CLI (`pdfcraft-cli`, que inclui renderizar páginas para PNG) e hoje desenha as páginas com o crate `hayro`, "enquanto seu próprio renderizador é construído". Não é um crate que os binários atuais usem. O que existe em comum é outra coisa: o crate **`craft-pdf` do `craft-libs`**, que o `ATTRIBUTION.md` do craft-libs descreve como a centralização dos quatro códigos do Hayro (com patches) vindos do PdfCraft. O `craft-pdf` tem as features `write` (krilla), `read`, `interpret` e `raster` (hayro).

O que cada binário atual inclui (medido nos `Cargo.lock` e nos binários):

| Binário | `craft-pdf` | Revisão do craft-libs | Features | Lê ou rasteriza PDF? |
|---|---|---|---|---|
| WordCraft (`wc-min`, `wordcraft-cli`) | sim | `c3487320` | só `write` (krilla 0.8.2) | não |
| DeckCraft (`dc-min`) | sim | `c3487320` | só `write` | não |
| GridCraft | não (escritor próprio `gridcraft-pdf`) | `d27646a5` só para `craft-ui` e `craft-print` na GUI | n/d | não |
| thumbcraft (3 motores) | sim, o dos dois acima | `c3487320` | só `write` | não |
| `pdf-min` (esta avaliação) | sim | `ae2f796` (main) | só `raster` (`read`, `interpret`, hayro) | sim |

Nota sobre os locks: os `Cargo.lock` do WordCraft e do DeckCraft **listam** `hayro-syntax`, `hayro-interpret`, `hayro-cmap` e afins, mas só como `[dev-dependencies]` dos crates `wordcraft-pdf` e `deckcraft-pdf` (testes que releem o PDF gerado). Eles não entram em compilação de release, o que a ausência de `hayro` nos binários confirma. A feature `raster` só existe a partir do commit `c07b3fa` do `main` do craft-libs; o `c3487320` só tem o escritor.

## 2. Resumo e recomendação

Resultados em uma linha (medianas, primeira página, 600 px de largura, 32 PDFs válidos; detalhes nas seções seguintes):

| Cenário | pdf-parse | pdf-min (Rust) | pdftoppm | mutool |
|---|---|---|---|---|
| COLD (processo novo, relógio) | 723 ms | 34 ms | 67 ms | 23 ms |
| WARM no serviço Node (`pdf-parse` em processo, os outros por `spawn`) | 42 ms | 34 ms (stdin e stdout), 45 ms (arquivo temporário) | 82 ms | 29 ms |
| Memória do processo que gera (WARM, RSS) | Node vai de 75 para 328 MB só ao importar a biblioteca, pico mediano 397 MB a 600 px, 674 MB a 1600 px | filho com 10 MB (600 px) a 31 MB (1600 px), Node pai +6 MB | filho de 18 a 30 MB | filho de 12 a 29 MB |
| Tamanho em disco | `node_modules` com 89,17 MB (85,04 MiB) | binário de 5,53 MB (5,27 MiB) | 35 KB mais `libpoppler` 3,62 MB | 44,7 MB (pacote do Ubuntu) |
| Sucesso em PDFs válidos | 30 de 30 | 30 de 30 | 30 de 30 | 30 de 30 |

**Recomendação (fundamentada nos números abaixo):**

1. **Não colocar PDF no binário unificado agora.** O custo estimado é de cerca de 4,2 MB (faixa de 3,9 a 4,6 MB, +16% sobre os 26,99 MB atuais), porque a pilha de raster do Hayro usa `vello_cpu` 0.0.8, que **não** deduplica com o `vello_cpu` 0.2.0 do thumbcraft (cerca de 0,95 MB de código repetido), e exigiria uma segunda revisão do craft-libs. O ganho de ter um binário só não compensa para uma função que o serviço Node já cumpre.
2. **Para a pergunta de desempenho: o código atual com `pdf-parse` não é mais lento.** Dentro de um serviço Node já aquecido, a mediana por arquivo foi de 42 ms (pdf-parse) contra 34 ms (pdf-min por `spawn` com stdin e stdout), uma diferença pequena para o ruído desta máquina (o pdf-min foi mais rápido em 25 dos 32 arquivos, razão mediana 0,74, mas a faixa foi de 0,33 a 1,72). Em arquivos pesados (vetorial, digitalização) o pdf-min foi cerca de 2 vezes mais rápido (332 contra 601 ms de mediana). Só no modo COLD o pdf-parse perde feio (723 contra 34 ms), mas esse não é o modo em que ele é usado hoje.
3. **O motivo real para trocar não é velocidade, é memória e isolamento.** O `pdf-parse` (pdf.js mais `@napi-rs/canvas`) deixa o processo Node com 328 MB de RSS logo após importar, chega a 674 MB (mediana) a 1600 px, e uma "bomba de imagem" (imagem 40000 x 40000 comprimida em 1,6 MB) fez o processo passar de 4,1 GB e ser morto. O `pdf-min` devolveu a mesma entrada em 0,03 s com uma página branca e um aviso de diagnóstico. Se o serviço aceita PDFs de qualquer usuário, isole a geração (processo filho com limite de memória e timeout) em vez de rodar o pdf.js no processo principal.
4. **Melhor caminho incremental:** manter o `pdf-parse` por ora, mas (i) passar `desiredWidth` (o código atual usa a escala padrão 1, então um PDF A0 gera um PNG de 2384 px de largura e um A4 gera 595 px), (ii) rodar em um processo filho ou worker com limite de memória e timeout de 30 s, (iii) limitar o tamanho de entrada. Se quiser sair do pdf.js, use o **binário `pdf-min` separado** (5,53 MB, `spawn` com stdin e stdout) no lugar de `pdf-parse`: mesma qualidade visual, RSS de 10 a 31 MB no filho (contra 328 a 674 MB do processo Node com o pdf.js) e limite natural de memória por `ulimit`. Se o objetivo é um binário só para operação, reavalie a unificação quando o craft-libs alinhar o Hayro ao `vello_cpu` 0.2 e os três motores subirem para uma revisão com `raster`.
5. **Licenças:** `pdftoppm` (Poppler, GPL) e `mutool` (MuPDF, AGPL) são só referência de velocidade aqui; os pacotes são copyleft (informação geral, não verificada nesta sessão), então confira antes de distribuir junto com o serviço. `pdf-min` herda MIT ou Apache-2.0 do craft-libs e do Hayro (Foxit, BSD-3, nas fontes padrão 14 embutidas).

## 3. Método

### Candidatos e como foram chamados

| Candidato | Versão | Chamada (primeira página, largura igual para todos) |
|---|---|---|
| `pdf-parse` | 2.4.5 (npm, a mais recente na data; dependências: `pdfjs-dist` 5.4.296 e `@napi-rs/canvas` 0.1.80 com o pacote nativo `linux-x64-gnu`; 4 pacotes instalados) | `new PDFParse({ data: buf })`, `getScreenshot({ first: 1, desiredWidth: W })`, `destroy()`. Opções de tamanho que a versão suporta: `scale` (padrão 1; 1 ponto do PDF vira 1 px) e `desiredWidth` (tem precedência sobre `scale`). Usei `desiredWidth`. O código original do usuário usa só `first: 1`, ou seja, escala 1 |
| `pdf-min` | `craft-pdf` no rev `ae2f796b673f29cf3161f36eb6a8933f0a3fe028`, feature `raster`, mais os 4 patches do Hayro repetidos | `pdf-min ENTRADA SAIDA.png --width W` (também `-` para stdin e stdout) |
| `pdftoppm` | poppler-utils 24.02.0 | `pdftoppm -png -f 1 -l 1 -scale-to-x W -scale-to-y -1 -singlefile ENTRADA SAIDA` |
| `mutool` | MuPDF 1.23.10 (apt, `mupdf-tools`) | `mutool draw -q -o SAIDA.png -w W -F png ENTRADA 1` |
| Linha de base | Node 22.22.0 | `node empty.ts` (processo vazio, para descontar o start do Node no COLD do pdf-parse) |

Não medi o LibreOffice (`soffice --convert-to png`) a partir de PDF: o item era opcional e o custo de partida do `soffice` já é conhecido de `RESULTS.md` (centenas de ms a segundos). Isso não foi medido nesta rodada.

O `pdf-min` é o `gp-min` do Caminho C do GridCraft sem o motor de planilha: `craft_pdf::open_slice(&bytes, None)`, escala `W / largura_da_página`, `raster::render(page, &RenderCache::new(), &Default::default(), &Options { scale, background: [255; 4], .. })`, achatamento sobre branco e codificação PNG RGB com o crate `png`. Perfil `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`. Dependências: `craft-pdf` (git, rev fixo, `default-features = false`, `features = ["raster"]`), `png` 0.18, e o bloco `[patch.crates-io]` para `hayro`, `hayro-interpret`, `hayro-syntax` e `hayro-jbig2` no mesmo rev (o cargo ignora `[patch]` de dependências, então a raiz precisa repetir). Compila em 2 min 14 s nesta máquina.

### Cenários

- **COLD**: processo novo por arquivo, relógio total (inclui o start do Node e o carregamento dos módulos para o pdf-parse). 5 repetições por arquivo e candidato, intercaladas entre os candidatos, 32 PDFs válidos (30 de `valido` mais 2 de `escaneado`; o PDF de zero páginas fica fora), larguras 600 px e 1200 px; e uma rodada a 1600 px em 10 arquivos representativos. Cada execução passou por `/usr/bin/time` (tempo de CPU de usuário e sistema e pico de RSS do filho; o tempo de CPU tem resolução de 10 ms).
- **WARM**: um processo Node novo por arquivo (simula o serviço), 3 chamadas de aquecimento e depois 20 medidas. `pdf-parse` roda em processo (`new PDFParse` e `destroy` a cada chamada, como no código do usuário). Os binários rodam por `child_process.spawn` a partir do mesmo processo: `pdf-min` em dois modos (stdin e stdout, sem arquivo; e arquivo temporário: grava os bytes, roda, lê o PNG de volta), `pdftoppm` e `mutool` por arquivo temporário. 32 arquivos a 600 px; 10 arquivos a 1200 e 1600 px. Memória: `process.memoryUsage()` antes (depois do import), depois do aquecimento e no fim; pico por `resourceUsage().maxRSS`.
- **Robustez**: 47 PDFs (os 45 do corpus mais os 2 grandes), 1 execução por candidato a 600 px, timeout de 30 s com `SIGKILL`, `ulimit -v 4000000` (4 GB) para os binários nativos (o V8 reserva endereços demais para isso, então o Node roda sem esse limite, com `--max-old-space-size=2048` nas bombas). Mais 3 PDFs hostis gerados à parte (bomba de fluxo, bomba de imagem, página de 14400 pt).
- **Qualidade**: 24 PDFs montados lado a lado nos 4 candidatos (6 montagens em `baselines/pdf/`), vistos um a um.

### Corpus

`corpus/pdf/` tem 45 PDFs pequenos (2,9 MB no total) e o `MANIFEST.tsv` (arquivo, caso, categoria, páginas, bytes, observação), que lista também 2 PDFs grandes que **não** estão no git (receita na seção 10). Origem: LibreOffice 24.2.7 a partir dos corpora de Word, Excel e PowerPoint (acentos, tabelas, imagens, gráficos, gradientes, sombras, 60 slides, texto não latino e emoji), mais PDFs escritos à mão (fontes não embutidas, transparência e gradientes, formulário AcroForm, página vazia, zero páginas, página A0, vetorial pesado de 12.000 curvas), derivados do qpdf (rotação, 3 criptografias, tamanhos mistos) e do ImageMagick (digitalizados só de imagem), e arquivos quebrados (truncados, corrompidos, extensão errada). Corpus gerado e pequeno: ele representa bem recursos do PDF, não a variedade de PDFs reais de usuários.

### Ambiente

Ubuntu 24.04, Linux 6.18.44, Xeon 2,80 GHz com **2 vCPUs**, 8 GB, Node 22.22.0, rustc 1.97.0. **Carga durante as medições** (`uptime`, `nproc` = 2): média de carga de 1 minuto entre 2,0 e 3,1 (mediana 2,2 no COLD, 3,0 no fim do WARM), porque outros agentes compilavam com `rustc` ocupando as duas CPUs. Ou seja, havia competição pelas CPUs o tempo todo: os valores absolutos de tempo estão inflados em relação a uma máquina ociosa, em uma proporção que não medi (o tempo de CPU dos filhos, que sofre menos, está nas tabelas).

## 4. Resultados de desempenho

### COLD (processo novo por arquivo)

Largura 600 px (32 arquivos, 160 amostras por candidato). "Mediana" é a mediana das medianas por arquivo; "p95" é o percentil 95 de todas as amostras (dominado pelos 3 arquivos pesados); "disp." é a amplitude interquartil sobre a mediana de todas as amostras.

| Candidato | Mediana (ms) | p95 (ms) | Mín e máx (ms) | Disp. | CPU mediana (ms) | Pico RSS mediano (MB) | PNG mediano (KB) |
|---|---|---|---|---|---|---|---|
| node vazio (linha de base) | 100,7 | 130,6 | 85,8 a 258,9 | 16% | 80 | 59,4 | n/d |
| pdf-parse | 723,3 | 1.201,7 | 561,4 a 1.639,2 | 26% | 685 | 324,8 | 24,6 |
| pdf-min | 33,6 | 321,2 | 16,8 a 423,4 | 67% | 20 | 9,9 | 20,8 |
| pdftoppm | 67,4 | 838,0 | 33,7 a 1.167,8 | 74% | 50 | 17,8 | 19,4 |
| mutool | 22,5 | 102,0 | 14,1 a 518,6 | 39% | 10 | 11,8 | 18,8 |

Largura 1200 px (32 arquivos, 160 amostras):

| Candidato | Mediana (ms) | p95 (ms) | Mín e máx (ms) | Disp. | CPU mediana (ms) | Pico RSS mediano (MB) | PNG mediano (KB) |
|---|---|---|---|---|---|---|---|
| node vazio | 104,3 | 136,5 | 87,1 a 766,6 | 19% | 90 | 59,4 | n/d |
| pdf-parse | 769,1 | 1.545,5 | 612,7 a 2.260,8 | 20% | 770 | 335,8 | 59,7 |
| pdf-min | 94,3 | 632,3 | 38,8 a 937,6 | 64% | 80 | 20,0 | 50,0 |
| pdftoppm | 182,8 | 2.643,1 | 75,9 a 3.747,7 | 82% | 160 | 23,8 | 43,9 |
| mutool | 59,5 | 349,6 | 27,6 a 1.166,2 | 48% | 40 | 19,9 | 50,0 |

Largura 1600 px (10 arquivos representativos, 50 amostras; o thumbcraft renderiza grande e reduz depois):

| Candidato | Mediana (ms) | p95 (ms) | Mín e máx (ms) | CPU mediana (ms) | Pico RSS mediano (MB) | PNG mediano (KB) |
|---|---|---|---|---|---|---|
| pdf-parse | 944,9 | 2.342,6 | 735,2 a 2.486,4 | 935 | 350,1 | 282,7 |
| pdf-min | 207,1 | 1.028,3 | 69,7 a 1.082,2 | 200 | 30,8 | 261,5 |
| pdftoppm | 543,0 | 4.909,0 | 137,0 a 5.076,5 | 525 | 29,5 | 255,3 |
| mutool | 112,8 | 1.322,7 | 43,5 a 1.410,5 | 95 | 28,6 | 248,5 |

Leitura: no COLD o `pdf-parse` gasta cerca de 100 ms só no start do Node (linha de base) e o resto em carregar `pdfjs-dist` e `@napi-rs/canvas` (RSS de 325 MB antes de renderizar qualquer coisa). O PNG do pdf-parse é 18% maior (é RGBA em vez de RGB; não verifiquei se o compressor também pesa). A dispersão do pdf-min é alta porque o conjunto mistura arquivos de 17 ms com um de 323 ms (vetorial) e outro de 384 ms (digitalizado colorido).

### WARM (dentro de um serviço Node em execução)

600 px, 32 arquivos, 20 medidas por arquivo depois de 3 de aquecimento. Tempo por chamada (ms): "mediana" é a mediana das medianas por arquivo, "p95" é de todas as medidas. Memória (MB decimais) é a do **processo Node**: antes é logo após o import, depois é no fim das medidas, pico é o `maxRSS` do processo.

| Candidato | Modo | Mediana (ms) | p95 (ms) | Mín e máx (ms) | PNG med. (KB) | RSS antes | RSS depois | Pico RSS | Heap antes e depois |
|---|---|---|---|---|---|---|---|---|---|
| pdf-parse | em processo | 42,2 | 571,7 | 16,0 a 1.140,5 | 24,6 | 328 | 397 | 397 | 13 e 27 |
| pdf-min | spawn, stdin e stdout | 34,0 | 322,7 | 15,5 a 624,5 | 20,8 | 75 | 81 | 81 | 9 e 9 |
| pdf-min | spawn, arquivo temporário | 45,1 | 361,1 | 17,5 a 559,3 | 20,8 | 75 | 81 | 81 | 9 e 10 |
| pdftoppm | spawn, arquivo temporário | 81,8 | 902,7 | 35,1 a 1.551,2 | 19,4 | 75 | 81 | 81 | 9 e 9 |
| mutool | spawn, arquivo temporário | 29,0 | 143,4 | 15,7 a 665,9 | 18,8 | 75 | 81 | 81 | 9 e 9 |

Separando os 29 arquivos "típicos" dos 3 pesados (48 vetorial, 60 e 61 digitalizados):

| Candidato (modo) | Típicos: mediana e p95 (ms) | Pesados: mediana e p95 (ms) |
|---|---|---|
| pdf-parse | 39,9 e 142,1 | 601,1 e 907,6 |
| pdf-min (stdin e stdout) | 33,2 e 77,8 | 331,5 e 425,4 |
| pdf-min (arquivo temporário) | 44,2 e 91,9 | 375,6 e 514,6 |
| pdftoppm | 76,5 e 184,4 | 931,8 e 1.382,1 |
| mutool | 28,5 e 48,7 | 144,1 e 598,2 |

Larguras altas (10 arquivos, 20 medidas cada):

| Largura | Candidato (modo) | Mediana (ms) | p95 (ms) | Pico RSS do processo Node (mediana, MB) | Maior pico (MB) |
|---|---|---|---|---|---|
| 1200 | pdf-parse | 175,3 | 1.045,4 | 540 | 749 |
| 1200 | pdf-min (stdin e stdout) | 150,1 | 767,6 | 82 | n/d |
| 1200 | pdf-min (arquivo temporário) | 189,2 | 774,6 | 81 | n/d |
| 1200 | pdftoppm | 408,4 | 3.712,6 | 81 | n/d |
| 1200 | mutool | 97,8 | 914,8 | 81 | n/d |
| 1600 | pdf-parse | 241,5 | 1.364,7 | 674 | 858 |
| 1600 | pdf-min (stdin e stdout) | 213,5 | 1.179,5 | 84 | n/d |
| 1600 | pdf-min (arquivo temporário) | 208,4 | 1.169,4 | 81 | n/d |
| 1600 | pdftoppm | 558,8 | 5.429,9 | 81 | n/d |
| 1600 | mutool | 113,4 | 1.395,3 | 81 | n/d |

Leitura:

- O custo de `spawn` mais transferência foi pequeno: o piso (página vazia: 22 ms no modo stdin e stdout, 34 ms com arquivo temporário) é praticamente o mesmo do pdf-parse em processo (22 ms). Não houve vantagem decisiva para nenhum dos dois em PDFs típicos, o que bate com o fato de o pdf.js usar o Skia nativo do `@napi-rs/canvas` para desenhar.
- O modo stdin e stdout do `pdf-min` foi cerca de 10 ms mais rápido que o de arquivo temporário (sem gravar e ler o disco).
- Memória: o processo Node que importa o `pdf-parse` tem 328 MB de RSS antes da primeira chamada (253 MB a mais que o Node de base, 75 MB) e continua subindo com a largura (pico mediano de 397 MB a 600 px, 540 MB a 1200 px, 674 MB a 1600 px; maior pico visto: 858 MB). Em chamadas repetidas o RSS cresceu 45 MB a 600 px, 167 MB a 1200 px e 283 MB a 1600 px entre o fim do aquecimento e o fim das 20 medidas (mediana); não investiguei se isso é cache do Skia ou coleta tardia, e não vi o RSS cair. Já os filhos nativos vivem só durante a chamada: pico de RSS do filho de 10 MB (pdf-min a 600 px) a 31 MB (pdf-min a 1600 px); o processo Node pai subiu 6 MB.
- O pdf-min foi mais rápido em 25 dos 32 arquivos contra o pdf-parse em WARM (razão mediana pdf-min sobre pdf-parse: 0,74, faixa de 0,33 a 1,72).
- `mutool` foi o mais rápido em quase tudo (mediana de 29 ms); `pdftoppm` o mais lento nos arquivos com imagens grandes.

### Por arquivo (600 px, mediana em ms)

COLD inclui o start do processo (para o pdf-parse, o do Node, cerca de 100 ms). WARM é dentro do serviço.

| Arquivo | COLD pdf-parse | COLD pdf-min | COLD pdftoppm | COLD mutool | WARM pdf-parse | WARM pdf-min (stdio) | WARM mutool |
|---|---|---|---|---|---|---|---|
| 10_word_simples_acentos | 652.1 | 44.7 | 71.0 | 24.1 | 58.8 | 38.1 | 27.9 |
| 11_word_tabelas | 711.2 | 27.5 | 44.8 | 20.8 | 35.4 | 24.9 | 21.8 |
| 12_word_imagens | 866.5 | 46.3 | 52.6 | 30.9 | 70.4 | 47.9 | 27.3 |
| 13_word_multipagina | 830.8 | 67.8 | 115.7 | 27.0 | 85.7 | 99.0 | 36.2 |
| 14_word_fontes | 718.9 | 29.3 | 60.8 | 22.6 | 48.6 | 30.2 | 20.7 |
| 15_word_estilos_cores | 670.0 | 22.7 | 38.9 | 19.5 | 32.2 | 22.3 | 21.6 |
| 20_excel_condicional_cor | 637.1 | 30.5 | 47.4 | 20.8 | 37.4 | 31.8 | 20.6 |
| 21_excel_grafico_barras | 734.6 | 34.5 | 53.3 | 21.4 | 40.6 | 31.8 | 22.9 |
| 22_excel_grafico_pizza | 658.7 | 34.5 | 52.5 | 21.8 | 39.9 | 33.4 | 29.3 |
| 23_excel_planilha_larga | 666.2 | 48.2 | 75.6 | 21.8 | 55.2 | 58.3 | 28.0 |
| 24_excel_nao_latino_emoji | 727.6 | 31.3 | 52.3 | 22.4 | 91.2 | 30.2 | 30.7 |
| 25_excel_painel_composto | 893.4 | 41.7 | 65.7 | 27.7 | 123.8 | 41.1 | 28.7 |
| 30_ppt_formas_gradiente | 913.6 | 61.0 | 187.2 | 51.6 | 168.8 | 90.0 | 47.5 |
| 31_ppt_sombra_transparencia | 871.1 | 37.8 | 81.1 | 30.4 | 84.6 | 38.5 | 35.9 |
| 32_ppt_grafico_barras | 639.2 | 19.7 | 34.5 | 15.3 | 33.8 | 34.7 | 39.5 |
| 33_ppt_fundo_gradiente | 636.5 | 24.8 | 100.1 | 21.8 | 38.7 | 33.2 | 41.4 |
| 34_ppt_fundo_imagem | 783.4 | 42.9 | 114.4 | 24.7 | 94.8 | 45.0 | 39.0 |
| 35_ppt_rtl_cjk_emoji | 781.5 | 21.4 | 48.7 | 18.0 | 35.1 | 22.3 | 30.3 |
| 36_ppt_4x3 | 648.9 | 20.2 | 41.2 | 15.7 | 27.4 | 31.2 | 27.9 |
| 37_ppt_16x9_paisagem | 609.0 | 17.7 | 40.5 | 16.0 | 27.8 | 22.2 | 26.6 |
| 38_ppt_60_slides | 743.3 | 25.0 | 52.6 | 18.3 | 38.9 | 24.0 | 25.4 |
| 40_texto_nao_latino_arabe_cjk_emoji | 871.0 | 35.2 | 69.1 | 24.2 | 76.2 | 35.1 | 33.8 |
| 41_fontes_nao_embutidas | 626.1 | 31.2 | 90.3 | 26.6 | 44.5 | 30.0 | 30.1 |
| 42_pagina_vazia | 626.9 | 24.0 | 36.7 | 20.2 | 22.3 | 21.8 | 28.5 |
| 44_transparencia_gradientes | 735.6 | 32.3 | 79.8 | 23.9 | 32.7 | 44.5 | 20.2 |
| 45_formulario_acroform | 714.5 | 27.6 | 58.8 | 17.0 | 18.3 | 17.8 | 23.5 |
| 46_pagina_gigante_A0 | 673.1 | 58.1 | 88.6 | 24.8 | 33.6 | 57.7 | 37.1 |
| 47_tamanhos_mistos | 667.9 | 32.8 | 77.3 | 25.3 | 36.1 | 32.5 | 36.4 |
| 48_vetorial_pesado_mapa | 1620.5 | 323.2 | 546.4 | 483.9 | 664.7 | 331.5 | 544.5 |
| 49_pagina_rotacionada_90 | 756.5 | 52.2 | 100.7 | 18.6 | 43.8 | 53.1 | 27.3 |
| 60_escaneado_so_imagem | 949.8 | 206.5 | 846.5 | 97.6 | 254.2 | 227.7 | 144.1 |
| 61_escaneado_cor_300dpi_4pag | 1232.9 | 384.2 | 1120.1 | 83.9 | 601.1 | 375.3 | 112.8 |

## 5. Robustez

Taxa de sucesso por categoria (gera um PNG de 600 px válido; 1 execução por candidato e arquivo, timeout de 30 s). O sucesso aqui é "devolveu imagem", não "a imagem está correta" (veja a nota sobre os arquivos em branco abaixo).

| Categoria (arquivos) | pdf-parse | pdf-min | pdftoppm | mutool |
|---|---|---|---|---|
| Válidos (30, inclui página vazia, A0, formulário, rotacionado, fontes não embutidas, texto não latino) | 30 de 30 | 30 de 30 | 30 de 30 | 30 de 30 |
| Digitalizados só de imagem (2) | 2 de 2 | 2 de 2 | 2 de 2 | 2 de 2 |
| Grandes (2: 1126 páginas, 33 MB) | 2 de 2 | 2 de 2 | 2 de 2 | 2 de 2 |
| Criptografado sem senha de abertura (2: AES-128 e RC4 40 bits) | 2 de 2 | 2 de 2 | 2 de 2 | 2 de 2 |
| Criptografado com senha de abertura (1: AES-256) | 0 de 1 | 0 de 1 | 0 de 1 | 0 de 1 |
| Truncado (2: 50% e 90% do arquivo) | 0 de 2 | 0 de 2 | 0 de 2 | 0 de 2 |
| Corrompido (4) | 3 de 4 | 3 de 4 | 3 de 4 | 3 de 4 |
| Extensão errada (3: Markdown, HTML, JPEG com extensão .pdf) | 0 de 3 | 0 de 3 | 0 de 3 | 1 de 3 (renderiza o JPEG) |
| Zero páginas (1) | 0 de 1 | 0 de 1 | 0 de 1 | 0 de 1 |

Tipos de erro (medidos):

| Caso | pdf-parse | pdf-min | pdftoppm | mutool |
|---|---|---|---|---|
| Senha de abertura | `PasswordException: No password given` | `Decryption(PasswordProtected)`, saída 3 | `Incorrect password` | `cannot draw` |
| Truncado | `InvalidPDFException: Invalid PDF structure.` | `parser rejected the document: Invalid` | `Couldn't read xref table` | `cannot draw` |
| Arquivo vazio | `The PDF file is empty` | `Invalid` | `Document stream is empty` | `cannot draw` |
| Extensão errada (texto ou HTML) | `Invalid PDF structure.` | `Invalid` | `Couldn't read xref table` | `cannot draw` |
| Zero páginas | o código do usuário lança "Não foi possível extrair a imagem da primeira página" | `PDF sem páginas` (nossa mensagem) | `Wrong page range given` | `cannot draw` |
| Corrompido, xref ou 64 bytes no meio | renderiza | renderiza | renderiza (avisos) | renderiza |
| Lixo de 1 KB antes do cabeçalho `%PDF-` | renderiza | renderiza | renderiza | renderiza |

Observações:

- Os quatro se comportam igual nas categorias: nenhum conseguiu abrir os truncados (50% e 90%) nem o PDF de zero páginas. O qpdf reconstrói a xref desses arquivos, então o ganho de uma etapa de reparo é possível, mas não foi testado.
- Nenhum travou nos PDFs do corpus: o maior tempo foi de 1,9 s (pdf-parse no vetorial pesado; 1,6 s na digitalização de 33 MB). Nenhum estourou o timeout de 30 s.
- **Páginas em branco enganosas:** uma página vazia e uma entrada fora do limite do Hayro viram "sucesso" com PNG branco. O `pdf-min` imprime `Diagnostics { image_decode_failed, content_truncated }` em stderr e sai com 0; um serviço deve tratar esses avisos como falha ou aceitar uma miniatura em branco.
- Com `panic = "abort"` (perfil de tamanho) o `catch_unwind` dentro de `craft_pdf::open` não faz efeito (leitura do código, não provocado por teste): um pânico do parser derruba o processo em vez de virar `OpenError::BackendPanic`. Em um binário separado isso fica isolado; em um binário unificado derrubaria o processo todo (como já vale para os outros motores).

### Bombas e limites (3 PDFs hostis, gerados à parte, não estão no git)

Timeout de 30 s; `ulimit -v` de 4 GB nos nativos; Node com `--max-old-space-size=2048` (limita o heap do V8, não a memória nativa do canvas).

| Caso | pdf-parse | pdf-min | pdftoppm | mutool |
|---|---|---|---|---|
| Bomba de fluxo (conteúdo da página: 512 MB de espaços em 0,52 MB Flate) | ok, 5,8 s, 1.125 MB de RSS | ok, 1,5 s, 10 MB (branco, `content_truncated`) | ok, 4,3 s, 17 MB | ok, 2,6 s, 11 MB |
| Bomba de imagem (cinza 40000 x 40000, 1,6 GB descomprimidos em 1,56 MB Flate) | **morto (137) aos 11 s com 4.113 MB de RSS** | ok, 0,03 s, 12 MB (branco, `image_decode_failed`) | ok, 12,4 s, 18 MB | ok, 4,1 s, 14 MB |
| Página de 14400 x 14400 pt (200 polegadas) a 600 px | ok, 1,4 s, 317 MB | ok, 0,02 s, 8 MB | ok, 0,04 s, 16 MB | ok, 0,02 s, 10 MB |

O pdf.js não tem limite de memória utilizável contra a bomba de imagem e matou o processo. Os outros três limitam por conta própria (o Hayro tem orçamentos de decodificação e de pixels: `MAX_PIXELS` de 64 milhões, entrada de até 512 MiB).

### PDFs grandes (1200 px, 1 execução por candidato; tempo de relógio e pico de RSS por `/usr/bin/time`)

| Arquivo | pdf-parse | pdf-min | pdftoppm | mutool |
|---|---|---|---|---|
| 1126 páginas de texto (6,05 MB) | 930 ms, 361 MB | 270 ms, 37 MB | 460 ms, 25 MB | 110 ms, 27 MB |
| 32 páginas digitalizadas (33,18 MB) | 1.780 ms, 532 MB | 860 ms, 136 MB | 690 ms, 30 MB | 270 ms, 33 MB |

Os quatro renderizam só a primeira página. O pico do pdf-min (136 MB para um PDF de 33 MB) mostra que ele carrega o arquivo e as imagens da página inteiros, enquanto `pdftoppm` e `mutool` ficaram em 30 e 33 MB.

## 6. Qualidade visual

Vistas lado a lado, 24 PDFs, colunas pdf-parse, pdf-min, pdftoppm, mutool (montagens em `baselines/pdf/comparacao_q1_word.png` a `comparacao_q6_quebrados.png`; tiles de falha em vermelho com "FALHOU"; o fundo cinza azulado é só a margem do tile e revela transparência: nenhuma imagem mostrou transparência).

- **Os quatro são visualmente equivalentes** em Word (acentos, tabelas, imagens PNG e JPEG, fontes), Excel (gráficos de barras, pizza, painel, planilha larga de 130 colunas), PowerPoint (gradientes, sombras, transparência, fundo com imagem, árabe, hebraico, CJK, emoji), formulário (campos, caixa marcada e botão aparecem nos quatro), página A0, tamanhos mistos (a primeira página pequena, 216 x 360 pt), vetorial pesado, rotação de 90 graus (todos giram igual), digitalizados e arquivo corrompido recuperável. Nenhum cortou texto.
- **Fundo:** o `pdf-parse` devolve um PNG RGBA, mas **opaco e branco** (alfa mínimo 1 nos arquivos testados); o `pdf-min`, `pdftoppm` e `mutool` devolvem RGB. Não há fundo transparente em nenhum. Um PDF com fundo próprio (gradiente, imagem) aparece com ele.
- **Fontes não embutidas (arquivo 41):** a única diferença visível. O `pdf-parse` cai em uma fonte serifada para Helvetica, Arial e Calibri (e usa um Courier mais claro); o `pdf-min` e o `mutool` usam equivalentes sem serifa (fontes padrão 14 Foxit embutidas no Hayro, ou as do MuPDF), e o `pdftoppm` usa fontes do sistema. Para texto em miniatura é pouco relevante, mas muda a aparência (medido por inspeção visual, não por métrica).
- **Texto não latino e emoji:** árabe, chinês, japonês, coreano e emoji colorido saíram nos quatro (as fontes Noto estavam instaladas na máquina; com fontes ausentes o resultado pode mudar, o que não foi testado). Em PDFs com fontes embutidas isso não depende do sistema.
- **Gradientes e transparência** (axial, radial, alfa 50%, modo Multiply): idênticos nos quatro.
- **Digitalizado de 150 dpi:** o `pdf-parse` suaviza um pouco mais o texto reduzido; os demais ficam mais nítidos (inspeção visual).
- Fração de pixels brancos por arquivo (medida com ImageMagick, limiar 99%): diferença máxima entre candidatos menor que 1,7 ponto percentual nos 30 PDFs válidos (a maior foi no gráfico de barras do PowerPoint, 85,5% contra 87,1%), o que concorda com a inspeção visual.

## 7. Custo de PDF no binário unificado (estimado)

Base: `pdf-min` medido, 5.527.304 bytes (5,53 MB, 5,27 MiB), com o mesmo perfil do thumbcraft. `cargo bloat --crates` em uma compilação do mesmo código sem `strip` (8,28 MB): `.text` de 3,08 MB, `.rodata` 1,11 MB, `.eh_frame` 0,30 MB, `.data.rel.ro` 0,44 MB. O thumbcraft atual tem 26.992.128 bytes (26,99 MB).

Código (`.text`) por crate no `pdf-min` (em KB decimais, valores do `cargo bloat`): `vello_cpu` 849, `hayro_interpret` 319, `std` 316, `hayro_syntax` 231, `skrifa` 225, `hayro_jbig2` 201, `moxcms` 163, `hayro_jpeg2000` 155, `hayro` 106, `vello_common` 97, `pic_scale` 82, `read_fonts` 48, `zune_jpeg` 34, `pxfm` 34, `kurbo` 31, `hayro_cmap` 27, `miniz_oxide` 27, `pdf_min` 26, `brotli_decompressor` 21, e o resto abaixo de 10 cada.

**O que seria deduplicado** (o thumbcraft já compila estes, conferido com `nm` no binário thumbcraft sem `strip`): `std` (316), `kurbo` 0.13.1, `peniko` 0.6.1, `fearless_simd` 0.4.1, `png`, `flate2` e `miniz_oxide`, `zune_jpeg` (via `image`), `read_fonts` e `skrifa` 0.42.1 (o `krilla` do Word já traz essa versão). Soma de código deduplicável: cerca de 0,6 MB (0,5 a 0,85 MB conforme a parte do `skrifa` que casa).

**O que NÃO seria deduplicado:**

- **`vello_cpu` 0.0.8, `vello_common` 0.0.8 e 0.0.9 e `glifo` 0.1.1.** O `hayro` (com patch, do craft-libs) exige `vello_cpu = "0.0.8"`, enquanto o thumbcraft usa `vello_cpu` 0.2.0 (`vello_common` 0.2.0, `glifo` 0.3.0). São versões incompatíveis pelo semver: o binário traria **duas cópias** do rasterizador, cerca de 0,95 MB de código só do `vello_cpu` e `vello_common` do Hayro. Este é o ponto em que a premissa "vello_cpu 0.2.0 compartilhado" **não se confirma** para o PDF.
- `hayro_interpret`, `hayro_syntax`, `hayro_jbig2`, `hayro_jpeg2000`, `hayro`, `hayro_cmap`, `hayro_postscript`, `hayro-ccitt` (cerca de 1,05 MB), `pic_scale` (82 KB), `moxcms` (163 KB) e `pxfm` (34 KB), que não aparecem no thumbcraft (`nm` deu 0 símbolos para `moxcms`, `pxfm` e `pic_scale`).
- Dados: as 14 fontes Foxit da família padrão 14 e um perfil ICC (os `assets` do `hayro-interpret` somam 281.777 bytes, 0,28 MB, pela feature `embed-fonts`, ligada por padrão), mais as tabelas de CMap e de decodificação: a maior parte dos cerca de 1,1 MB de `.rodata`.

**Estimativa de custo incremental: cerca de 4,2 MB** (faixa de 3,9 a 4,6 MB), isto é, de 26,99 MB para cerca de 31,2 MB (+16%). Duas contas deram o intervalo: por baixo (somar só o que não é compartilhado: 2,46 MB de `.text`, 1,05 MB de `.rodata`, 0,35 MB de `.data.rel.ro` e pouco mais) dá cerca de 3,9 MB; por cima (5,53 MB menos 0,6 MB de código compartilhado e menos 0,3 MB do `.eh_frame` da `std`) dá cerca de 4,6 MB. Não foi medido dentro do thumbcraft. Se o Hayro do craft-libs fosse portado para o `vello_cpu` 0.2.0 (trabalho de upstream, sem estimativa de esforço), o custo cairia para cerca de 3,2 MB.

**Como o construtor pode medir de verdade:** adicionar ao `crates/thumbcraft` uma feature `pdf` com `craft-pdf` (features `raster`) renomeado (`package = "craft-pdf"`, rev `ae2f796`), os 4 patches do Hayro no `Cargo.toml` raiz, uma rota para `Engine::Pdf` que chame `open_slice`, `raster::render` e achate sobre branco, compilar com o mesmo perfil e comparar o tamanho com 26.992.128 bytes; `cargo bloat --release -p thumbcraft --crates` (sem `strip`) mostra as duas cópias de `vello_cpu`.

## 8. Revisões do craft-libs e conflitos

O WordCraft e o DeckCraft pinam o `craft-pdf` na revisão `c3487320`, que **só tem o escritor** (features `write`, com `krilla`). As features `read`, `interpret` e `raster` só existem a partir do commit `c07b3fa` do `main`; o `pdf-min` usa `ae2f796`. Portanto incluir PDF exige **uma segunda revisão** do craft-libs no mesmo binário, ou subir a revisão dos três motores.

Teste feito (resolução e `cargo check`, não execução): um crate de teste com as duas revisões lado a lado, `craft-pdf-write` (`c3487320`, `write`) e `craft-pdf-raster` (`ae2f796`, `raster`), com `[patch.crates-io]` do `krilla` no `c3487320` e dos quatro `hayro*` no `ae2f796`. O cargo resolveu os dois `craft-pdf` como pacotes distintos (um `krilla` 0.8.2 só do lado antigo, o Hayro só do lado novo) e o `cargo check --release` terminou sem erro em 1 min 13 s. Ou seja, não houve conflito de `krilla` nem de Hayro nesse teste mínimo.

O que isso **não** prova: a compilação dentro do workspace completo do thumbcraft, o `patch` do Hayro afetando os `dev-dependencies` do Word e do Deck (testes de `wordcraft-pdf` e `deckcraft-pdf` pedem `hayro-syntax` 0.7.2 e `hayro-interpret` 0.7.0; o patch usa as mesmas versões), e o risco de subir a revisão dos três motores (não testado: o `krilla` do `main` pode ter mudado). O `docs/CRAFT-FAMILY.md` já registra que "não foi testado se subir a revisão no WordCraft e no DeckCraft quebra o `krilla`".

## 9. Limitações do experimento

- **Máquina compartilhada de 2 vCPUs**, com carga média de 2 a 3 durante todas as medições (outros agentes compilando). Os tempos são ruidosos e, em termos absolutos, inflados; as comparações entre candidatos são mais confiáveis do que os números isolados porque as repetições COLD foram intercaladas entre candidatos, mas diferenças de 10 a 20% (por exemplo 34 contra 42 ms no WARM) estão dentro do ruído. A medida de CPU (`/usr/bin/time`) tem resolução de 10 ms.
- **Cada arquivo do WARM foi medido em um processo Node novo, uma vez** (20 medidas por processo, não várias execuções do processo). A dispersão entre processos não foi medida.
- **Corpus gerado:** PDFs do LibreOffice, do qpdf e escritos à mão, mais ruído sintético nos digitalizados. Não inclui PDFs reais de usuários (escaneados de verdade, JBIG2, JPEG 2000, PDFs de impressoras, formulários XFA, PDFs com camadas e anotações complexas), onde as diferenças entre motores costumam aparecer. Os PDFs grandes são sintéticos (texto repetido; ruído).
- **Uma execução por candidato na robustez**, sem repetição; `ulimit -v` só nos nativos.
- **Versões:** pdf-parse 2.4.5 (pdfjs-dist 5.4.296, canvas 0.1.80), poppler 24.02.0, MuPDF 1.23.10, Node 22.22.0, rustc 1.97.0, craft-libs `ae2f796`. Outras versões podem mudar os números.
- O tamanho estimado do `pdf-min` dentro do thumbcraft é estimativa (seção 7). O LibreOffice como referência de lentidão não foi medido. A qualidade foi avaliada por inspeção visual (e fração de pixels brancos), sem métrica perceptual.
- O código do benchmark (`bench.ts`, `pdfparse-one.ts`, o crate `pdf-min`, os scripts de geração) ficou **fora do repositório** por decisão da coordenação, no diretório de trabalho da sessão (`/tmp/claude-0/-home-claude/37e359fe-23e0-5bd2-965b-70005e330799/scratchpad/pdf-bench-work`), que é temporário. Esta página e o corpus descrevem o suficiente para refazer.

## 10. Como refazer

Ferramentas: Node 22.18 ou mais novo (executa `.ts` direto), `pdf-parse@2.4.5` fixo, rustc 1.95 ou mais novo, LibreOffice, `qpdf`, ImageMagick 6, `poppler-utils`, `mupdf-tools`, `time`.

- **Corpus pequeno:** os PDFs estão em `corpus/pdf/`. A coluna "observação" do `MANIFEST.tsv` diz como cada um foi feito. Receitas curtas: conversões do Word, Excel e PowerPoint por `soffice --headless --convert-to pdf`; rotação por `qpdf --rotate=+90:1`; criptografia por `qpdf --encrypt abrir123 dono456 256 -- ...` (AES-256), `--allow-weak-crypto --encrypt "" dono456 128` (AES-128) e `--allow-weak-crypto --encrypt "" dono456 40` (RC4); tamanhos mistos por `qpdf --empty --pages a.pdf 1 b.pdf 1 c.pdf 1 --`; truncado por `head -c`; corrompido por 64 bytes `Z` com `dd conv=notrunc` no terço do arquivo; digitalizado por `pdftoppm -r 150`, `convert -colorspace Gray -rotate 0.6 -attenuate 0.35 +noise Gaussian -blur 0x0.6 -quality 70` e `convert imagem.jpg -density 150 saida.pdf`. Os PDFs escritos à mão (fontes não embutidas, transparência, formulário, A0, vetorial) são PDFs mínimos cuja `xref` foi refeita pelo `qpdf` (o vetorial foi gerado por um `awk` com semente 42: 12.000 curvas de Bézier).
- **PDFs grandes (não estão no git):**
  - 1126 páginas, cerca de 6,05 MB: repetir 60 vezes o `17_txt_longo_multipagina.txt` do corpus Word em um `.txt` e rodar `soffice --headless --convert-to pdf`.
  - 32 páginas digitalizadas, cerca de 33,18 MB: gerar 8 JPEGs distintos com `convert -size 2480x3508 plasma:fractal -attenuate 0.6 +noise Gaussian -quality 65 pN.jpg` e juntá-los 4 vezes (32 páginas) com `convert p1.jpg ... p8.jpg ... -density 300 saida.pdf`. Com qualidade 92 o arquivo passa de 230 MB, com qualidade 30 fica em 12 MB.
- **Bombas:** um script em Node gera, com `zlib.createDeflate` em fluxo, 512 MB de zeros como conteúdo da página, uma imagem cinza de 40000 x 40000 e uma página `MediaBox [0 0 14400 14400]`; depois `qpdf` reconstrói a `xref`.
- **`pdf-min`:** crate autônomo (`[workspace]` vazio) com `craft-pdf` no rev `ae2f796b673f29cf3161f36eb6a8933f0a3fe028`, `default-features = false`, `features = ["raster"]`, `png = "0.18"` e o `[patch.crates-io]` de `hayro`, `hayro-interpret`, `hayro-syntax` e `hayro-jbig2` apontando para o mesmo git e rev; perfil de release com `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`.
- **Medir:** COLD com `/usr/bin/time -f '%e %U %S %M'` por execução; WARM com `child_process.spawn` e `performance.now()`; memória com `process.memoryUsage()` e `process.resourceUsage().maxRSS`; robustez com `timeout -s KILL 30` e `ulimit -v 4000000`.

## 11. O que foi medido e o que foi estimado

| Item | Situação |
|---|---|
| Tempos COLD, WARM, memória, robustez, bombas, tamanhos de binário, `node_modules` | medido |
| Qualidade visual | observado (inspeção de 24 PDFs) |
| Ausência de leitor de PDF nos binários atuais | medido (`Cargo.lock`, `hayro` ausente nos binários, rota do thumbcraft) |
| PdfCraft como app separado | lido (README do GridCraft e do DeckCraft, `ATTRIBUTION.md` do craft-libs, página do repositório) |
| Coexistência das duas revisões do craft-libs | testado só por resolução e `cargo check` em um crate mínimo |
| Custo de PDF no thumbcraft (cerca de 4,2 MB) e o cenário com `vello_cpu` 0.2 (3,2 MB) | estimado |
| Licenças de Poppler e MuPDF | conhecimento geral, não verificado |
