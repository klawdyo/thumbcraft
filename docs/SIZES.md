# Tamanhos do binário

Unidades: MB decimais (1 MB = 1.000.000 bytes), com MiB entre parênteses quando relevante.
Todas as medidas: rustc 1.97.0, perfil de release `opt-level = "z"`, `lto = "fat"`,
`codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`, Linux x86_64.

## Tabela de 2026-10-19

| Item | Bytes | MB | MiB |
|---|---|---|---|
| Mini Word (wc-min) | 19.763.984 | 19,76 | 18,85 |
| Mini PowerPoint (dc-min) | 6.673.360 | 6,67 | 6,36 |
| Mini Excel (gc-min, com `--crop`, PNG) | 8.407.856 | 8,41 | 8,02 |
| **Soma dos três minis** | **34.845.200** | **34,85** | **33,23** |
| Combinado da análise anterior (Word + PPT + Excel PDF, sem PNG de Excel) | 26.600.832 | 26,60 | 25,37 |
| **thumbcraft (PNG, JPEG e WebP, 3 motores)** | **26.992.128** | **26,99** | **25,74** |
| thumbcraft só com PNG (sem as features `jpeg` e `webp`) | 26.958.048 | 26,96 | 25,71 |

- Economia contra a soma dos minis: 7.853.072 bytes (7,85 MB), 22,5%.
- Contra o combinado anterior, o thumbcraft é 391.296 bytes maior (+1,5%). O combinado anterior
  tinha, no lugar do Excel, só um exportador de PDF (sem PNG, sem a pilha de rasterização do
  GridCraft); o thumbcraft traz o exportador de PNG de verdade (crate `gridcraft-raster`, 12 fontes
  TTF Arimo, Tinos e Cousine) e a CLI.
- Compactado: `xz -9e` leva o binário a 8.605.508 bytes (8,61 MB); `gzip -9` a 12.329.076 bytes.
  Útil se o binário for distribuído compactado; em disco continua sendo 26,99 MB.
- O binário é o mesmo se `RUSTFLAGS="-C force-unwind-tables=no"` é usado (experimento: 26.992.128
  bytes nos dois casos). Com `panic = "abort"` o rustc já omite as tabelas do nosso código; as
  2,2 MB de `.eh_frame` vêm da `std` pré-compilada (só um `-Zbuild-std` em nightly as removeria;
  não testado).

## Custo dos formatos de saída

| Formato | Custo no binário | Observação |
|---|---|---|
| PNG | referência | `png` já está na árvore (vello, image) |
| JPEG | juntos: +34.080 bytes (0,034 MB) | `image` com `jpeg` já é compilado pelos motores; o codificador vem a mais |
| WebP (sem perdas) | juntos com o JPEG | `image-webp` já está na árvore |

Cada um custa bem menos que 0,3 MB, então os dois ficam ligados (features `jpeg` e `webp`,
padrão). O WebP é sem perdas (o `image` 0.25 não tem codificador com perdas). Exemplo, página de
documento em 480x480 contain: PNG 42.923 bytes, WebP sem perdas 28.524 bytes, JPEG (qualidade 85)
21.539 bytes. Para compilar sem eles: `cargo build --release -p thumbcraft --no-default-features`.

## Decisão do redimensionamento

Duas opções foram avaliadas: o `imageops::resize` do crate `image` (já na árvore, 435 KiB de
`.text` no total, mas o código de reamostragem só entraria ao ser usado) e um reamostrador próprio
no `thumbcraft-core` (sem dependências). Escolhi o próprio, porque o do `image` não aceita janela de
origem fracionária limitada a uma região (necessária para cover com gravity e crop sem vazar pixels
vizinhos) e opera em alfa direto, enquanto os motores entregam alfa pré-multiplicado. O custo do
módulo próprio no binário é pequeno: o crate `thumbcraft` inteiro (CLI, adaptadores, núcleo) soma
cerca de 52 KiB de `.text` no cargo-bloat. A versão com `image::imageops` não foi construída, então
a comparação em bytes entre as duas não foi medida.

