# Playbook: reabrir o thumbcraft e avaliar a atualização dos *craft

Use este roteiro quando pedir: "reabra o thumbcraft e veja a viabilidade de atualizar os *craft". Os *craft (WordCraft, GridCraft e DeckCraft) continuam evoluindo, e o projeto existe como ponte: o GridCraft ainda não gera PNG (por isso há patch), o DeckCraft não lê .ppt nem .odp, e o PDF não é suportado. Uma atualização pode fazer o patch sair, melhorar algumas coisas e não outras.

Os números de 2026-10-19 contra os quais comparar estão em `RESULTS.md` e `../baselines/`. Os comandos `scripts/*.sh` abaixo existem no repositório na data deste documento; confirme com `ls scripts` e `--help`.

## 0. Preparação

1. Leia `../UPSTREAM.md` (SHAs em uso, datas e tamanho do binário) e `BUILD.md`. Leia também `PITFALLS.md` e `ENVIRONMENT.md`.
2. Confira o ambiente: Rust, `qpdf`, `pdftoppm`, ImageMagick e LibreOffice (para baseline novo), e a rede para GitHub e registries (ver `ENVIRONMENT.md`).
3. Crie uma branch. Anote a data e o SHA do thumbcraft antes de mexer.

## 1. Descobrir o que mudou nos upstream

Para cada projeto, busque o SHA mais recente do `main` e compare com o fixado em `../scripts/fetch-upstream.sh`:

- WordCraft (`117aea31a8e61bd3e306e9210800cdc42897e3c1`), GridCraft (`66ce6be4743f4227047f76f56815520f824e6d81`), DeckCraft (`010941ec60f81f1eece31ec1ea83825c9a396217`).
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
4. Alinhe as revisões do `craft-libs` e do hayro com o `Cargo.toml` raiz, se mudarem.

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
