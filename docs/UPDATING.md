# Playbook: reabrir o thumbcraft e avaliar a atualização dos *craft

Use este roteiro quando pedir: "reabra o thumbcraft e veja a viabilidade de atualizar os *craft". Os *craft (WordCraft, GridCraft e DeckCraft) continuam evoluindo, e o projeto existe como ponte: o GridCraft ainda não gera PNG (por isso há patch), o DeckCraft não lê .ppt nem .odp, e o PDF não é suportado. Uma atualização pode fazer o patch sair, melhorar algumas coisas e não outras.

Os números de 2026-10-19 contra os quais comparar estão em `RESULTS.md` e `../baselines/`. Os comandos `scripts/*.sh` abaixo existem no repositório na data deste documento; confirme com `ls scripts` e `--help`.

## 0. Preparação

1. Leia `../UPSTREAM.md` (SHAs em uso, datas e tamanho do binário), `UPSTREAM-CHANGES.md` (todo patch e remendo nos projetos importados, e como saber se ainda é necessário) e `BUILD.md`. Leia também `PITFALLS.md` e `ENVIRONMENT.md`.
2. Confira o ambiente: Rust, `qpdf`, `pdftoppm`, ImageMagick e LibreOffice (para baseline novo), e a rede para GitHub e registries (ver `ENVIRONMENT.md`).
3. Crie uma branch. Anote a data e o SHA do thumbcraft antes de mexer.

## 1. Descobrir o que mudou nos upstream

Para cada projeto, busque o SHA mais recente do `main` e compare com o fixado em `../scripts/fetch-upstream.sh`:

- WordCraft (`e6a5cd05f045e4caf25a28660bfeb622f25b5129`), GridCraft (`67e39faeea1fa4b603cbab421a30737db2cef8d9`), DeckCraft (`a3fbf02952839c223df600dc33106a2dbc65be1b`), craft-libs (`c39e435cc34417843793df240a0b38bac970179b`), todos de 2026-10-31.
- `git ls-remote https://github.com/storytold/<projeto> HEAD` (a API `gh api` da organização foi bloqueada com 403 na sessão original; `git clone` e `git fetch` funcionam).
- Leia o changelog, o `ROADMAP.md`, o `docs/cli.md` e o `docs/gaps.md` de cada um, e os commits desde o SHA fixado (`git log --oneline SHA_ANTIGO..SHA_NOVO`). Procure por "png", "thumbnail", ".ppt", ".odp", "pdf", "render" e "raster".

Perguntas a responder e o que fazer em cada caso:

| Pergunta | Se sim |
|---|---|
| O GridCraft ganhou exportação PNG nativa (`file.exportPng`, `convert x.xlsx y.png`)? | Remover o patch (`../patches/gridcraft/`) e o `ARVORE_GRIDCRAFT` do `fetch-upstream.sh`, adaptar o adaptador do GridCraft ao comando nativo e comparar com o patch (item 5). Se ele usa o mesmo `vello_cpu` 0.2, o tamanho deve ser parecido. Registre em `PR-GRIDCRAFT.md` que o PR deixou de ser necessário. |
| O PR do patch foi aceito (total ou parcial)? | Mesmo caminho: remover ou reduzir o patch. Veja se `Op`/`Rec`/`replay` e o crate `gridcraft-raster` entraram com outros nomes. |
| O GridCraft mudou o `print.rs`, o `Page` ou o crate `pdf`? | O patch pode não aplicar. Reaplicar à mão e rodar a regressão do PDF (item 4). |
| O DeckCraft passou a ler .ppt ou .odp? | Reexecutar os arquivos 35 a 39 do corpus PowerPoint e atualizar `README.md` e `PITFALLS.md` (lista de formatos não suportados). |
| O DeckCraft passou a reportar slide 0 perdido como erro? | A mitigação do thumbcraft (tratar `lost` do slide 0 como erro) pode sair. O caso 44 do corpus deixa de ser "ESPERADO" no `compare-native.sh`. |
| O WordCraft embutiu fontes Microsoft equivalentes, melhorou o `.doc` (tabela sem bordas) ou passou a carregar imagens relativas de md? | Refazer o corpus Word e comparar (casos 06, 09, 14). |
| O `craft-libs` mudou a revisão fixada, ou o `craft-pdf` ganhou `read` ou `raster` na revisão fixada? | Ver `CRAFT-FAMILY.md`. Se o `raster` existir na revisão fixada de todos, a avaliação de PDF (`PDF.md`) pode mudar; repita-a. Lembre dos patches do hayro em `vendor/`, que o crate raiz precisa repetir. |
| Algum projeto passou a suportar PDF como entrada? | Registrar em `README.md` e `PDF.md`. |
| Mudaram a versão do Rust mínima, a `edition` ou as versões de `vello_cpu`, `kurbo`, `peniko`, `skrifa`? | Verificar se ainda deduplicam (ver `CRAFT-FAMILY.md`); versões diferentes entre os projetos aumentam o binário. |

