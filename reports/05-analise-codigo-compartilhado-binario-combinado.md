# Código compartilhado entre os *craft e binário combinado

> Arquivo de relatório. Origem: subagente `ab39820d4829f5884`, turno 1 (único). Sessão de 2026-10-19.
> Tema: Análise de código compartilhado e do binário unificado: 26,60 MB contra soma de 32,87 MB (menos 19%).
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

Sim, o binário combinado fica menor que a soma dos três minis, mas só um pouco: 26,60 MB contra 32,87 MB, ou seja, 6,26 MB a menos (19,1%). Os três projetos compartilham pouco código próprio, então a economia vem quase toda das bibliotecas externas e da std. Compilei tudo com as mesmas otimizações de tamanho (opt-level=z, lto=fat, codegen-units=1, panic=abort, strip). O PowerPoint é o DeckCraft (`storytold/deckcraft`).

**Tamanhos (todos medidos, nenhum estimado):**

| Item | Bytes | MB |
|---|---|---|
| Mini Word (original) | 19.763.984 | 19,76 |
| Mini Word refeito no meu workspace | 19.770.832 | 19,77 |
| Mini PowerPoint (PNG do slide 1) | 6.673.136 | 6,67 |
| Mini Excel (abrir xlsx e exportar PDF, não é thumbnail real) | 6.428.568 | 6,43 |
| Soma dos 3 minis | 32.865.688 | 32,87 |
| Word + PowerPoint | 22.472.480 | 22,47 |
| Word + PowerPoint + Excel | 26.600.832 | 26,60 |

- **Curva:** o PowerPoint custa +2,70 MB sobre o Word, e o Excel custa +4,13 MB sobre os dois. Isso reaproveita 59% do mini PowerPoint e 36% do mini Excel.
- **Só Word + PowerPoint:** a economia é de 3,96 MB (15%).
- **Compressão:** compactando com xz como indicador (UPX não foi instalado), os três minis somam 10,56 MB e o combinado fica em 8,43 MB, também cerca de 20% menor.
- **Funcionamento:** o combinado escolhe por extensão e gera PNG idêntico byte a byte ao do binário Word + PowerPoint. Também abre `.doc` e gera PDF válido para `.xlsx`.

**Código compartilhado:**
- **Git:** Word e PowerPoint usam o mesmo commit do `craft-libs` (`c348732`) para `craft-pdf` e `krilla`. O Excel tem um escritor de PDF próprio e só usa `craft-libs` (outro commit, `d27646a`) na interface gráfica, fora dos minis.
- **Dependências dos minis:** o Word tem 157 crates externos, o PowerPoint 90 e o Excel 57, somando 304. O combinado tem 160, ou seja, 144 a menos. Só 41 crates são comuns aos três, e 87 são comuns entre Word e PowerPoint.
- **Lock dos repositórios inteiros:** têm 559, 455 e 578 pacotes (Word, Excel, PowerPoint), com 408 externos idênticos nos três, mas isso inclui a interface gráfica, que não entra nos minis.
- **O que dá para deduplicar:** a pilha de renderização e texto (`vello_cpu`, `skrifa`, `harfrust`, `kurbo`, `image`/`png`, `zip`, `quick-xml`, `serde_json`) e a `std`.
- **Lógica copiada e divergida, não compartilhada:** `geom`, `fonts`, `metafile`, `pdf` e `render` existem nos projetos como cópias separadas. Na comparação Word contra PowerPoint, nenhum arquivo de `geom`, `fonts` ou `pdf` é idêntico, e só 2 de 9 em `metafile`. Os três leitores OOXML também são independentes. O código próprio soma cerca de 154 mil linhas no Word, 53 mil no PowerPoint e 76 mil no Excel, sem nenhuma em comum.

**Fontes:**
- O WordCraft embute 13 TTFs (4.305.160 bytes) mais 617 KB de hifenização e ortografia.
- O DeckCraft e o GridCraft não embutem essas fontes. O DeckCraft só embute as de `craft-fonts` se `CRAFT_FONTS_DIR` estiver definido, o que não ocorreu aqui.
- Não há arquivo de fonte em comum para comparar por hash. O combinado paga as fontes do Word uma vez e nada dos outros dois.
- Se `craft-fonts` for usado nos dois, cada app embutiria a sua cópia, a menos que o crate de fontes seja unificado. Isso é uma dedução do código, não foi medido.

**Compensa?**
- Um binário único é mais enxuto no total, mas quem só precisa de PowerPoint carregaria 26,6 MB em vez de 6,7 MB.
- A compressão rende tanto quanto a unificação.
- Como o Excel não gera PNG, o mini Excel medido é só um substituto para o custo incremental. Um thumbnail real exigiria rasterizar o PDF por fora ou usar o exemplo `snapshot` do `ui-egui` (wgpu, dezenas de MB a mais, estimativa).

**Ressalvas:**
- O mini PowerPoint foi construído por mim, com uma cópia do código do outro agente. O build dele estava em perfil padrão, então não o usei. Nada em `pptx-test/` foi alterado.
- O lock do meu workspace foi semeado com a união dos três locks. O mini Word refeito ficou 0,03% maior que o original.
- Cada configuração foi medida uma vez, com as duas vCPUs compartilhadas com outro build.

Caminhos (todos em `S/combined-analysis/`):
- `RELATORIO.md`
- `ws/apps/all-min/src/main.rs`
- `bin/all-min-word+ppt+xlsx` (o combinado)
- `bin/all-min-word+ppt`
- `bin/all-min-word`
- `bin/all-min-ppt`
- `bin/all-min-xlsx`
- `bin/wc-min-original`
- `ext-word.txt`, `ext-ppt.txt`, `ext-xlsx.txt` e `ext-all.txt` (listas de dependências)

Para a memória do Cláudio: a pergunta de fundo foi respondida com medições e o resultado é "sim, ~19% menor, 26,6 MB contra 32,9 MB". A causa é a deduplicação de crates externos e da `std`, não as fontes.
