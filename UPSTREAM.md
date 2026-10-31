# Upstream: versões fixadas dos motores *craft

Tudo aqui é reproduzido por `scripts/fetch-upstream.sh`, que clona os projetos em
`third_party/` (ignorado pelo git) nos SHAs abaixo, aplica os patches e confere a árvore.
Para atualizar uma versão, veja a seção "Como atualizar" no fim.

## Projetos

| Projeto | URL | SHA | Data do commit | Versão do crate | Patches |
|---|---|---|---|---|---|
| WordCraft | https://github.com/storytold/wordcraft | `e6a5cd05f045e4caf25a28660bfeb622f25b5129` | 2026-10-31T16:03:07Z | 0.13.2 | `patches/wordcraft/0001-wordcraft-pdf-stub.patch` |
| GridCraft | https://github.com/storytold/gridcraft | `67e39faeea1fa4b603cbab421a30737db2cef8d9` | 2026-10-31T14:17:29Z | 0.14.2 | `patches/gridcraft/0001-exportador-png.patch` |
| DeckCraft | https://github.com/storytold/deckcraft | `a3fbf02952839c223df600dc33106a2dbc65be1b` | 2026-10-31T05:06:52Z | 0.13.1 | `patches/deckcraft/0001-skrifa-0.46.patch` |
| craft-libs | https://github.com/storytold/craft-libs | `c39e435cc34417843793df240a0b38bac970179b` | 2026-10-31T05:04:55Z | 0.1.0 (`craft-pdf`) | `patches/craft-libs/0001-hayro-vello-cpu-0.2.patch` |

Os SHAs foram lidos dos clones usados nas análises anteriores (`git rev-parse HEAD`). O
`fetch-upstream.sh` busca cada commit direto por SHA (`git fetch --depth 1 origin <sha>`), então
falha alto se o commit sumir do remoto.

## Patch do GridCraft (por que existe)

O GridCraft upstream NÃO exporta PNG (só PDF). O patch `0001-exportador-png.patch` é o diff
completo do PR de exportação de PNG contra o commit acima:

- extrai o desenho de impressão para uma lista de operações (`Op`, `Rec`, `replay` em
  `crates/pdf/src/display.rs`), que serve tanto ao PDF quanto ao PNG;
- cria o crate `crates/raster` (vello_cpu 0.2.0 + skrifa 0.46, mesmas versões do WordCraft e do
  DeckCraft) com 12 fontes TTF embutidas (Arimo, Tinos e Cousine, licença OFL, no próprio patch);
- adiciona o comando `file.exportPng` e a saída `.png` no CLI.

Verificação feita em 2026-10-31: o patch foi refeito contra o commit `67e39fa` (o upstream dividiu o
`print.rs` em `print/*.rs` e mudou a API do `Page`; ver `docs/UPSTREAM-CHANGES.md`). Em formato
`git diff --binary` (seguro para as fontes binárias), aplica limpo num checkout do commit. A árvore
patchada tem hash `d07fc084349f6864c578564786dcf05c7a2cd4d3` (`git write-tree`), conferido pelo script.
Tamanho do patch: 295.366 bytes. Quando o upstream ganhar PNG nativo, o patch sai.

## craft-libs (https://github.com/storytold/craft-libs)

O craft-libs é uma dependência compartilhada: o WordCraft e o DeckCraft pedem o `craft-pdf` e o `krilla` por git na
revisão `c3487320b2d1de657b860b3c8abb44e1f2b32bd8` (só escrita de PDF), e o GridCraft usa só crates de GUI e impressão
(`d27646a5`) que o thumbcraft não compila. O thumbcraft nunca escreve PDF: `patches/wordcraft/0001-wordcraft-pdf-stub.patch`
troca o `wordcraft-pdf` por um substituto (`stubs/wordcraft-pdf`), e o `krilla` e a revisão `c3487320` saem do binário.
O craft-libs entra só para ler PDF: o thumbcraft o clona em `third_party/craft-libs` na revisão
`c39e435cc34417843793df240a0b38bac970179b` (2026-10-31), que tem a feature `raster` do `craft-pdf`, e aplica
`patches/craft-libs/0001-hayro-vello-cpu-0.2.patch` (porta o hayro para o `vello_cpu` 0.2.0 dos motores). O que cada
patch faz está em `docs/UPSTREAM-CHANGES.md`.

## Crates do crates.io com patch

O `glifo` 0.3.0 (texto do `vello_cpu`) é baixado do crates.io como `.crate` (sha256 conferido) para `third_party/glifo`, e
`patches/glifo/0001-skrifa-0.46.patch` o alinha ao `skrifa` dos motores. Tabela `CRATES` do `fetch-upstream.sh`.

## Versões iguais nos três (condição para unir o binário)

`vello_cpu` e `vello_common` 0.2.0, `kurbo` 0.13.1, `peniko` 0.6.1, `skrifa` 0.46.2 (patches 1 a 5 de docs/UPSTREAM-CHANGES.md,
que alinham com o read-fonts 0.43.3 do harfrust), `quick-xml` 0.41, `zip` 3.0.0, `png` 0.18.1, `image` 0.25.10. O
`Cargo.lock` do thumbcraft parte do lock do binário combinado da análise anterior (a união dos
três locks), e cargo ajusta o que faltar.

## DeckCraft sem `craft-fonts`

O `.cargo/config.toml` do DeckCraft aponta `CRAFT_FONTS_DIR` para `../craft-fonts`. O cargo só
lê esse arquivo quando roda dentro da árvore do DeckCraft, então no thumbcraft a variável fica
vazia, como no `dc-min` de referência, e o DeckCraft usa só as fontes do sistema e o fallback
(Ubuntu Light). Isso afeta a fidelidade do texto dos slides (ver `docs/ARCHITECTURE.md`).

## Como atualizar

1. Escolha o novo SHA do projeto e edite a tabela `PROJETOS` em `scripts/fetch-upstream.sh`
   (SHA e data); atualize a tabela acima.
2. Rode `scripts/fetch-upstream.sh --force`. Se o patch do GridCraft não aplicar, refaça-o contra
   o novo commit e atualize `ARVORE_GRIDCRAFT` no script (o script informa o hash obtido).
3. Rode `scripts/build.sh`; se houver conflito de versões, veja `docs/BUILD.md`.
4. Rode `cargo test --workspace`, `scripts/compare-native.sh` e `scripts/run-corpus.sh`, e
   compare com `baselines/`.
