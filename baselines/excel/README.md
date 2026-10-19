# Baseline Excel (GridCraft, protótipos e LibreOffice)

O que é: resultados do teste de thumbnail (primeira página A4 da primeira aba visível, em PNG) nos 57 arquivos de `../../corpus/xlsx/`. O GridCraft original não gera PNG pela CLI, então este baseline guarda a linha de base do LibreOffice e as três implementações construídas na sessão para comparação futura: o protótipo `gp-min` (PDF do GridCraft rasterizado com `craft-pdf` raster), o `gc-min` (GridCraft com o patch de exportação PNG, backend `vello_cpu` 0.2.0) e a comparação final entre os três.

De quando: 2026-10-19.

## Binários e versões

| Item | Versão |
|---|---|
| GridCraft | commit `66ce6be4743f4227047f76f56815520f824e6d81` (2026-10-18), versão 0.8.0 |
| Patch do PR (refatoração, `file.exportPng`, crate `gridcraft-raster`) | 32 arquivos, +1043 e -33 (guardado em `../../patches/gridcraft/`, ver `../../docs/PR-GRIDCRAFT.md`) |
| `gp-min` | 169 linhas (A4 sem zoom), `prototipos/gp-min/main.rs`; `craft-pdf` com feature `raster` na revisão `ae2f796b673f29cf3161f36eb6a8933f0a3fe028` do craft-libs, mais os patches do hayro repetidos no `Cargo.toml` raiz |
| `gc-min` | `prototipos/gc-min/main.rs`, só depende do motor patchado, `png` 0.18 para o crop |
| Perfil de build | `opt-level=z`, LTO fat, `codegen-units=1`, `panic=abort`, `strip=symbols` |
| Tamanho do `gp-min` | A3: 11.409.736 bytes (11,41 MB, 10,88 MiB); A4 sem zoom: 11.407.912 bytes (11,41 MB, 10,88 MiB) |
| Tamanho do `gc-min` | sem crop 8.388.368 bytes (8,39 MB, 8,00 MiB); com crop 8.407.856 bytes (8,41 MB, 8,02 MiB) |
| Rust | 1.97.0 |
| LibreOffice | 24.2.7.2, `soffice --headless --convert-to png`, processo novo por arquivo, `timeout 60`, `nice` |

## Conteúdo

| Caminho | O que é |
|---|---|
| `resultados/libreoffice.tsv` | linha de base do LibreOffice (arquivo, exit, ms, bytes do PNG). Todos exit 0, 54 com PNG |
| `resultados/gp-min-a3-recortado-ao-conteudo.tsv` | 1a versão do Caminho C: A3, intervalo A1:Z60 ajustado a uma página, recortado ao conteúdo, escala 2. Tem 58 linhas porque processou também o `MANIFEST.tsv` (virou PNG de texto), que não conta |
| `resultados/gp-min-a4-recortado-ao-conteudo.tsv` | A4 em vez de A3, ainda com ajuste e recorte |
| `resultados/gp-min-a4-sem-zoom-escala2.tsv` | versão final do `gp-min`: A4 inteira, sem zoom, escala 2 (1190x1683 px) |
| `resultados/gp-min-a4-sem-zoom-escala1.tsv` | idem, escala 1 (595x842 px) |
| `resultados/gc-min-escala2.tsv` | `gc-min` (patch do GridCraft), escala 2 (1191x1684 px), 55 arquivos (ignora .tsv e o manifesto) |
| `resultados/gc-min-escala1.tsv` | idem, escala 1 |
| `resultados/gc-min-escala2-crop.tsv` | idem, escala 2 com `--crop` (recorte ao conteúdo mais 8 px de folga) |
| `resultados/comparacao-final-tres-colunas.tsv` | saída do `compare.sh` com as três colunas (ver aviso abaixo) |
| `imagens/` | contact sheets das três implementações (`contact-sheet-*.png`), comparação A3 contra A4, comparação A4 sem zoom contra LibreOffice, e 9 imagens lado a lado (`lado-a-lado-NN.png`, 7 casos por imagem, em ordem alfabética, colunas: LibreOffice, `gp-min` A4 sem zoom, `gc-min` patch). Todas reduzidas a no máximo 1600 px de largura |
| `amostras/` | 5 casos em PNG reduzido a 600 px de largura, um por motor (15 arquivos) |
| `prototipos/` | fontes do `gp-min` e do `gc-min` (`Cargo.toml.txt` é o `Cargo.toml` renomeado; os caminhos absolutos da sessão foram trocados por `S`) |
| `ferramentas/` | `compare.sh` (lado a lado em 3 colunas, ImageMagick), `run-lo.sh` (linha de base do LibreOffice), `run-corpus-gp-min.sh` e `run-corpus-gc-min.sh` |
| `tamanhos/` | quebra do tamanho do binário: `variantes-de-build.log` (original, fase 1 e PR), `cargo-bloat-por-crate.txt` e o script `build-variants.sh` |

## Avisos sobre os dados

- Em `comparacao-final-tres-colunas.tsv`, as colunas `ms_pdf-raster` e `exit_pdf-raster` estão trocadas (o script leu o `results.tsv` do `gp-min`, que tem a ordem arquivo, ms, código). Leia o valor da coluna `exit_pdf-raster` como milissegundos e o de `ms_pdf-raster` como código de saída.
- Nos `resultados/gp-min-*.tsv` a ordem das colunas é arquivo, ms, código, bytes, mensagem. Nos `gc-min-*.tsv` é arquivo, exit, ms, bytes.
- Nos arquivos do `gp-min` o código de saída das falhas é 2, e no `gc-min` é 1 (os dois com mensagem de erro limpa).
- A linha de base do LibreOffice mostra uma página A4 em escala original, 795x1124 px. Os protótipos usam outra escala em px por pt, então as dimensões diferem mesmo com a mesma paginação.
- Os PNGs completos (cerca de 10 MB) não foram guardados, só amostras. Para regenerar, use `ferramentas/` com os binários.

## Resumo dos números

Tabelas completas em `../../docs/RESULTS.md`.

| | LibreOffice | `gp-min` A4 sem zoom (esc. 2) | `gc-min` patch (esc. 2) |
|---|---|---|---|
| PNG gerado | 54 de 57 | 54 de 57 | 52 de 55 (os 2 .tsv não são processados) |
| Tempo médio | 1653 ms | 129 ms sobre os 57 (136 ms nos 54 sucessos) | 134 ms sobre os 55 (141 ms nos 52 sucessos) |
| Tempo máximo | 2101 ms | 478 ms (planilha de 10 mil linhas) | 393 ms (idem) |
| PNG médio | 34 KB | 48 KB | 48 KB |

## Como repetir

Ver `../../docs/TESTS.md` (seção "Thumbnails Excel") e `../../docs/UPDATING.md`.
