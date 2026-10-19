# A família *craft

O que os projetos da organização storytold compartilham e o que não compartilham, em 2026-10-19. Fontes: `../reports/05`, `06`, `08`, `10`, `14` e `16`. Versões e SHAs dos upstream usados pelo thumbcraft: `../UPSTREAM.md` e `../scripts/fetch-upstream.sh`.

## Projetos

| Projeto | Para quê | Repositório | Versão | SHA usado nos testes |
|---|---|---|---|---|
| WordCraft | documentos Word e afins | storytold/wordcraft | | `117aea31a8e61bd3e306e9210800cdc42897e3c1` |
| GridCraft | planilhas (Excel) | storytold/gridcraft | 0.8.0 | `66ce6be4743f4227047f76f56815520f824e6d81` |
| DeckCraft | apresentações (PowerPoint) | storytold/deckcraft (o nome `slidecraft` redireciona para ele) | 0.8.0 (alpha) | `010941ec60f81f1eece31ec1ea83825c9a396217` |
| craft-libs | bibliotecas comuns | storytold/craft-libs | | ver abaixo |
| craft-fonts | fontes (fora dos repositórios) | storytold/craft-fonts | | não acessível na máquina de testes (403) |

Licença dos projetos: MIT ou Apache-2.0. Rust edition 2024, exigem rustc 1.90 (GridCraft) a 1.95 (WordCraft e DeckCraft). O README do GridCraft e o do DeckCraft listam os "Crafting Apps"; o README do WordCraft não cita os irmãos.

## O que é compartilhado

### `craft-libs`: dependência git, com revisão fixada por projeto

Crates do craft-libs: `ai-core`, `atomic`, `cms`, `docpath`, `i18n`, `pdf`, `print`, `quit`, `settings` e `ui`. O único com função de renderização é o `craft-pdf` (features `read`, `interpret`, `raster` e `write`).

| Projeto | Revisões pinadas | Para quê |
|---|---|---|
| WordCraft | `d27646a5f7c8b4fc85d9dd3213b8c88b58e0088d` e `c3487320b2d1de657b860b3c8abb44e1f2b32bd8` | `craft-ui` no primeiro; `craft-pdf` (só `write`, com `default-features = false`) e `krilla` no segundo |
| DeckCraft | `c3487320b2d1de657b860b3c8abb44e1f2b32bd8` | `craft-pdf` (só `write`) e `krilla` 0.8.2 |
| GridCraft | `d27646a5f7c8b4fc85d9dd3213b8c88b58e0088d` | `craft-ui` e `craft-print`, só na GUI e na impressão nativa, fora do motor |

- O `d27646a5` é ancestral do `c3487320`, com 56 commits de diferença (12 e 13 de outubro de 2026). No `d27646a5` o crate `pdf` não existe.
- O `craft-pdf` com a feature `raster` só existe a partir do `main`, commit `c07b3fa` (e `ca50d63`). O `c3487320` só tem o writer. O protótipo `gp-min` usou o `main` em `ae2f796b673f29cf3161f36eb6a8933f0a3fe028`. Não foi testado se subir a revisão no WordCraft e no DeckCraft quebra o `krilla`.
- Os patches do `hayro` (`hayro`, `hayro-interpret`, `hayro-syntax`, `hayro-jbig2`) ficam em `vendor/` do craft-libs e entram por `[patch.crates-io]`, que o cargo ignora em dependências: o crate raiz que usar `craft-pdf` com `raster` precisa repetir os quatro `patch` apontando para o mesmo rev. O `raster` usa hayro, que embute as fontes padrão 14 da Foxit (BSD-3, listadas no NOTICE e ATTRIBUTION do craft-libs).
- API do raster: `raster::render(page, cache, settings, Options{scale, size, offset, background})`, devolve RGBA premultiplicado.
- O `AGENTS.md` do GridCraft diz que repositórios não compartilham código ("Never code: repos don't share code"): o compartilhamento é só pelo `craft-libs`. Copiar o `wordcraft-render` para o GridCraft não é aceitável.

### Pilha de renderização (mesmas versões nos projetos que rasterizam)

`vello_cpu` 0.2.0, `kurbo` 0.13.1, `peniko` 0.6.1 e `skrifa` 0.47.0 (WordCraft e DeckCraft, e o patch do GridCraft usa exatamente essas). Também deduplicáveis: `vello_common` 0.2.0, `harfrust` 0.13.3, `fearless_simd`, `image` 0.25.10 e `png`, `flate2`, `zlib-rs`, `zip` 3.0.0, `quick-xml` 0.41.0, `serde`, `serde_json`, `thiserror`, `hashbrown`, `aes`, `sha1` e `sha2` (Word e Excel), `regex` (Word e Excel).

