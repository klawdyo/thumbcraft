# 08 — Motor de PDF (hayro via craft-pdf) no binário unificado

## Objetivo

Fazer o `thumbcraft` gerar miniaturas de PDF (primeira página, ou a pedida por `--page`) com o Hayro através do
crate `craft-pdf` (feature `raster`), dentro do binário unificado, com as mesmas opções de corte, ajuste, redução e
formato de saída dos outros motores. Medir no Windows as proteções contra PDFs hostis e o custo de tamanho.

## Contexto

Decisão do dono (2026-10-19), a partir de `docs/PDF-WINDOWS.md` (comparação de hayro, pdfium, pdf-parse, mutool e
Poppler no Windows) e de `docs/PDF.md` (Linux): hayro pelo `craft-pdf` dentro do binário. Pontos conhecidos:

- Hayro usado direto, sem o `craft-pdf`, não se protege: 1.061 MB na bomba de fluxo e 2.055 MB na de imagem
  (Windows). O `craft-pdf` impõe limites (entrada de 512 MiB, limites de decodificação, saída de no máximo 8192 px
  de lado e 64 megapixels, orçamentos de imagem e de conteúdo); em Linux o `pdf-min` ficou com 10 a 12 MB nesses
  casos (`docs/PDF.md`). Isso ainda não foi medido no Windows nem dentro do thumbcraft.
- O WordCraft e o DeckCraft usam o `craft-pdf` na revisão `c3487320`, só com a feature `write`. A feature `raster`
  exige a revisão `ae2f796b673f29cf3161f36eb6a8933f0a3fe028`, e portanto duas revisões do `craft-libs` no mesmo
  binário, mais os patches dos crates `hayro*` repetidos no `Cargo.toml` raiz (o cargo ignora `[patch]` de
  dependências). O Hayro dessa revisão usa `vello_cpu` 0.0.8 enquanto o thumbcraft usa 0.2.0: o rasterizador entra
  duas vezes. Custo estimado em `docs/PDF.md`: cerca de 4,2 MB (Linux); em `docs/PDF-WINDOWS.md`: de 4 a 6,6 MB.
- Ponto de partida: o roteamento reconhecia `%PDF` (`Sniff::Pdf`) e `open.rs` respondia "PDF ainda não é
  suportado" (código 3). Agora `Engine::Pdf` abre pelo `crates/thumbcraft/src/pdf.rs`.

## Referências

- `docs/PDF-WINDOWS.md`, `docs/PDF.md`, `docs/CRAFT-FAMILY.md`, `docs/ARCHITECTURE.md`
- `crates/core/src/route.rs`, `crates/thumbcraft/src/open.rs`, `word.rs` (modelo de adaptador `Render`), `info.rs`
- API do `craft-pdf` na revisão `ae2f796` (`open_slice_with_options`, `raster::render`, `raster::Options`, `Diagnostics`)
- `corpus/pdf/` (45 PDFs com `MANIFEST.tsv`) e `benchmarks/pdf/` (executor com medição de memória; fora do git)
- `docs/rules/testing.md`, `principles.md`, `naming-and-language.md`

## Regra do dono: uma versão por pacote (2026-10-19)

O thumbcraft junta projetos para ficar **pequeno, padronizado e eficiente**; versões diferentes do mesmo pacote no
binário não cabem em nenhuma das três. Então esta tarefa **não aceita** duas revisões do `craft-pdf` nem duas
versões do `vello_cpu` (e de `vello_common`, `glifo`) no binário. Como resolver, com o mínimo de mudança nos projetos
importados (o núcleo deles não é reescrito):

1. Baixar o `craft-libs` na revisão `ae2f796b673f29cf3161f36eb6a8933f0a3fe028` para `third_party/craft-libs`
   (`scripts/fetch-upstream.sh`), como já se faz com WordCraft, GridCraft e DeckCraft.
2. Usar essa única cópia para tudo: `craft-pdf` (com `write` e `raster`) e os crates `krilla` e `hayro*` vendorizados,
   por caminho. O `[patch]` do `craft-pdf` na raiz faz o WordCraft e o DeckCraft, que pedem a revisão `c3487320` por
   git, usarem a mesma cópia.
3. Portar o `hayro` vendorizado para o `vello_cpu` 0.2.0 (o que os motores já usam), num patch pequeno em
   `patches/craft-libs/`, só nos dois arquivos que usam o `vello_cpu` (`lib.rs` e `renderer.rs`).
