# Histórico do projeto

Narrativa cronológica da sessão de 2026-10-19 que originou o thumbcraft. Horários aproximados no fuso local (UTC-3), derivados dos carimbos dos relatórios dos subagentes. Cada fase aponta para o relatório correspondente em `../reports/` e para os dados em `../baselines/` e `../corpus/`. As decisões estão em `DECISIONS.md`, os testes em `TESTS.md` e os números em `RESULTS.md`.

## Fase 1: o WordCraft gera PNG pela CLI? (cerca de 06:25 a 06:55)

Pedido do usuário: confirmar se o WordCraft converte doc em PNG pela linha de comando só com o core (sem GUI, servidor ou extras) e qual o tamanho do binário mínimo.

- Não existe um crate "core" isolado. O `wordcraft-cli` não tem GUI nem servidor, e `convert in out.png` chama `file.exportPng` (página 1). O rasterizador é `vello_cpu` (Rust puro, sem GPU).
- Foi escrito um binário de 15 linhas (`wc-min`) que depende só de `wordcraft-engine`, `wordcraft-doc` e `serde_json`.
- Resultado: o mini Word tem 19.763.984 bytes (19,76 MB, 18,85 MiB). O CLI completo com o perfil do repositório tem 42,60 MB. Cortar `mcp` e `zotero` economiza só 0,22 MB, porque o peso vem do motor (comandos, leitores e escritores, krilla, 4,3 MB de fontes embutidas e dicionários).
- Relatório: `../reports/01-wordcraft-binario-minimo.md`.

## Fase 2: thumbnails em vários formatos (cerca de 06:50 a 07:50)

Pedido: testar o thumbnail em vários formatos de entrada e fazer o mesmo para os irmãos Excel e PowerPoint, contra o LibreOffice.

- Word: 30 arquivos (docx, doc 97, odt, rtf, html, md, tex, txt, inválidos). 25 de 30 geraram PNG, média 0,12 s contra 1,48 s do LibreOffice. Relatório 02.
- Excel (GridCraft): a investigação parou no primeiro passo, porque a CLI do GridCraft não gera PNG (só PDF). Relatório 03.
- PowerPoint (DeckCraft): 46 arquivos, 38 de 46 com PNG, média 83 ms contra 1792 ms do LibreOffice, mini de 6,67 MB. Não lê .ppt nem .odp. Teste extra de 48 variantes mutadas sem panic. Relatório 04.

## Fase 3: código compartilhado e binário unificado (cerca de 07:00 a 07:40)

Pedido: analisar o código compartilhado e a viabilidade de um binário único.

- Cinco builds com as mesmas otimizações de tamanho. O combinado Word, PowerPoint e Excel (este só exportando PDF) tem 26,60 MB contra a soma de 32,87 MB dos três minis (menos 19%).
- O código próprio não é compartilhado: o que se reaproveita são crates externos e a `std`. O `craft-libs` é a única dependência compartilhada, e cada projeto o fixa em uma revisão diferente.
- Relatórios 05 e 16, anexos em `../reports/anexos/combinado-all-min/`.

## Fase 4: quanto custa um PNG no GridCraft (cerca de 07:05 a 07:35)

Pedido: o custo de código para o GridCraft gerar PNG.

- Foram comparados os caminhos A (trait de canvas com `vello_cpu`), B (cena nova), C (PDF e depois rasterização com `craft-pdf` raster) e E (cena simplificada com texto em barras). O agente recomendou o C por ser o menor diff.
- Descobertas: o WordCraft tem uma display list `Draw` compartilhada por PDF e PNG (relatório 08), enquanto o GridCraft desenha em modo imediato direto no PDF (relatório 07).
- O usuário observou que as estimativas em "dias" estavam exageradas para poucas centenas de linhas. Relatório 06.

## Fase 5: corpus e linha de base de planilhas (cerca de 07:45 a 07:50)

Montagem de 57 planilhas de teste (42 válidas, 6 de formatos alternativos, 9 inválidas) e da linha de base do LibreOffice (54 PNGs, média 1653 ms, página A4 de 795x1124 px). Relatórios 09 e 17.

## Fase 6: protótipo gp-min, Caminho C (cerca de 07:50 a 08:45)

PDF do GridCraft rasterizado com `craft-pdf` raster (hayro) em um binário só, de 11,41 MB. Passou por três ajustes pedidos pelo usuário:

1. A3, recortado ao conteúdo (relatório 10).
2. A4 em vez de A3 (relatório 11).
3. Correção de requisito: o thumbnail é a primeira página A4 sem zoom, mostrando só o que couber, como o LibreOffice (relatório 12).

Achados de dependência: o commit `c3487320` do craft-libs só tem o writer de PDF, e o `raster` só existe a partir do `main` (commit `c07b3fa`); o hayro com patches em `vendor/` precisa ser repetido no `Cargo.toml` raiz.

## Fase 7: refatoração do GridCraft em duas fases (cerca de 08:00 a 09:55)

Decisão do usuário: trabalhar em fases e manter separadas três frentes (refatoração que possa virar PR, viabilidade de thumbnails, unificação).

- Fase 1 (relatório 13): só `print.rs` ganha `Op`, `Rec` e `replay`. Regressão do PDF em três camadas, 325 de 325 casos PASS, 306 PDFs idênticos byte a byte, 0 pixels diferentes. Teste de sanidade do detector com defeito injetado (99 FAIL). Dados em `../baselines/gridcraft-pdf-regression/`.
- Fase 2, com liberação para criar arquivos (relatório 14): `Op`/`Rec`/`replay` foram para `crates/pdf/src/display.rs`; novo crate `gridcraft-raster` com `vello_cpu` 0.2.0; comando `file.exportPng`; CLI com `.png`. PR de 32 arquivos, +1043 e -33. O esboço com `tiny-skia` foi descartado. Padrões de thumbnail (A4, escala 2) ficaram no `gc-min`, fora do PR.
- Comparação final em três colunas (LibreOffice, `gp-min`, `gc-min`), com erro de cor entre 0,6% e 1,5% contra o protótipo.
- Crop e quebra de tamanho (relatório 15): o usuário apontou que o crop era trivial, e foi implementado no `gc-min` (+0,019 MB). O tamanho do `gc-min` (8,39 MB) se divide em motor e PDF com 6,43 MB mais backend PNG com 1,95 MB. O "8,00 MB" do relatório anterior era MiB.

## Fase 8: criação do thumbcraft (cerca de 10:05 em diante)

Pedido: criar o projeto thumbcraft com o binário unificado, documentação de tudo, corpus, comparações e como refazer, para uso futuro em um fluxo de upload de outro projeto. Em paralelo:

- um construtor implementa o binário (`crates/`, `scripts/`, `patches/`);
- um agente de PDF avalia o `pdf-parse` do Node (getScreenshot) contra os *craft e se o `craft-pdf` já está nos binários atuais;
- este arquivamento guarda corpora, baselines, relatórios e documentação.

A sessão foi interrompida por falta de tokens e retomada depois (ver `STATUS.md`). Decisões posteriores: o repositório guarda só código, patches, documentação, método e resultados (sem benchmarks, binários ou arquivos grandes), e o CI/CD de release para Windows e Ubuntu 24 é etapa futura (ver `UPDATING.md`).
