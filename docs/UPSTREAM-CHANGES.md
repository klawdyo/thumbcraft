# Mudanças nos projetos importados

Registro de **toda** modificação feita no código dos projetos importados (WordCraft, GridCraft, DeckCraft e
craft-libs) e dos remendos de dependência que o `Cargo.toml` raiz aplica sobre eles. Quem atualizar os pacotes
(roteiro em `UPDATING.md`) deve ler este arquivo antes, reaplicar ou descartar cada item e atualizá-lo no fim.

## Regras

1. **`third_party/` é gerado e nunca editado à mão.** Toda mudança no código importado é um patch pequeno em
   `patches/<projeto>/NNNN-descricao.patch`, aplicado por `scripts/fetch-upstream.sh`.
2. **Mínimo de mudança.** O núcleo dos projetos não é reescrito: o patch toca só o que o thumbcraft precisa.
3. **Uma versão por pacote.** O binário é pequeno, padronizado e eficiente; não aceitamos duas versões do mesmo
   crate (`cargo tree -p thumbcraft -d`). Quando duas versões aparecem, o mais antigo é que se alinha ao mais novo.
4. **Todo patch ou remendo entra aqui** com: o que muda, por quê, como reaplicar e como saber se ainda é necessário.

## Patches de código

### 1. GridCraft: exportação de PNG (`patches/gridcraft/0001-exportador-png.patch`)

- **O que muda:** o upstream só exporta PDF. O patch (287.117 bytes, formato `git diff --binary`) extrai o desenho de
  impressão para uma lista de operações (`Op`, `Rec`, `replay` em `crates/pdf/src/display.rs`), cria o crate
  `crates/raster` (vello_cpu 0.2.0 e skrifa 0.47, com 12 fontes TTF OFL embutidas) e adiciona o comando
  `file.exportPng` e a saída `.png` no CLI.
- **Por quê:** o thumbcraft gera PNG de planilhas pelo `file.exportPng`.
- **Como reaplicar:** `scripts/fetch-upstream.sh --force`. Se não aplicar contra um SHA novo, aplique em um clone limpo,
  resolva à mão (o ponto sensível é `print.rs`), gere o patch de novo com `git diff --binary` e atualize
  `ARVORE_GRIDCRAFT` no script. Detalhes em `../UPSTREAM.md` e `PR-GRIDCRAFT.md`.
- **Ainda é necessário?** Enquanto o GridCraft não tiver `file.exportPng` nativo. Quando tiver, o patch sai.
- **Estado conhecido:** no Windows o hash da árvore patchada (`ARVORE_GRIDCRAFT`) diverge do calculado no Linux e o
  script termina com erro depois de baixar tudo (provável diferença de modo de arquivo; tarefa 05).

### 2. craft-libs: hayro no `vello_cpu` 0.2.0 (`patches/craft-libs/0001-hayro-vello-cpu-0.2.patch`)

- **Base:** `craft-libs` na revisão `ae2f796b673f29cf3161f36eb6a8933f0a3fe028` (2026-10-18), baixado para
  `third_party/craft-libs`. Os crates usados são `crates/pdf` (`craft-pdf`) e `vendor/{krilla,hayro,hayro-interpret,
  hayro-syntax,hayro-jbig2}`.
- **O que muda** (3 arquivos de `vendor/hayro`, 10 linhas inseridas e 14 removidas):
  - `Cargo.toml`: `vello_cpu` de `0.0.8` para `0.2.0` (mesmos recursos: `std`, `png`, `u8_pipeline`).
  - `src/renderer.rs`: `ctx.render_to_pixmap(&mut resources, &mut pix)` vira `ctx.render(&mut pix, &mut resources)`
    (4 lugares).
  - `src/lib.rs`: `ctx.render_to_buffer(resources, buffer, w, h, modo)` vira `ctx.render(PixmapMut::new(w, h, buffer),
    &mut resources)` (o tamanho do buffer já é conferido antes); sai o campo `render_mode` de `RenderSettings`
    (agora vive em `RasterizerSettings`, e o padrão `OptimizeSpeed` é o mesmo que o hayro usava).