## 2. Atualizar os SHAs e reaplicar os patches

1. Edite a tabela `PROJETOS` do `../scripts/fetch-upstream.sh` (SHA e data) e rode o script. Ele falha alto se o SHA não existir ou se a árvore patchada do GridCraft diferir do `ARVORE_GRIDCRAFT`.
2. Se o patch do GridCraft não aplicar: aplique em um clone limpo, resolva os conflitos à mão (o ponto sensível é o `print.rs`), gere de novo o patch e atualize `ARVORE_GRIDCRAFT` (o script descreve como calcular o hash da árvore).
3. Atualize `../UPSTREAM.md` e o `Cargo.lock` (`--locked` no build).
4. O `craft-libs` também está na tabela `PROJETOS` (`third_party/craft-libs`); se mudar o SHA, confira o patch `patches/craft-libs/` e os remendos do `Cargo.toml` raiz. Depois confira que o binário segue com uma versão por pacote (`cargo tree -p thumbcraft -d`) e atualize `UPSTREAM-CHANGES.md`.

## 3. Compilar e medir o tamanho

`scripts/build.sh` (perfil de tamanho: `opt-level=z`, LTO fat, `codegen-units=1`, `panic=abort`, `strip=symbols`). Anote os bytes, MB decimais e MiB. Compare com `RESULTS.md` e `SIZES.md`. Um build limpo leva dezenas de minutos em 2 vCPUs; compilar com um `CARGO_TARGET_DIR` compartilhado serializa os builds.

## 4. Regressão do PDF do GridCraft

Se o GridCraft mudou, gere novos goldens com o novo commit original (sem patch) e confira o patch contra eles: siga `../baselines/gridcraft-pdf-regression/README.md`. Critério: 325 de 325 PASS, PDFs idênticos byte a byte (ou, se o upstream mudou o PDF de propósito, os goldens refeitos com o novo original mostrarem 325 de 325). Sempre rode também o teste de sanidade do detector (injetar um defeito e ver FAIL).

## 5. Rodar os corpora e comparar com os baselines