Divergências de versão nos locks completos: `vello_cpu` 0.1.0 (via epaint, na GUI do GridCraft e do WordCraft) convive com 0.2.0; `harfrust` 0.12 e 0.13; o WordCraft tem `fearless_simd` 0.4.1 e 1.1.0 (os outros só 0.4.1) e arrasta via `krilla` versões antigas (`skrifa` 0.42.1, `read-fonts` 0.39.2, `rustybuzz` 0.20, `ttf-parser` 0.25). No GridCraft com patch entram `vello_cpu` e `vello_common` 0.2.0, `skrifa` 0.47.0, `read-fonts` 0.44.0 e `glifo` 0.3.0; num binário que junte só os motores, `vello_cpu` 0.2.0 deduplica, e com a GUI do GridCraft ficariam duas versões.

## O que NÃO é compartilhado: código copiado e divergido

| Módulo | Word | PowerPoint | Observação |
|---|---|---|---|
| `geom` | 743 linhas | 5.576 linhas | 0 arquivos idênticos |
| `fonts` | 5.703 linhas | 2.709 linhas | 0 arquivos idênticos |
| `metafile` (EMF e WMF) | 3.212 linhas | 7.043 linhas | 2 de 9 arquivos idênticos |
| `pdf` | próprio | próprio | 0 arquivos idênticos; o GridCraft tem escritor próprio (847 linhas, fontes padrão 14) |
| `render` | `wordcraft-render` (`vello_cpu`) | `deckcraft-render` | cópias separadas |

Os três leitores OOXML (docx, pptx e xlsx) são independentes, com leitores de zip e XML próprios. Código próprio: cerca de 154 mil linhas no Word, 53 mil no PowerPoint e 76 mil no Excel, sem nenhuma linha em comum. Há parentesco evidente (mesma origem, mesmos nomes de módulos), mas são cópias, não crate compartilhado. Por isso o binário unificado só ganha na camada de crates externos e na `std`.

## Fontes embutidas

| Projeto | Fontes no binário |
|---|---|
| WordCraft | 13 TTFs em `assets/fonts` por `include_bytes!` (Source Sans 3, Source Serif 4, Inter, JetBrains Mono), 4.305.160 bytes (4,31 MB), mais 617 KB de hifenização e ortografia. Cerca de 25% do mini Word |
| DeckCraft | nenhuma dessas (exclui Source Sans e Serif por política); embute as de `craft-fonts` só se `CRAFT_FONTS_DIR` apontar para o clone (em `.cargo/config.toml`, padrão `../craft-fonts`); sem o clone o array fica vazio (foi o caso nos testes). Traz `epaint_default_fonts` (Ubuntu Light, 362 KB) como fallback |
| GridCraft (motor) | nenhuma (PDF com as fontes padrão 14); só a GUI usa `epaint_default_fonts`. O patch de PNG embute 12 TTFs Arimo, Tinos e Cousine (239 KB, SIL OFL 1.1) |
| craft-pdf raster (hayro) | fontes padrão 14 da Foxit |

Não há arquivo de fonte em comum para comparar por hash. Se `CRAFT_FONTS_DIR` for definido, `wordcraft-fonts` e `deckcraft-fonts` embutiriam cada um a sua cópia (crates distintos) e o binário unificado pagaria as fontes em dobro, a menos que o crate de fontes seja unificado (dedução da mecânica, não medida).

## Fontes do sistema e substituições

Na máquina de testes havia Carlito e Caladea (equivalentes métricos de Calibri e Cambria), Liberation e DejaVu. Fontes da Microsoft não são embutidas em nenhum projeto: quando ausentes, caem em equivalentes (no WordCraft, nas OFL embutidas), e a métrica muda.

## Capacidades por projeto (2026-10-19)

| | WordCraft | GridCraft | DeckCraft |
|---|---|---|---|
| Lê | .docx, .doc 97 (só leitura), .odt, .rtf, .html, .md, .tex, .txt, docm, dotx, dotm, dot, xml (WordML), json, htm e xhtml (detecta por conteúdo: ZIP, OLE, `{\rtf`, WordML, página web, texto) | .xlsx e .xlsm (leitura e escrita), .xls (BIFF8, sem fontes, fundos, bordas nem gráficos), .xlsb e .ods (só dados), .csv e .tsv | .pptx, .potx, .ppsx (por conteúdo), `.deckcraft`, .txt e .md (só na CLI completa) |
| Não lê | | | .ppt e .odp |
| PNG pela CLI | sim (`convert`, `render`) | não (só com o patch do thumbcraft) | sim (`render`) |
| PDF | sim (`wordcraft-pdf`, `krilla`) | sim (`gridcraft-pdf` próprio) | sim (`craft-pdf`, `krilla`) |
| Rasterizador | `vello_cpu` 0.2 | só no patch: `vello_cpu` 0.2.0 | `vello_cpu` 0.2 |
| Arquitetura de desenho | layout produz display list `Draw` (14 variantes), PDF e PNG a consomem | modo imediato no `print.rs`; só gráficos têm representação intermediária (`Prim`) | próprio |
| Aceita PRs externos | | sim (40 PRs abertos, sem CLA) | |
