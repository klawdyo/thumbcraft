# Upstream: versões fixadas dos motores *craft

Tudo aqui é reproduzido por `scripts/fetch-upstream.sh`, que clona os três projetos em
`third_party/` (ignorado pelo git) nos SHAs abaixo, aplica os patches e confere a árvore.
Para atualizar uma versão, veja a seção "Como atualizar" no fim.

## Projetos

| Projeto | URL | SHA | Data do commit | Versão do crate | Patches |
|---|---|---|---|---|---|
| WordCraft | https://github.com/storytold/wordcraft | `117aea31a8e61bd3e306e9210800cdc42897e3c1` | 2026-10-19T03:08:47-04:00 | 0.8.0 | nenhum |
| GridCraft | https://github.com/storytold/gridcraft | `66ce6be4743f4227047f76f56815520f824e6d81` | 2026-10-18T16:14:33-06:00 | 0.8.0 | `patches/gridcraft/0001-exportador-png.patch` |
| DeckCraft | https://github.com/storytold/deckcraft | `010941ec60f81f1eece31ec1ea83825c9a396217` | 2026-10-18T12:31:16-06:00 | 0.8.0 | nenhum |

Os SHAs foram lidos dos clones usados nas análises anteriores (`git rev-parse HEAD`). O
`fetch-upstream.sh` busca cada commit direto por SHA (`git fetch --depth 1 origin <sha>`), então
falha alto se o commit sumir do remoto.

## Patch do GridCraft (por que existe)

O GridCraft upstream NÃO exporta PNG (só PDF). O patch `0001-exportador-png.patch` é o diff
completo do PR de exportação de PNG contra o commit acima:

- extrai o desenho de impressão para uma lista de operações (`Op`, `Rec`, `replay` em
  `crates/pdf/src/display.rs`), que serve tanto ao PDF quanto ao PNG;
- cria o crate `crates/raster` (vello_cpu 0.2.0 + skrifa 0.47, mesmas versões do WordCraft e do
  DeckCraft) com 12 fontes TTF embutidas (Arimo, Tinos e Cousine, licença OFL, no próprio patch);
- adiciona o comando `file.exportPng` e a saída `.png` no CLI.

Verificação feita em 2026-10-19: o patch (formato `git diff --binary`, seguro para as fontes
binárias) aplicado num checkout limpo do commit `66ce6be` reconstrói exatamente a árvore da
cópia de trabalho do PR (`diff -r`, sem diferenças). A árvore patchada tem hash
`0b8e31e398afda27d6c1015cbbb79275edb20d7d` (`git write-tree`), conferido pelo script.
Tamanho do patch: 287.117 bytes. Quando o upstream ganhar PNG nativo, o patch sai.

## Revisões do craft-libs (https://github.com/storytold/craft-libs)

O craft-libs é uma dependência git compartilhada. Cada projeto pina revisões diferentes:

| Projeto | `craft-pdf` e `krilla` | `craft-ui` e outros |
|---|---|---|
| WordCraft | `c3487320b2d1de657b860b3c8abb44e1f2b32bd8` | `d27646a5` (GUI) |
| DeckCraft | `c3487320b2d1de657b860b3c8abb44e1f2b32bd8` | `d27646a5` (GUI) |
| GridCraft | não usa | `d27646a5` (`craft-ui` e `craft-print`, só GUI e impressão) |

No thumbcraft só entra a revisão `c3487320` (via `krilla` e `craft-pdf`, que o WordCraft e o
DeckCraft puxam). A `d27646a5` fica de fora porque o thumbcraft não compila nenhum crate de GUI.
O `Cargo.toml` raiz repete o remendo `[patch.crates-io] krilla` que o WordCraft e o DeckCraft
têm nas suas raízes (sem ele o `craft-pdf` não resolve). O clone de referência do craft-libs
usado nas análises estava em `ae2f796b673f29cf3161f36eb6a8933f0a3fe028` (2026-10-18), mas o
cargo baixa o craft-libs sozinho, na revisão acima.

## Versões iguais nos três (condição para unir o binário)

`vello_cpu` e `vello_common` 0.2.0, `kurbo` 0.13.1, `peniko` 0.6.1, `skrifa` 0.47.0 (o patch do
GridCraft alinhou esta), `quick-xml` 0.41, `zip` 3.0.0, `png` 0.18.1, `image` 0.25.10. O
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
