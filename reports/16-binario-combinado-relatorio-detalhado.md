# Binário único de thumbnails (Word, Excel e PowerPoint): relatório detalhado

> Arquivo de relatório. Origem: subagente `ab39820d4829f5884`, arquivo `combined-analysis/RELATORIO.md` (complementa o 05). Sessão de 2026-10-19.
> Tema: Relatório completo da análise de código compartilhado, com tabelas de tamanho, curva incremental, fecho de dependências e fontes embutidas.
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

# Binário único de thumbnails (Word + Excel + PowerPoint): vale a pena?

Projetos: WordCraft (`storytold/wordcraft`), GridCraft (`storytold/gridcraft`) e DeckCraft (`storytold/deckcraft`, o irmão de PowerPoint).
Todos os números de tamanho abaixo são MEDIDOS (rustc 1.97, release com opt-level=z, lto=fat, codegen-units=1, panic=abort, strip=symbols), exceto onde marcado.

## Resposta direta

Sim, o combinado fica menor que a soma, mas só um pouco: **26,60 MB contra 32,87 MB, ou seja, 6,26 MB a menos (19,1%)**. Não é uma redução dramática, porque a maior parte do tamanho é código próprio de cada projeto, que não é compartilhado.

## Tabela de tamanhos (bytes, MB = 10^6)

| Item | Bytes | MB | Status |
|---|---|---|---|
| Mini Word (wc-min original, PNG pág. 1) | 19.763.984 | 19,76 | medido |
| Mini Word refeito no meu workspace (feature `word`) | 19.770.832 | 19,77 | medido (valida o método: +0,03%) |
| Mini PowerPoint (PNG do slide 1) | 6.673.136 | 6,67 | medido |
| Mini Excel (abrir xlsx + exportar PDF, substituto) | 6.428.568 | 6,43 | medido, ver ressalva |
| **Soma dos 3 minis** | **32.865.688** | **32,87** | medido |
| Word + PowerPoint combinado | 22.472.480 | 22,47 | medido |
| **Word + PowerPoint + Excel combinado** | **26.600.832** | **26,60** | medido |

Curva incremental (ordem Word, depois PPT, depois Excel):

| Etapa | Custo incremental | Standalone | Economia por dedup |
|---|---|---|---|
| Word | 19,77 MB | 19,76 MB | 0 |
| + PowerPoint | +2,70 MB | 6,67 MB | 3,96 MB (59% do mini PPT é reaproveitado) |
| + Excel | +4,13 MB | 6,43 MB | 2,30 MB (36% do mini Excel é reaproveitado) |

Diferença total: soma 32.865.688 menos combinado 26.600.832 = **6.264.856 bytes (19,06%)**. Só Word+PPT: 3.964.640 bytes (15,0%).
Compactado com xz -9e (proxy do que UPX/LZMA faria, nada foi instalado): 3 minis = 6,26 + 2,17 + 2,13 = 10,56 MB; combinado = 8,43 MB (também cerca de 20% menor).

Sanidade funcional: o combinado gera PNG idêntico byte a byte ao do binário Word+PPT para .docx e .pptx, abre .doc, e gera PDF 1.7 válido para .xlsx. Extensão desconhecida retorna erro (exit 2).

## Código compartilhado (resumo com números)

**Git craft-libs:** os 3 usam o mesmo commit `c348732` para `craft-pdf` e `krilla 0.8.2` (Word e PowerPoint). O GridCraft só depende de craft-libs no commit `d27646a` (`craft-ui`, `craft-print`), apenas na GUI/impressão, fora dos minis. O Excel tem escritor de PDF próprio (`gridcraft-pdf`, 847 linhas, fontes padrão 14), então não usa craft-pdf/krilla. `craft-ui` e `craft-print` não entram em nenhum mini.