4. **Registrar toda mudança** em `docs/UPSTREAM-CHANGES.md` (o que mudou, por quê, como reaplicar, como saber se
   ainda é necessário), para os próximos agentes saberem o que fazer quando o dono pedir para atualizar os pacotes.
5. Duplicações que já existiam antes desta tarefa (por exemplo `skrifa` em três versões) são listadas no mesmo
   documento, com a causa e o que seria preciso para resolver; resolver só as que o PDF introduziu é o mínimo desta
   tarefa, e o restante vira tarefa própria se o dono pedir.

## Regras de negócio

- Um arquivo é PDF quando começa com `%PDF` ou tem extensão `.pdf` sem outra assinatura. Texto puro com extensão
  `.pdf` continua indo para o WordCraft (tolerância já existente). Um arquivo `.pdf` que não é PDF (por exemplo uma
  imagem) vai para o motor de PDF e falha com código 4, não 3.
- `--detect` imprime `pdf`; `--formats` lista PDF como suportado; `--version` informa a revisão do `craft-pdf`.
- Abertura com os limites padrão do `craft-pdf`; PDF com senha de abertura, truncado, vazio ou sem páginas falha
  com código 4 e mensagem em português; PDF criptografado sem senha de abertura abre normalmente.
- `--page N` renderiza a página N (a partir de 1); página inexistente é erro 4 (já tratado pelo pipeline).
- Fundo branco opaco (uma página de PDF não é transparente). A escala vem do pipeline comum e respeita os limites do
  `craft-pdf` (8192 px e 64 megapixels), que coincidem com os do núcleo.
- Quando o Hayro avisa que omitiu conteúdo ou falhou ao decodificar uma imagem (`content_truncated`,
  `image_decode_failed`), a miniatura é gerada (código 0) e uma linha de aviso em português vai para a stderr; a
  stdout continua sem texto.
- `panic = "abort"` continua valendo: o motor não captura pânico além do que o `craft-pdf` já captura.
- Tamanho e memória: meta de pico nos PDFs hostis definida depois da primeira medição e registrada abaixo; sem
  ela a tarefa não termina.

## Detalhes técnicos

- Dependência `craft-pdf` renomeada (`package = "craft-pdf"`), `default-features = false`, `features = ["raster"]`,
  git e revisão fixos; patches `hayro`, `hayro-interpret`, `hayro-syntax` e `hayro-jbig2` no mesmo git e revisão.
  `Cargo.lock` atualizado e versionado (o build usa `--locked`).
- Novo `crates/thumbcraft/src/pdf.rs` com um tipo que implementa `Render` (como `word.rs`); a leitura do arquivo e o
  desenho ficam atrás de funções pequenas que os testes unitários exercitam com PDFs mínimos montados em código.
- `route.rs` (núcleo): extensão `pdf` roteia para `Engine::Pdf`; remover `pdf_pending`.

## Critérios de aceitação

- [x] Testes do núcleo (roteamento de PDF por conteúdo, por extensão, com lixo antes do cabeçalho e texto com extensão `.pdf`) escritos antes, falhando pelo motivo certo, depois passando
- [x] Testes unitários do adaptador, com PDFs mínimos em código, sem disco: tamanho da página, número de páginas, página inexistente, PDF sem páginas e dados inválidos
- [x] Testes e2e sobre `corpus/pdf/`: os 32 PDFs válidos geram PNG com as dimensões pedidas; zero páginas, senha de abertura, truncados e vazio dão código 4 sem arquivo de saída; `--page`, `--detect` e `--formats` funcionam
- [x] `cargo test --workspace --locked` passa
- [x] `cargo tree -p thumbcraft -d` não lista `craft-pdf`, `vello_cpu`, `vello_common` nem `glifo` duplicados; as duplicações restantes estão registradas em `docs/UPSTREAM-CHANGES.md`
- [x] `docs/UPSTREAM-CHANGES.md` registra cada patch dos projetos importados (WordCraft, GridCraft, DeckCraft e craft-libs), e `docs/UPDATING.md` e `CLAUDE.md` apontam para ele
- [x] Medição no Windows com os PDFs hostis (`benchmarks/pdf/hostile`): pico de memória e tempo registrados abaixo, com a meta decidida
- [x] Tamanho do binário de release (perfil de tamanho) registrado, com o custo contra os 23.995.392 bytes atuais
- [x] `docs/ARCHITECTURE.md`, `README.md`, `docs/PDF.md`, `docs/PDF-WINDOWS.md` e `docs/CRAFT-FAMILY.md` atualizados
- [x] **UTF-8 sem BOM, LF e sem mojibake** em todos os arquivos criados/modificados (`bash scripts/check-encoding.sh`)