1. Nativo contra os minis: `scripts/compare-native.sh` (renderização do thumbcraft com `--native-scale 2` contra `wc-min`, `dc-min` e `gc-min`, byte a byte). Para o GridCraft sem patch (nativo novo), a comparação é contra o PNG do `gc-min` antigo e do LibreOffice.
2. Corpora completos: `scripts/run-corpus.sh corpus/docx saida/docx`, `corpus/pptx` e `corpus/xlsx` (`TIMEOUT=30`); resumo por motor com `scripts/summarize.sh saida/<tipo>/results.tsv`; contact sheet com `scripts/sheets.sh`.
3. Compare cada `results.tsv` com os de `../baselines/word/`, `../baselines/powerpoint/` e `../baselines/excel/resultados/` (taxa de sucesso, tempo médio e máximo, bytes do PNG).
4. Se quiser refazer a linha de base do LibreOffice (nova versão do LibreOffice, por exemplo): `../baselines/excel/ferramentas/run-lo.sh` e o script do PowerPoint em `../baselines/powerpoint/ferramentas/run.sh`.
5. Inspeção visual: compare contact sheets antigas (`../baselines/*/imagens/`) com as novas, e olhe os casos que mudaram (use `compare.sh` em `../baselines/excel/ferramentas/`).
6. Repita o teste de mutação do DeckCraft (`../corpus/pptx/mutacao-48/`) e confirme ausência de panic.
7. **Antes contra depois, pixel a pixel** (feito em 2026-10-31, resultado em `../baselines/thumbcraft-run-2026-10-31/`):
   compile o `main` com os SHAs antigos (`CARGO_TARGET_DIR=<outro> cargo build --release --locked --manifest-path <main>/Cargo.toml -p thumbcraft`)
   e o ramo novo; rode `scripts/run-corpus.sh` com os dois binários (`RECURSIVO=1`; modos `--native-scale 2`, `-W 480 -H 480 --fit contain`
   e `-W 120 -H 120`; o PDF não tem modo nativo) e compare cada par de pastas com `scripts/compare-runs.py ANTES DEPOIS --out DIR`.
   Qualquer `NOVO_ERRO`, `CODIGO` ou `DIMENSAO` bloqueia a atualização até ser explicado; `GRANDE` e `MEDIA` pedem olhar as imagens
   de `DIR/pares/`. Para saber se uma diferença do GridCraft é do upstream ou do nosso patch, exporte o PDF de cada planilha do corpus
   com a CLI do commit sem patch e com patch (`gridcraft-cli convert x.xlsx x.pdf`) e compare os bytes: o patch não pode mudar o PDF.
8. **Guardas que rodam em `cargo test`:** `crates/core/tests/dependencies.rs` (uma versão por pacote no `Cargo.lock`; falha na hora se
   um motor novo trouxer `miniz_oxide`, `skrifa`, `read-fonts` etc. duplicados) e `crates/thumbcraft/tests/layout.rs` (caixa de tinta,
   dimensões e quantidade de tinta de 16 arquivos de referência contra `layout-<plataforma>.tsv`). Se a mudança de layout for esperada,
   confira as imagens e regrave com `THUMBCRAFT_RECORD=1 cargo test -p thumbcraft --test layout`; há um arquivo por plataforma
   (`layout-windows.tsv` gravado em 2026-10-31; o `layout-linux.tsv` falta e o teste só avisa até alguém gravá-lo no Linux).

## 5.1. Onde uma atualização quebra

Pontos que já quebraram ou mudaram de comportamento (versões 0.8 para 0.13/0.14), na ordem em que aparecem:

| Onde | Sintoma | O que fazer |
|---|---|---|
| `patches/gridcraft` (`print.rs`, `Page`, `crates/pdf`) | O patch não aplica. O upstream dividiu `print.rs` em `print/*.rs` e mudou a API do `Page` (`Point`, `Rect`, `link_uri`, `link_page`) | Aplicar no commit antigo, `git cherry-pick` no novo, resolver à mão; o `Rec` em `crates/pdf/src/display.rs` tem de espelhar a API do `Page`. Prova: o PDF com patch igual ao PDF sem patch |
| `stubs/wordcraft-pdf` | Erro de compilação em `PdfOptions` ("no field named") | Acrescentar o campo ao stub (em 2026-10-31: `links`, `numeral`) |
| Pares `skrifa` e `read-fonts` | Dois `read-fonts` no binário: o `harfrust` 0.13.3 pede o 0.43 e o `skrifa` 0.47 pede o 0.44 | Alinhar o `skrifa` por patch ao `read-fonts` do `harfrust` (patches 1 a 5 de `UPSTREAM-CHANGES.md`); a API muda pouco (`.ok()` em `glyph_id`) |
| `miniz_oxide` | O WordCraft passou a pedir 0.9; `flate2` e `png` ficam no 0.8 | Patch do WordCraft baixa para 0.8 |
| `craft-libs` `vendor/hayro` | O patch do `vello_cpu` não aplica | Refazer as trocas de `render_to_pixmap` e `render_to_buffer` (patch 2) |
| Layout do Word | Texto mais compacto na vertical, quebras de página diferentes | Esperado entre versões; regravar `layout-<plataforma>.tsv` depois de olhar `pares/` |
| Layout do Excel | Banner cortado, gráfico menor, texto não latino com espaços | Comportamento do GridCraft 0.14.2 (o PDF sem patch já faz o mesmo); acompanhar no upstream |
| Tamanho do binário | +6% na atualização de 2026-10-31 (25,67 para 27,22 MB) | Investigar com `cargo-bloat` quando passar do esperado |