Filtros oferecidos: Lanczos3 (padrão) e média de área exata (`--filter area`). Medidas em uma
página de documento de uma página A4, média de 5 execuções do processo inteiro (inclui abrir e
renderizar; a máquina estava com 2 vCPUs compartilhadas e outro build rodando, então os tempos são
indicativos):

| Alvo (contain) | Lanczos3 | Área | PNG Lanczos3 | PNG área |
|---|---|---|---|---|
| 120x120 | 332 ms | 236 ms | 7.927 B | 7.789 B |
| 480x480 | 480 ms | 292 ms | 119.174 B | 97.646 B |
| 1200x1200 | 1.348 ms | 600 ms | 502.875 B | 418.952 B |

Lanczos3 é mais nítido em texto pequeno mas custa mais tempo e gera PNG até 20% maior; a média de
área é a opção rápida. Padrão proposto: Lanczos3 (a confirmar).

## Quebra por crate

`cargo bloat --release -p thumbcraft --crates`, numa compilação sem `strip` (34,77 MB) do mesmo
código (ainda sem JPEG e WebP). Valores de `.text`; a seção `.text` tem 15,2 MiB (15.932.919
bytes), `.rodata` 7,15 MB, `.eh_frame` 2,19 MB, `.data.rel.ro` 0,64 MB. As porcentagens do bloat
são estimativas.

| Crate | `.text` | Observação |
|---|---|---|
| wordcraft_engine | 2,4 MiB | inclui comandos de edição, exportação docx e PDF, que a miniatura não usa |
| gridcraft_engine | 1,6 MiB | idem (escrita de xlsx, funções de planilha, análises) |
| vello_cpu | 1,1 MiB | rasterizador, compartilhado pelos três (uma cópia) |
| std | 929 KiB | |
| wordcraft_docx | 870 KiB | leitor e escritor de docx |
| deckcraft_format | 606 KiB | formato nativo do DeckCraft (não usado para pptx) |
| skrifa | 586 KiB | duas versões somadas (ver abaixo) |
| wordcraft_layout | 483 KiB | |
| gridcraft_xlsx | 455 KiB | leitor e escritor de xlsx |
| image | 435 KiB | decodificadores de várias imagens |
| wordcraft_formats | 393 KiB | html, md, tex, rtf, txt |
| wordcraft_doc | 377 KiB | |
| gridcraft_functions | 372 KiB | funções de fórmulas |
| harfrust | 333 KiB | shaping |
| wordcraft_proof | 284 KiB | ortografia e hifenização |
| deckcraft_render | 266 KiB | |
| krilla | 264 KiB | escrita de PDF (puxada pelo WordCraft) |
| wordcraft_docbin | 238 KiB | leitura de .doc |
| regex_automata, regex_syntax, aho_corasick | 438 KiB | |
| thumbcraft (CLI, adaptadores, núcleo) | 52 KiB | |

Os dados embutidos (`.rodata`, 7,15 MB) são principalmente as 13 fontes TTF do WordCraft
(4.322.916 bytes em `assets/fonts`), os dicionários de hifenização (607.002 bytes em
`assets/hyphenation`, quase tudo `en-us.dic`) e as 12 fontes do exportador de PNG do GridCraft
(238.924 bytes).

## Cópias únicas (`cargo tree -d`)

Uma cópia só, como exigido: `vello_cpu` e `vello_common` 0.2.0, `kurbo` 0.13.1, `peniko` 0.6.1,
`png` 0.18.1, `zip` 3.0.0, `quick-xml` 0.41.0, `image` 0.25.10, `harfrust`.

Duplicadas, e por quê (nenhuma pode ser unificada sem mudar o upstream):