**Locks completos (workspace inteiro, inclui GUI/wgpu/egui):** Word 559 pacotes, Excel 455, PowerPoint 578. Pacotes externos com mesmo nome+versão+fonte nos três: 408. Comuns por par: Word/PPT 492, Word/Excel 435, Excel/PPT 410. Só 2 nomes comuns divergem em versão em todos os três (ex.: `vello_cpu` 0.1.0 vs 0.2.0, `harfrust` 0.12 vs 0.13). Mas isso inclui a GUI, que não está nos minis.

**Fecho de dependências dos minis (o que realmente importa, via `cargo tree`):**

| Conjunto | Crates externos (nome@versão) |
|---|---|
| Word | 157 |
| PowerPoint | 90 |
| Excel | 57 |
| Soma | 304 |
| União (combinado) | 160 |
| Comuns Word e PPT | 87 |
| Comuns aos três | 41 |

Ou seja, o combinado tem 144 crates externos a menos que a soma (-47%), mas o peso está em código próprio: Word ~154 mil linhas (fecho de 12 crates), PowerPoint ~53 mil, Excel ~76 mil, sem nenhuma linha em comum.

**O que é deduplicável de verdade (mesma versão):** `vello_cpu/vello_common 0.2.0`, `peniko`, `kurbo`, `fearless_simd`, `harfrust 0.13.3` e `skrifa 0.47/0.44` (Word e PPT, pilha de rasterização e texto), `image 0.25.10`/`png`/`flate2`/`zlib-rs`, `zip 3.0.0`, `quick-xml 0.41.0`, `serde`/`serde_json`, `thiserror`, `hashbrown`, `aes`/`sha1`/`sha2` (Word e Excel), `regex` (Word e Excel), além de `std`/`alloc`/panic runtime. O Excel não usa vello, skrifa nem harfrust (não rasteriza texto: PDF com fontes padrão).

**Não deduplicável apesar do mesmo "gênero":** o Word arrasta versões antigas via krilla (`skrifa 0.42.1`, `read-fonts 0.39.2`, `rustybuzz 0.20`, `ttf-parser 0.25`); o PPT não usa krilla no mini. Lógica duplicada entre crates diferentes, copiados e divergidos: `geom` (Word 743 linhas, PPT 5.576, 0 arquivos idênticos), `fonts` (5.703 vs 2.709, 0 idênticos), `metafile` EMF/WMF (3.212 vs 7.043, 2 de 9 arquivos idênticos), `pdf`, `render`, além de três modelos OOXML separados (docx, pptx, xlsx) com leitores de zip/XML independentes. Há parentesco evidente (mesma origem, mesmos nomes de módulos), mas são cópias, não crate compartilhado. Por isso o combinado não ganha nada nessa camada.

## Fontes embutidas

- **WordCraft:** 13 TTFs em `assets/fonts` via `include_bytes!` (Source Sans 3, Source Serif 4, Inter, JetBrains Mono): **4.305.160 bytes**, mais 617 KB de hifenização/ortografia (`assets/hyphenation`, `assets/spelling`). Isso é cerca de 25% do mini Word.
- **DeckCraft:** não embute nenhuma dessas fontes (política explícita de excluir Source Sans/Serif). Embute fontes só se `CRAFT_FONTS_DIR` apontar para o repo externo `storytold/craft-fonts` (em `.cargo/config.toml`, padrão `../craft-fonts`); sem o clone, o array fica vazio (foi o caso no build medido). Traz `epaint_default_fonts` (Ubuntu Light 362 KB como fallback).
- **GridCraft:** o motor não embute fontes (PDF com fontes padrão 14); só a GUI usa `epaint_default_fonts`.
- Hash: não há arquivo de fonte em comum para comparar, então as fontes **não** são compartilhadas hoje. O combinado paga os 4,3 MB do Word uma vez e nada dos outros dois (sem craft-fonts). Atenção: se um dia `CRAFT_FONTS_DIR` for definido, `wordcraft-fonts` e `deckcraft-fonts` embutiriam cada um a sua cópia (os `include_bytes!` são de crates distintos), e o combinado pagaria as fontes duas vezes a menos que se unifique o crate de fontes. Isso é uma estimativa da mecânica, não medida.