## Resultados (2026-10-19, Windows 11, rustc 1.99.1, perfil de release do projeto)

**Uma versão por pacote.** `third_party/craft-libs` (revisão `ae2f796b`) com o patch
`patches/craft-libs/0001-hayro-vello-cpu-0.2.patch` (hayro no `vello_cpu` 0.2.0, 3 arquivos, 10 linhas inseridas e
14 removidas) e os remendos do `Cargo.toml` raiz. `cargo tree -p thumbcraft -d` não lista mais `craft-pdf`,
`vello_cpu`, `vello_common` nem `glifo`. Sobram `skrifa` (3 versões), `read-fonts` (4), `font-types` (2),
`miniz_oxide` (2) e `syn` (2, só em tempo de compilação), que já existiam antes do PDF; causas e saídas em
`docs/UPSTREAM-CHANGES.md`. Tudo que foi modificado nos projetos importados está registrado lá.

**Tamanho.** 26.701.824 bytes (25,46 MiB) contra 23.995.392 antes do PDF: **+2.706.432 bytes (+2,71 MB, +11,3%)**,
abaixo dos 4 a 6,6 MB estimados porque o rasterizador não entra duas vezes. Por crate (`cargo bloat --crates`,
`.text` de 16,4 MiB): hayro_interpret 208 KiB, hayro_jbig2 174, hayro_syntax 172, hayro_jpeg2000 82, pic_scale 79,
hayro 44 (cerca de 0,76 MB de código só do PDF, fora o que se compartilha com os motores).

**Proteções (PDFs hostis de `benchmarks/pdf/hostile`, `-W 600`, 3 repetições, processo novo a cada uma):**

| Caso | Código | Tempo | Pico de memória | Observação |
|---|---|---|---|---|
| Bomba de fluxo (512 MB descomprimidos) | 0, com aviso | 0,83 s (4,5 s na primeira, com o arquivo fora do cache) | 15 MB | Hayro direto: 1.061 MB |
| Bomba de imagem (cinza 40000 x 40000) | 0 | 0,12 s | 16 MB | Hayro direto: 2.055 MB; a imagem é descartada e sai página branca com aviso |
| Página de 14400 x 14400 pt | 0 | 2,6 s | 269 MB | 64 megapixels (256 MB) é o teto do `craft-pdf`; era 505 MB antes de tirar a cópia do buffer |
| `10_word_simples_acentos` (referência) | 0 | 0,16 s | 18 MB | |
| `38_ppt_60_slides` (referência) | 0 | 0,08 s | 12 MB | |

**Meta decidida** (a partir da primeira medição): bombas de fluxo e de imagem com no máximo 20 MB de pico e 5 s;
página gigante com no máximo 300 MB (256 MB do raster no teto de 64 megapixels mais a base do processo) e 5 s.
Todos cumpridos. No Linux o `pdf-min` ficou com 10 a 12 MB nas bombas (`docs/PDF.md`), coerente com estes 15 e 16 MB.
A cópia de `Raster::bytes` no adaptador dobrava o pico da página gigante; o adaptador agora renderiza no próprio
buffer (`raster::render_into`).

**Estudo do `krilla`.** Só o `wordcraft-pdf` (exportação de PDF do WordCraft) o usa, e o thumbcraft nunca exporta PDF.
No binário final ele pesa cerca de 270 KiB de código (`krilla` 157, `wordcraft_pdf` 64, `subsetter` 46), isto é, 1,6%
do `.text`: trocá-lo por um substituto vazio economizaria menos de 0,3 MB. O ganho real seria tirar o `skrifa`
0.42.1 (e `read-fonts` e `font-types` antigos) da árvore, mas exige um patch no `craft-pdf` e no `wordcraft-pdf`
(no núcleo do WordCraft), contra a regra de mexer o mínimo. Recomendação: não fazer agora; se o dono quiser
eliminar as duplicações de `skrifa`, abrir tarefa própria (opção A: stub do `krilla`; opção B: alinhar `krilla` e
`glifo` ao `skrifa` 0.47 por patch).

## Não escopo

- PDF com senha fornecida pelo usuário. Edição, texto ou metadados de PDF. Mais de uma página por chamada.
- Subir a revisão do `craft-libs` além da `ae2f796` e reescrever o núcleo do Hayro (só o porte mínimo para o `vello_cpu` 0.2.0).
- Substituir o `krilla` por um substituto vazio (estudado em separado; ver o relatório desta tarefa).