| Crate | Versões | Quem puxa |
|---|---|---|
| `skrifa` | 0.47.0, 0.44.0, 0.42.1 | 0.47: WordCraft, DeckCraft e `gridcraft-raster`; 0.44: `glifo`, dentro do `vello_cpu` 0.2.0; 0.42.1: `krilla` e `subsetter` (PDF do WordCraft, via craft-libs) |
| `read-fonts` | 0.43.3, 0.44.0, 0.41.0, 0.39.2 | as mesmas cadeias acima, mais `write-fonts` |
| `font-types` | 0.11.3, 0.12.6 | as mesmas |
| `miniz_oxide` | 0.8.9 (png), 0.9.1 (flate2, via krilla, tiff e outros) | |
| `syn` | 2 e 3 | só em tempo de build, não vai para o binário |

## Oportunidades que exigem mudar o upstream (NÃO aplicadas)

Estimativas, não medidas, a menos que o texto diga o contrário.

1. **Fontes do WordCraft fora do binário ou reduzidas** (4,32 MB, 16% do binário): carregar de
   arquivo, ou subconjunto das fontes (por exemplo só Source Sans e Source Serif regulares e negrito,
   sem Inter e JetBrains Mono, que a miniatura de um documento quase nunca usa). Mudaria a
   aparência da substituição de fontes.
2. **Dicionário de hifenização** (607 KB): `en-us.dic` (433 KB) sozinho. Uma feature para
   desligar hifenização na renderização de miniaturas economizaria até 0,6 MB (mais o código de
   `wordcraft_proof`, 284 KiB, parte dele).
3. **Feature `no-pdf` no WordCraft**: `wordcraft-pdf`, `craft-pdf`, `krilla`, `subsetter`,
   `write-fonts`, `flate2`/`miniz_oxide` 0.9 e `skrifa` 0.42.1 com `read-fonts` 0.39.2 só entram
   porque o motor registra o comando `file.exportPdf`. Somando krilla (264 KiB), wordcraft_pdf
   (84 KiB), subsetter (97 KiB), write_fonts (95 KiB) e as cópias de skrifa/read-fonts, são cerca de
   0,7 MB de `.text`, sem contar dados. Removeria também a duplicata de skrifa 0.42.1.
4. **Features de escrita nos motores**: `wordcraft_docx` (escritor), `gridcraft_xlsx` (escritor),
   comandos de edição, funções de fórmula e análises do GridCraft (`gridcraft_functions` 372 KiB), o
   formato nativo `deckcraft_format` (606 KiB), o `docbin` do .doc, `image` com tiff, bmp, gif.
   Um perfil "somente leitura e renderização" nos motores poderia cortar vários MB (a ordem de
   grandeza só poderia ser medida com a mudança feita).
5. **Unificar as versões de `skrifa` e `read-fonts`**: o `vello_cpu` 0.2.0 puxa `skrifa` 0.44 via
   `glifo`, enquanto os motores usam 0.47. Uma versão nova do vello alinharia e tiraria uma cópia
   (cerca de 0,3 a 0,6 MB, estimado).
6. **Unificar crates copiados** entre os projetos (`geom`, `fonts`, `metafile` EMF/WMF, `pdf`,
   `render`, três modelos OOXML com leitores de zip e XML independentes): a análise anterior viu
   linhas parecidas e sem nenhuma em comum. Seria o maior ganho possível, mas é um trabalho de
   arquitetura dos três projetos.
7. **Recursos do `image`**: os motores habilitam png, jpeg, webp, gif, bmp e tiff. Quem só precisa
   ler as imagens que aparecem nos documentos poderia limitar os formatos (o tiff puxa `flate2` e
   `miniz_oxide` 0.9).
8. **Perfil**: `-C target-cpu`, `-Zbuild-std` (nightly) e UPX ou compressão do executável poderiam
   reduzir mais; não foram testados por exigirem nightly ou ferramentas fora do escopo.

## Histórico de medidas

| Data | Binário | Bytes | Observação |
|---|---|---|---|
| 2026-10-19 | thumbcraft 0.1.0, PNG apenas | 26.958.048 | primeira versão funcional |
| 2026-10-19 | thumbcraft 0.1.0, PNG, JPEG e WebP | 26.992.128 | versão da rodada em `baselines/thumbcraft-run-2026-10-19/` |
