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
- O roteamento já reconhece `%PDF` (`Sniff::Pdf`) e tem `Engine::Pdf`; hoje `open.rs` responde "PDF ainda não é
  suportado" (código 3).

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

- [ ] Testes do núcleo (roteamento de PDF por conteúdo, por extensão, com lixo antes do cabeçalho e texto com extensão `.pdf`) escritos antes, falhando pelo motivo certo, depois passando
- [ ] Testes unitários do adaptador, com PDFs mínimos em código, sem disco: tamanho da página, número de páginas, página inexistente, PDF sem páginas e dados inválidos
- [ ] Testes e2e sobre `corpus/pdf/`: os 32 PDFs válidos geram PNG com as dimensões pedidas; zero páginas, senha de abertura, truncados e vazio dão código 4 sem arquivo de saída; `--page`, `--detect` e `--formats` funcionam
- [ ] `cargo test --workspace --locked` passa
- [ ] `cargo tree -p thumbcraft -d` não lista `craft-pdf`, `vello_cpu`, `vello_common` nem `glifo` duplicados; as duplicações restantes estão registradas em `docs/UPSTREAM-CHANGES.md`
- [ ] `docs/UPSTREAM-CHANGES.md` registra cada patch dos projetos importados (WordCraft, GridCraft, DeckCraft e craft-libs), e `docs/UPDATING.md` e `CLAUDE.md` apontam para ele
- [ ] Medição no Windows com os PDFs hostis (`benchmarks/pdf/hostile`): pico de memória e tempo registrados abaixo, com a meta decidida
- [ ] Tamanho do binário de release (perfil de tamanho) registrado, com o custo contra os 23.995.392 bytes atuais
- [ ] `docs/ARCHITECTURE.md`, `README.md`, `docs/PDF.md`, `docs/PDF-WINDOWS.md` e `docs/CRAFT-FAMILY.md` atualizados
- [ ] **UTF-8 sem BOM, LF e sem mojibake** em todos os arquivos criados/modificados (`bash scripts/check-encoding.sh`)

## Não escopo

- PDF com senha fornecida pelo usuário. Edição, texto ou metadados de PDF. Mais de uma página por chamada.
- Subir a revisão do `craft-libs` além da `ae2f796` e reescrever o núcleo do Hayro (só o porte mínimo para o `vello_cpu` 0.2.0).
- Substituir o `krilla` por um substituto vazio (estudado em separado; ver o relatório desta tarefa).