- **Por quê:** o hayro do craft-libs usava `vello_cpu` 0.0.8, e os três motores usam 0.2.0. Sem o patch o binário
  levava dois rasterizadores completos (`vello_cpu`, `vello_common`, `glifo`) e, sem alinhar o `craft-pdf`, duas
  revisões dele (a `c3487320` do WordCraft e do DeckCraft e a `ae2f796` com `raster`).
- **Como reaplicar:** `scripts/fetch-upstream.sh` (aplica sozinho). Num SHA novo do craft-libs que mude esses
  arquivos, refaça as quatro trocas acima à mão e gere o patch de novo
  (`git -C third_party/craft-libs -c core.autocrlf=false diff --binary > patches/craft-libs/0001-....patch`).
- **Ainda é necessário?** Não, se o `vendor/hayro` do craft-libs já usar `vello_cpu` 0.2.x. Confira com
  `grep -n vello_cpu third_party/craft-libs/vendor/hayro/Cargo.toml`. Se usar 0.3 ou mais novo, o certo é subir os
  motores ou refazer o porte para a versão que os três usam.
- **Cuidado:** o `vendor/hayro/SOURCE.json` guarda os hashes originais dos arquivos; depois do patch eles não batem
  mais. Nada no thumbcraft lê esse arquivo.

## Remendos de dependência (`Cargo.toml` raiz)

Não alteram código: dizem ao cargo de onde vem cada pacote, para o binário ter uma cópia só.

| Remendo | Efeito |
|---|---|
| `[patch."https://github.com/storytold/craft-libs"] craft-pdf = { path = "third_party/craft-libs/crates/pdf" }` | O WordCraft e o DeckCraft pedem o `craft-pdf` da revisão `c3487320` por git; passam a usar a cópia local (revisão `ae2f796`), a mesma do motor de PDF. A feature `write` que eles pedem existe nas duas revisões. |
| `[patch.crates-io] krilla` | O `craft-pdf` exige o `krilla` 0.8.2 do craft-libs (os dois motores têm o mesmo remendo nas raízes deles). |
| `[patch.crates-io] hayro, hayro-interpret, hayro-syntax, hayro-jbig2` | O `craft-pdf` com `raster` exige o hayro remendado do craft-libs. O cargo ignora `[patch]` de dependências, então a raiz repete o remendo. |

**Como reaplicar:** se o craft-libs mudar de revisão, edite a tabela `PROJETOS` do `fetch-upstream.sh` e confira se as
versões (`=0.7.1` e outras) de `vendor/` ainda casam com o que o `craft-pdf` pede.

## Duplicações que ainda existem no binário (`cargo tree -p thumbcraft -d -e normal`)

Depois desta mudança não há mais duplicata de `craft-pdf`, `vello_cpu`, `vello_common` nem `glifo`. As que sobram já
existiam antes do motor de PDF e não foram mexidas, porque resolver exige alterar o núcleo dos projetos importados:

| Pacote | Versões | Quem puxa | O que resolveria |
|---|---|---|---|
| `skrifa` | 0.42.1, 0.44.0, 0.47.0 | 0.42.1: `krilla` e `subsetter` (escrita de PDF); 0.44.0: `glifo` (texto do `vello_cpu`); 0.47.0: os três motores e o `hayro-interpret` | Alinhar o `krilla` e o `glifo` ao 0.47 (patch nos dois) ou tirar o `krilla` do binário (o thumbcraft nunca exporta PDF; ver o estudo do `krilla` na tarefa 08) |
| `read-fonts` | 0.39.2, 0.41.0, 0.43.3, 0.44.0 | acompanha `skrifa` e `font-types` | Some junto com o `skrifa` |
| `font-types` | 0.11.3, 0.12.6 | acompanha `skrifa` | Some junto com o `skrifa` |
| `miniz_oxide` | 0.8.9, 0.9.1 | 0.8.9: `png` 0.18.1 e `wordcraft-docbin`/`wordcraft-proof`; 0.9.1: `flate2` 1.1.10 | Esperar `png` e `flate2` convergirem, ou fixar uma versão com `cargo update` se a faixa permitir |
| `syn` | 2.0.119, 3.0.6 | derivações (tempo de compilação); não entra no código do binário | Nada a fazer |

Estas duplicações valem uma tarefa própria se o dono pedir.
