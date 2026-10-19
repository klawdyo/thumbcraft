# Como compilar o thumbcraft

## Pré-requisitos

| Item | Versão usada | Para quê |
|---|---|---|
| Linux x86_64 | kernel 6.18 | testado só aqui |
| Rust (rustc e cargo) | 1.97.0 (2026-07-07) | os projetos exigem 1.95 ou mais (edition 2024) |
| git | 2.43 | `scripts/fetch-upstream.sh` |
| Rede | GitHub e crates.io | clones por SHA, crates e o craft-libs (git) |
| bash, awk, od, cmp, timeout | coreutils | scripts |
| ImageMagick | 6.9.12 (`convert`, `montage`, `identify`) | só `scripts/sheets.sh` |
| cargo-bloat | opcional | quebra por crate (docs/SIZES.md) |

Espaço em disco: o `target/` de release ocupa cerca de 0,9 GB e os três clones em `third_party/`
cerca de 71 MB. Memória: o link com LTO total usa alguns GB; 8 GB bastam.

## Comandos exatos

```sh
git clone https://github.com/klawdyo/thumbcraft && cd thumbcraft
scripts/build.sh                 # busca os upstream, aplica o patch e compila em release
```

`scripts/build.sh` equivale a:

```sh
scripts/fetch-upstream.sh        # third_party/{wordcraft,gridcraft,deckcraft} em SHAs fixos + patch
cargo build --release --locked -p thumbcraft
```

O binário fica em `$CARGO_TARGET_DIR/release/thumbcraft` (padrão: `target/release/thumbcraft`).
Para compilar fora do repositório, `export CARGO_TARGET_DIR=/caminho/target`.

Testes:

```sh
cargo test                       # 80 testes do núcleo (rápido, sem compilar os motores)
cargo test --workspace --locked  # inclui os testes da CLI (compila os motores em debug, demorado)
scripts/compare-native.sh        # compara byte a byte com wc-min, dc-min e gc-min (ver abaixo)
scripts/run-corpus.sh corpus/docx /tmp/saida -W 120 -H 120
scripts/sheets.sh /tmp/saida /tmp/saida/results.tsv /tmp/saida/sheet.png
```

Sem `--locked`, o cargo pode atualizar o `Cargo.lock`; o lock versionado parte do lock do binário
combinado da análise anterior (mesmas versões de vello_cpu, skrifa, kurbo, peniko).

### compare-native

Os minis de referência não estão no repositório (são binários). Para refazer a prova, compile-os
a partir das três árvores em `third_party/` com o mesmo perfil e aponte `WC_MIN`, `DC_MIN` e
`GC_MIN`. O `gc-min` é o programa descrito em `baselines/excel/prototipos/` (com `--crop`
opcional, não usado aqui). Os minis usados em 2026-10-19 estavam em:

- `wc-min` (apps/wc-min do WordCraft): escala 2 fixa;
- `dc-min IN OUT ESCALA`: o script passa a escala 2;
- `gc-min IN OUT`: A4, sem zoom, escala 2, página 1.

## Tempos de build (2 vCPUs compartilhadas)

| Situação | Tempo |
|---|---|
| Primeiro build completo (com download de crates) | 6 min 35 s |
| Recompilar só o crate `thumbcraft` (o link com LTO total refaz tudo) | 4 min 10 s |
| Build completo com outro build concorrente | 13 a 14 min |
| `cargo test` do núcleo | cerca de 15 s; `cargo test --workspace` leva cerca de 5 min |

## Armadilhas

1. **`third_party/` dentro do workspace**: o cargo trata dependências por caminho que ficam dentro
   do diretório do workspace como membros dele, e os crates do WordCraft e do DeckCraft usam
   `workspace = true` do próprio repositório deles. O `Cargo.toml` raiz tem
   `exclude = ["third_party"]` por isso; sem ele aparece "error inheriting ... from workspace root
   manifest".
2. **`[patch.crates-io] krilla`**: o `craft-pdf` exige o krilla do craft-libs; o remendo na raiz
   repete o do WordCraft e do DeckCraft (revisão `c3487320`). Sem ele a resolução falha.
3. **DeckCraft sem `craft-fonts`**: o `.cargo/config.toml` do DeckCraft só vale dentro da árvore
   dele; aqui `CRAFT_FONTS_DIR` fica vazio e o `build.rs` do DeckCraft só avisa. É o mesmo
   comportamento do `dc-min` de referência.
4. **Patch do GridCraft**: sem ele não existe `file.exportPng` e o build falha em
   `gridcraft_engine::...`. `fetch-upstream.sh` confere o hash da árvore patchada.