## Causa provável do ganho

Medido indiretamente (sem cargo-bloat): Word+PPT economiza 3,96 MB graças ao fecho de 87 crates comuns (vello_cpu/vello_common, harfrust, skrifa 0.47, kurbo, image/png, zip, quick-xml, serde_json) mais std/alloc. O Excel economiza 2,30 MB (zip, quick-xml, image/png, serde_json, regex, aes/sha, std). Fontes não contribuem para a economia (só existem no Word). O ganho absoluto é limitado pelo código próprio, que é a maior parte: o mini Word tem ~4,9 MB de dados embutidos (fontes e hifenização) e o resto é código.

## Alternativas e conclusão prática

- Um binário único (26,6 MB) economiza 6,3 MB (19%) frente a três (32,9 MB). Compensa se o objetivo é distribuir um só executável ou reduzir disco; não compensa se o critério é tamanho por tipo de arquivo: quem só precisa de PPT carregaria 26,6 MB em vez de 6,7 MB.
- Compressão ajuda mais que a unificação: xz leva o combinado a 8,4 MB (UPX tem ordem de grandeza parecida, mas não foi instalado nem medido, é só proxy). Combinar ambas as técnicas dá o menor resultado.
- O Excel sem PNG nativo muda a conclusão para pior no combinado real: o mini Excel medido é um substituto (PDF, rasterização externa). Um thumbnail PNG real exigiria rasterizar o PDF (por exemplo, com um crate de PDF para imagem) ou usar o exemplo `snapshot` do `ui-egui` (wgpu, dependências de GUI, tamanho muito maior); o ganho de dedup subiria (mais vello/skrifa em comum) mas o custo absoluto também. Estimativa, não medida: dezenas de MB só pela GUI.
- Melhor caminho para encolher de verdade: unificar crates de fontes/geom/metafile/PDF entre os repositórios e alinhar as versões de skrifa/read-fonts (4 versões de `read-fonts` no combinado), e mover as fontes do Word para fora do binário.

## Ressalvas

- O mini Excel não gera PNG nem thumbnail: é um substituto (abrir xlsx e exportar PDF) para medir o custo incremental.
- O mini PowerPoint foi construído por mim com as mesmas otimizações (o outro agente estava construindo em perfil padrão, não otimizado); usei uma cópia do código do `dc-min` dele, sem alterar nada em `pptx-test/`.
- O workspace combinado resolve versões a partir de uma união dos três `Cargo.lock` (seed), e o resultado é um lock novo; as versões coincidem com as dos locks originais para o que foi comparado, e o mini Word refeito ficou 0,03% maior que o original.
- Resultados de dedup valem para este conjunto de minis, com `CRAFT_FONTS_DIR` ausente. Medição única por configuração (sem repetição), com 2 vCPUs compartilhadas com outro build, então tempos não são comparáveis.
- Não instalei UPX nem cargo-bloat; o xz é apenas indicativo.

## Caminhos

- Relatório: `S/combined-analysis/RELATORIO.md`
- Workspace: `S/combined-analysis/ws` (código em `apps/all-min/src/main.rs`)
- Binário combinado: `S/combined-analysis/bin/all-min-word+ppt+xlsx`
- Binário Word+PPT: `S/combined-analysis/bin/all-min-word+ppt`
- Minis: `S/combined-analysis/bin/all-min-word`, `S/combined-analysis/bin/all-min-ppt`, `S/combined-analysis/bin/all-min-xlsx`, `S/combined-analysis/bin/wc-min-original`
- Listas de fecho de dependências: `S/combined-analysis/ext-word.txt` (e `ext-ppt.txt`, `ext-xlsx.txt`, `ext-all.txt`)