## 6. Critérios de aceitação

Atualizar se, contra `RESULTS.md`:

- Taxa de sucesso por motor igual ou melhor: Word 25 de 30 (as 5 falhas são entradas inválidas), PowerPoint 38 de 46 (8 falhas, sendo 5 formatos não suportados), Excel 52 de 55 no `gc-min` e 54 de 57 no corpus.
- Tempo médio não pior que cerca de 1,5 vez o baseline (Word 0,12 s, PowerPoint 98 ms nos sucessos, Excel 134 ms em escala 2). Lembre que as medições originais têm ruído (2 vCPUs compartilhadas).
- Tamanho do binário unificado dentro do esperado (referência: 26,60 MB do combinado sem o PNG do Excel, mais o backend PNG; ver `SIZES.md`). Qualquer salto grande merece investigação com `cargo-bloat`.
- Regressão do PDF 325 de 325 e `compare-native.sh` sem FAIL.
- Sem regressão visual: nenhum caso que era "ok" passa a "degradado" ou "falhou". Diferenças intencionais devem ser listadas e justificadas.
- Sem panic em entradas inválidas e nas 48 mutações.

Não atualizar (ficar no SHA antigo) se houver regressão que o upstream não corrige; registre o motivo.

## 7. O que registrar ao final

- `../UPSTREAM.md`: SHAs novos, datas e tamanho.
- `HISTORY.md`: nova fase com data, o que foi avaliado e a conclusão. `DECISIONS.md`: decisões (por exemplo, remover o patch). `TESTS.md` e `RESULTS.md`: novos testes e tabelas, datadas, mantendo as de 2026-10-19 para comparação.
- `../baselines/`: novo subdiretório datado (por exemplo `../baselines/excel/resultados-AAAA-MM-DD/`) em vez de sobrescrever os atuais.
- `PR-GRIDCRAFT.md`: estado do PR (aceito, aberto, abandonado).
- `PITFALLS.md` e `CRAFT-FAMILY.md`: limitações que mudaram.
- Reports: um relatório novo em `../reports/` com o id do agente, se usou subagentes.

## 8. Pendência futura: CI/CD de release

Etapa futura, só depois que o binário unificado estiver funcionando: criar um workflow do GitHub Actions que gere o binário de release para Windows e Ubuntu 24. Pontos a resolver:

- fetch dos upstream em SHAs fixos e aplicação do patch do GridCraft dentro do workflow (`scripts/fetch-upstream.sh`);
- cache do cargo (e cuidado com o tamanho do cache, o build limpo dura dezenas de minutos);
- perfil de release para tamanho (`opt-level=z`, LTO fat, `codegen-units=1`, `panic=abort`, `strip=symbols`) e relatório do tamanho do artefato em MB decimais e MiB;
- uma matriz Windows e Ubuntu 24 (os testes de tempo e tamanho de 2026-10-19 foram só em Linux x86_64; no Windows, a busca de fontes do sistema e a ausência de Carlito e Caladea mudam a fidelidade);
- publicação dos artefatos em uma release do GitHub;
- rodar `compare-native.sh` e os corpora no CI (ou um subconjunto), para detectar regressão a cada atualização dos SHAs.