5. **Repositórios rasos**: o script busca cada commit por SHA com `--depth 1`; o histórico não
   está disponível em `third_party/`.
6. **Disco cheio**: o build precisa de alguns GB livres no `CARGO_TARGET_DIR`.
7. **`cargo test --workspace` é pesado**: compila os três motores em debug. Use `cargo test` (só o
   núcleo) no dia a dia.
8. **panic = "abort"**: uma falha interna derruba o processo (veja docs/ARCHITECTURE.md).
9. **Variáveis de ambiente**: `LANG` e `LC_*` mudam o papel padrão (A4 ou Letter) de documentos
   sem tamanho (txt, md, html).
10. O `build.rs` embute SHAs e datas lidos de `third_party/*/.git`; sem git, defina
    `THUMBCRAFT_WORD_SHA`, `THUMBCRAFT_WORD_DATE` (e `GRID`, `DECK`).

## Diário de build

### 2026-10-19

- 10:07 Ambiente: o disco estava 100% cheio (259 MB livres). Liberei cerca de 22 GB removendo
  caches de build de depuração e wasm de uma análise anterior
  (`grid-png/target/debug` e `wasm32-unknown-unknown` no scratchpad da sessão), que são regeneráveis.
- 10:15 Verificado que `patches/gridcraft/0001-exportador-png.patch` (`git diff --binary`, 287.117
  bytes) é seguro para as fontes binárias: aplicado num checkout limpo do commit `66ce6be`, `diff -r`
  contra a cópia do PR dá árvores idênticas. Hash da árvore patchada:
  `0b8e31e398afda27d6c1015cbbb79275edb20d7d`. Nenhum patch alternativo foi preciso.
- 10:20 `scripts/fetch-upstream.sh` (6,9 s na primeira vez, idempotente na segunda) clona os três
  projetos por SHA e aplica o patch.
- 10:40 Núcleo (`crates/core`) escrito com 80 testes; `cargo test`: ok.
  Conflito encontrado: "error inheriting `harfrust` from workspace root manifest". Causa: dependência
  por caminho dentro do diretório do workspace vira membro. Solução: `exclude = ["third_party"]`.
- 11:25 Primeiro `cargo build --release -p thumbcraft`: 6 min 35 s até o único erro (tipo
  `Presentation` privado em `deckcraft-pptx`; solução: dependência direta de `deckcraft-model`).
- 11:33 Build ok: 26.958.048 bytes. `cargo tree -d`: skrifa 3 versões, read-fonts 4, font-types 2,
  miniz_oxide 2 (ver docs/SIZES.md; nenhuma resolvível sem mudar o upstream); vello_cpu, vello_common,
  kurbo, peniko, png, zip e quick-xml em uma cópia só.
- 11:40 `scripts/compare-native.sh`, primeira rodada: Word 29 de 30, Grid 57 de 57, Deck 9 de 46.
  Causa dos 37 do Deck: o script chamava o `dc-min` sem escala (padrão 1) e o thumbcraft usa 2;
  corrigido no script (passa `2`). Causa do 1 do Word (`28_extensao_errada_md_como.pdf`, um
  markdown salvo como pdf): o roteamento mandava `.pdf` para o motor de PDF (erro 3) e o wc-min o
  abre como texto. Correção: texto sem assinatura com extensão `.pdf` segue para o WordCraft.
- 11:57 Segunda rodada: Word 30 de 30, Deck 46 de 46, Grid 57 de 57 (ver abaixo o significado).
- 12:00 Adicionados JPEG e WebP (features `jpeg` e `webp`): +34.080 bytes. Build de 4 min 10 s.
  Binário final: 26.992.128 bytes.
- 12:16 Experimento `RUSTFLAGS="-C force-unwind-tables=no"`: mesmo tamanho, 26.992.128 bytes
  (build de 14 min com concorrência). Não adotado.
- 12:20 Rodadas finais nos corpora: ver `baselines/thumbcraft-run-2026-10-19/`.

Resultado da compare-native (`baselines/thumbcraft-run-2026-10-19/compare-native.txt`): 133 arquivos
nos três corpora, nenhum FAIL. Desses, 116 geram PNG idêntico byte a byte ao do mini (PASS), 16 falham
nos dois com código diferente de zero (PASS-ERRO: arquivos inválidos ou legados) e 1 é a diferença
intencional ESPERADO (`44_slide1_xml_quebrado.pptx`: o dc-min renderiza o slide 2 no lugar do 1, o
thumbcraft recusa com código 4).
