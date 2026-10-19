# Mudanças nos projetos importados

Registro de **toda** modificação feita no código dos projetos importados (WordCraft, GridCraft, DeckCraft e
craft-libs), dos crates próprios que os substituem (`stubs/`) e dos remendos de dependência que o `Cargo.toml` raiz
aplica sobre eles. Quem atualizar os pacotes
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

### 3. WordCraft: `wordcraft-pdf` trocado por um substituto (`patches/wordcraft/0001-wordcraft-pdf-stub.patch`)

- **O que muda:** uma linha em `Cargo.toml` (raiz do WordCraft): `wordcraft-pdf = { path = "crates/pdf" }` vira
  `{ path = "../../stubs/wordcraft-pdf" }`. O substituto (`stubs/wordcraft-pdf`, crate do thumbcraft, com testes) mantém
  a API que o `wordcraft-engine` chama (`PdfOptions`, `export`, `export_layout`, `PdfError`) e sempre devolve erro.
  Nenhum arquivo do WordCraft é reescrito.
- **Por quê:** o thumbcraft nunca exporta PDF, e o crate real arrasta o `krilla` (escritor de PDF), o `subsetter` e o
  `skrifa` 0.42.1 com seus `read-fonts` e `font-types` antigos, duplicados no binário. Com o substituto o binário
  encolheu 1.046.528 bytes (26.701.824 para 25.655.296).
- **Como reaplicar:** `scripts/fetch-upstream.sh` aplica sozinho. Se o `PdfOptions` ganhar campo que o engine use, ou o
  engine chamar outra função de `wordcraft_pdf`, o build quebra: acrescente o campo ou a função ao substituto.
  Se o `Cargo.toml` do WordCraft mudar a linha, refaça o patch.
- **Ainda é necessário?** Enquanto o `wordcraft-pdf` real depender do `krilla` e o thumbcraft não exportar PDF.
  O `deckcraft-pdf` não entra no binário (o thumbcraft não usa o `deckcraft-engine`), então não precisa de substituto.
- **Efeito:** `file.export` para `.pdf` e `file.printPdf` do engine do WordCraft falham com
  "PDF export is not available in thumbcraft"; a CLI do thumbcraft não chama nenhum dos dois.

### 4. glifo: `skrifa` 0.47 (`patches/glifo/0001-skrifa-0.47.patch`)

- **Base:** crate `glifo` 0.3.0 do crates.io (texto do `vello_cpu` 0.2.0), baixado como `.crate` por
  `scripts/fetch-upstream.sh`, conferido pelo sha256 do `Cargo.lock` e extraído em `third_party/glifo`.
- **O que muda:** uma linha em `Cargo.toml`: `skrifa` de `0.44.0` para `0.47.0`. Nenhum código muda; compila e os
  testes passam.
- **Por quê:** o glifo pedia o `skrifa` 0.44, e os motores e o hayro usam o 0.47: duas versões no binário.
- **Como reaplicar:** `scripts/fetch-upstream.sh` (aplica sozinho). Se um `vello_cpu` novo trouxer outro `glifo`, atualize
  a tabela `CRATES` do script (versão e sha256) e refaça o patch.
- **Ainda é necessário?** Não, se o `glifo` que o `vello_cpu` usar já pedir o `skrifa` que os motores usam. Confira com
  `cargo tree -p thumbcraft -d -e normal`.
- **Cuidado:** os contornos dos glifos vêm agora do `skrifa` 0.47; uma diferença visual mínima no texto é possível.
  Os testes do workspace passam; a comparação visual completa com `baselines/` fica com a tarefa 05.

## Remendos de dependência (`Cargo.toml` raiz)

Não alteram código: dizem ao cargo de onde vem cada pacote, para o binário ter uma cópia só.

| Remendo | Efeito |
|---|---|
| `[patch.crates-io] hayro, hayro-interpret, hayro-syntax, hayro-jbig2` (para `third_party/craft-libs/vendor/`) | O `craft-pdf` com `raster` exige o hayro remendado do craft-libs. O cargo ignora `[patch]` de dependências, então a raiz repete o remendo. |

| `[patch.crates-io] glifo` (para `third_party/glifo`) | Usa a cópia com o patch 4. |

Já houve remendos de `krilla` e de `craft-pdf` (para o WordCraft e o DeckCraft usarem a mesma cópia do craft-libs);
saíram com o substituto do `wordcraft-pdf` (patch 3), porque nada no binário usa mais a feature `write`.

**Como reaplicar:** se o craft-libs mudar de revisão, edite a tabela `PROJETOS` do `fetch-upstream.sh` e confira se as
versões de `vendor/` ainda casam com o que o `craft-pdf` pede.

## Duplicações que ainda existem no binário (`cargo tree -p thumbcraft -d -e normal`)

Não há duplicata de `craft-pdf`, `vello_cpu`, `vello_common`, `glifo`, `krilla`, `skrifa`, `font-types` nem
`miniz_oxide`. Sobra uma, justificada:

| Pacote | Versões | Quem puxa | Por que fica |
|---|---|---|---|
| `read-fonts` | 0.43.3, 0.44.0 | 0.43.3: `harfrust` 0.13.3 (formatação de texto do WordCraft e do DeckCraft); 0.44.0: `skrifa` 0.47 | O `harfrust` 0.13.x é a última versão que o WordCraft e o DeckCraft aceitam (`"0.13"`) e pede o `read-fonts` 0.43. Testado em 2026-10-19: subir o `read-fonts` dele para 0.44 dá 14 erros de compilação (`advance_width_delta`, `compute_float_delta` e outras mudanças de API) e exigiria reescrever partes da formatação de texto, contra a regra de mexer o mínimo. Sai quando o WordCraft e o DeckCraft aceitarem um `harfrust` para o `read-fonts` 0.44. |
| `syn` | 2.0.119, 3.0.6 | derivações (tempo de compilação); não entra no código do binário | Nada a fazer |

`miniz_oxide` ficou numa versão só fixando o `flate2` em 1.1.9 no `Cargo.lock` (a 1.1.10 pede o `miniz_oxide` 0.9, e o
`png` e o WordCraft pedem 0.8). Um `cargo update` do `flate2` desfaz isso: depois de atualizar, rode
`cargo tree -p thumbcraft -d -e normal` e, se o `miniz_oxide` voltar a duplicar, fixe o `flate2` de novo
(`cargo update -p flate2 --precise 1.1.9`).
