# GridCraft: crop no gc-min e quebra do tamanho do binário

> Arquivo de relatório. Origem: subagente `ab4ce65aee885f788`, turno 3 (de 3). Sessão de 2026-10-19.
> Tema: Duas dúvidas do usuário: por que o crop não existia (era omissão, 0,019 MB) e de onde vem o tamanho do gc-min (motor 6,43 MB, backend PNG 1,95 MB).
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

Os dois pontos estão resolvidos só na parte de viabilidade. O `changes-pr.patch` e a árvore do PR não mudaram.

## 1) `--crop` no `gc-min`

O `--crop` agora existe: recorta para a caixa dos pixels não brancos, com 8 px de folga branca. Se a página inteira for branca, devolve a página inteira.

- **Como foi feito:** o `gridcraft-raster` só entrega o PNG já codificado, então o `gc-min` o decodifica e recodifica com o crate `png`. Esse crate já estava no lock (é do `vello_cpu`), então não há dependência nova. O `gc-min` ganhou 19.488 bytes (0,019 MB) e o build foi de 1m48.
- **Corpus com `--crop`, escala 2:** 55 arquivos, 52 com código de saída 0 e as 3 falhas esperadas de sempre (0 bytes, truncado e corrompido). Tempo médio de 164 ms e máximo de 431 ms (a planilha de 10 mil linhas). Sem crop, a média era 134 ms e o máximo 393 ms.
- **Conferência visual:** 6 PNGs. Os recortes de bordas, gráfico, texto rotacionado, planilha larga e sparklines estão corretos, com a folga de 8 px. A planilha vazia ficou com a página A4 inteira, como previsto.

**Por que não tinha sido implementado antes.** Foi decisão de escopo minha, e não havia dificuldade real. Eu tinha um recorte no esboço do crate raster e o tirei quando você pediu que o comando do PR não tivesse comportamento de thumbnail. Depois não o recoloquei no `gc-min`, e o relatório anterior apresentou isso como "não implementado" em vez de dizer que era uma omissão. São cerca de 40 linhas e custam 0,019 MB.

## 2) Tamanho do `gc-min`

**a) Antes e depois do crop** (você tem razão na unidade: o "8,00 MB" do relatório anterior era MiB):

| `gc-min` | Bytes | MB | MiB |
|---|---|---|---|
| Antes do `--crop` | 8.388.368 | 8,388 | 8,000 |
| Com `--crop` | 8.407.856 | 8,408 | 8,018 |

**b) e c) Quebra.** Medi com variantes de build, no mesmo perfil (`opt-level=z`, LTO fat, 1 codegen unit, `panic=abort`, strip), e com `cargo-bloat` (instalado) em um build separado com símbolos:

| Variante (abre xlsx e exporta PDF) | Bytes | MB |
|---|---|---|
| Código original, sem nada de PNG | 6.432.920 | 6,433 |
| Fase 1 (só `Rec`/`replay`) | 6.435.480 | 6,435 |
| PR, com o backend PNG ligado ao motor | 8.384.912 | 8,385 |
| `gc-min`, antes do crop | 8.388.368 | 8,388 |

- **Motor e PDF:** pesa cerca de 6,43 MB (6,14 MiB). Isso é o `gridcraft-engine` com planilha, fórmulas, xlsx, gráficos, layout de impressão e PDF, mais a `std`.
- **Backend PNG:** pesa 1.949.432 bytes, ou **1,95 MB** (1,86 MiB), que é a diferença entre a variante da fase 1 e a do PR.
- **Decomposição aproximada do PNG:** 1,51 MB de código (`.text`) e 0,24 MB de fontes (238.924 bytes), mais cerca de 0,20 MB de outras seções. Nada disso mede dados do `.rodata` por crate.
- **Código do PNG, por crate** (`cargo-bloat`, `.text`): `vello_cpu` 1,0 MiB, `vello_common` 151 KiB, `skrifa` 146 KiB, `gridcraft_raster` 57 KiB, `png` 42 KiB, `read_fonts` 31 KiB, `kurbo` 23 KiB.
- **Código do motor, por crate:** `gridcraft_engine` 1,7 MiB, `gridcraft_xlsx` 514 KiB, `std` 501 KiB, `gridcraft_functions` 373 KiB, `regex_automata` 232 KiB, `gridcraft_calc` 140 KiB, `image` 132 KiB.
- **`gc-min` sobre a variante PR:** o `main` do `gc-min`, que chama o PNG, só soma 3.456 bytes. O motor com `file.exportPng` registrado já puxa o backend inteiro, mesmo sem chamá-lo.

**d) A diferença para o mini Excel.** Confirmado: a diferença é o backend PNG e nada mais. O mini Excel do agente de unificação tinha 6.428.568 bytes, e minha variante sem PNG tem 6.432.920, ou seja, 4.352 bytes a mais (0,07%), pelo `main` e pelo lock um pouco diferentes. A diferença de 8.388.368 - 6.428.568 = 1.959.800 bytes (1,96 MB) se divide em 1.949.432 de PNG, 2.560 do `Rec`/`replay` e cerca de 7.800 do `main` e do lock. Rust e perfil são os mesmos (rustc 1.97, mesmos parâmetros de release).

**Unidades:** o 6,43 MB do outro agente é MB decimal (6,13 MiB). Os 8,388 MB do `gc-min` equivalem a 8,000 MiB.

## Caminhos

- `gc-min` novo (com `--crop`): `S/grid-png/a-refactor/feasibility/target/release/gc-min`
- PNGs recortados e `results.tsv`: `S/grid-png/a-refactor/out-crop/`
- Quebra por crate e variantes de build: `S/grid-png/a-refactor/feasibility/bloat-crates.txt` e `S/grid-png/a-refactor/feasibility/build-variants.log`

Parei aqui.
