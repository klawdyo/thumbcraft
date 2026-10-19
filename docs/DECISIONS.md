# Decisões

Cada decisão tem contexto, alternativas e razão. Marca de origem: **[usuário]** foi decidida ou exigida por Cláudio; **[agente]** foi recomendação ou escolha de um subagente, aceita ou não contestada. Ordem cronológica de 2026-10-19. Datas e fases em `HISTORY.md`.

## D01 [usuário] Usar um binário mínimo próprio em vez do CLI completo do WordCraft

- Contexto: o `wordcraft-cli` carrega MCP, Zotero e o conjunto completo de comandos. Não há `[features]` para desligá-los.
- Alternativas: usar o CLI completo (42,60 MB no perfil do repositório, 19,99 MB com perfil de tamanho); cortar `mcp` e `zotero` (economia de 0,22 MB); escrever um binário de 15 linhas (`wc-min`).
- Razão: o binário mínimo gera PNG byte a byte idêntico ao do CLI completo e fica em 19,76 MB. O peso vem do motor, não dos extras.

## D02 [usuário] Medir sempre contra o LibreOffice, um processo por arquivo

- Contexto: o objetivo é substituir o LibreOffice como gerador de thumbnail.
- Alternativas: comparar só entre os *craft; comparar com `pdftoppm` ou outras ferramentas.
- Razão: o LibreOffice é o que se usaria hoje (lê quase tudo). A comparação de tempo, taxa de sucesso e qualidade visual responde se compensa trocar. Cada arquivo roda em um processo novo, porque é assim que um fluxo de upload funciona.

## D03 [agente, aceita] Parar o teste do GridCraft quando a CLI não gerou PNG

- Contexto: o `gridcraft-cli` exporta PDF mas não PNG.
- Alternativas: rasterizar o PDF com `pdftoppm` (não é binário único); compilar o exemplo `snapshot` do `ui-egui` (wgpu, GPU ou lavapipe, binário muito maior).
- Razão: as duas fogem do objetivo de um binário enxuto. Isso abriu a frente de fazer o GridCraft gerar PNG.

## D04 [usuário] Avaliar um binário unificado dos três

- Contexto: pergunta sobre código compartilhado.
- Resultado medido: 26,60 MB contra a soma de 32,87 MB (menos 19%). A compressão (xz) rende tanto quanto a unificação.
- Razão para seguir: um único executável no fluxo de upload, com a economia vindo de crates externos. Os *craft não compartilham código próprio.

## D05 [agente, parcialmente aceita] Caminho C como menor diff para PNG no GridCraft

- Alternativas: A (trait `Canvas` com `vello_cpu`, 600 a 900 linhas), B (cena nova, 1200 a 1800), C (PDF e depois `craft-pdf` raster, 80 a 150), E (cena simplificada com texto em barras).
- O agente recomendou C. Os esforços em "dias" foram contestados pelo usuário como exagerados. A escolha final mudou para uma variante do A (D09).

## D06 [usuário] Trabalhar em fases: primeiro refatorar sem regressão, depois exportar PNG

- Contexto: o `print.rs` desenha em modo imediato. Para ter PNG com o mesmo desenho, é preciso gravar as operações.
- Fase 1: só `print.rs`, provar zero regressão contra o código original (PDF byte a byte, streams e pixels). Fase 2, com liberação para criar arquivos: extrair o reutilizável e plugar o exportador de PNG.
- Razão: a fase 1 é a prova de segurança que um mantenedor aceitaria. Se o PDF não mudou, o risco do PR é baixo.

## D07 [usuário] Manter separadas três frentes

1. Refatoração que possa virar PR no GridCraft, no padrão dos *craft e não sob medida para o binário unificado.
2. Viabilidade de thumbnails (padrões do thumbnail ficam em `gc-min`, não no comando do PR).
3. Unificação, como consequência.

Razão: evitar que decisões do binário unificado contaminem o PR e vice-versa.

## D08 [usuário] A4 em vez de A3, sem zoom, primeira página como o LibreOffice

- Contexto: o `gp-min` começou em A3 com ajuste e recorte ao conteúdo.
- Decisão: a thumbnail é a primeira página A4, sem zoom, mostrando só o que couber (como o LibreOffice), porque será usada em 120x120.
- Razão: comportamento previsível e igual à linha de base. Cada planilha segue o próprio Page Setup (orientação, margens, escala em %, área de impressão), só o papel é forçado para A4.
- Consequência conhecida: sem zoom, o conteúdo útil pode ocupar só o canto da página; `--crop` (D11) ajuda.

## D09 [usuário] `vello_cpu` 0.2.0 como backend de raster do GridCraft

- Alternativas: `craft-pdf` raster (hayro, exige subir a revisão do craft-libs e repetir patches); `tiny-skia` (esboço descartado); `rasterize` do crate chart (sem texto, sem clip, sem imagem).
- Razão: é o mesmo rasterizador do WordCraft e do DeckCraft, com as mesmas versões (`vello_cpu` 0.2.0, `kurbo` 0.13.1, `peniko` 0.6.1, `skrifa` 0.47.0), então o binário unificado fica enxuto por deduplicação. No GridCraft isolado, a GUI continua em `vello_cpu` 0.1.0 (via epaint).

## D10 [agente] `Op`, `Rec` e `replay` em `crates/pdf/src/display.rs`

- Contexto: o backend PNG, em outro crate, não consegue consumir tipos privados do `print.rs`.
- Razão: o crate `pdf` é camada 0, sem dependências, e já é dono de `Font`, `Rgb` e `Page`. O `Page` não foi alterado.

## D11 [usuário] Crop é trivial e entra no `gc-min`

- Contexto: o primeiro `gc-min` não tinha `--crop` (omissão do agente, apresentada como "não implementado").
- Decisão: implementar no `gc-min`, não no comando do PR: decodifica e recodifica o PNG com o crate `png` (já no lock), 0,019 MB a mais.

## D12 [agente, pendente dos mantenedores] Fontes embutidas no crate raster

- Contexto: o `AGENTS.md` do GridCraft manda manter fontes em `storytold/craft-fonts`. O PNG precisa de fontes que casem com as métricas AFM do PDF.
- Escolha: 12 TTF Arimo, Tinos e Cousine (subconjunto Latin/WinAnsi, 239 KB, SIL OFL 1.1) embutidos. Alternativa para o PR: mover para o `craft-fonts` e carregar por `CRAFT_FONTS_DIR`.
- Origem das fontes: pacote npm `@expo-google-fonts/{arimo,tinos,cousine}`, porque o GitHub do `craft-fonts` deu 403 na máquina de testes.

## D13 [usuário] O thumbcraft é uma ponte

- O binário unificado existe enquanto os *craft evoluem (o GridCraft ainda não gera PNG, por isso há patch). Por isso o projeto guarda corpus, baselines e um playbook (`UPDATING.md`) para reavaliar versões novas.
- Renderização interna em alta resolução e depois reduzida, porque o binário tem trim, crop, resize e ajustes `contain` e `cover`.

## D14 [agente, recomendação] Um processo por arquivo, com timeout

- Contexto: os builds de tamanho usam `panic=abort`, que derruba o processo em falha interna (perde o `catch_unwind` do renderizador).
- Razão: num fluxo de upload, um processo por arquivo com timeout isola falhas.

## D15 [agente, recomendação, a confirmar na implementação] Tratar perda do slide 0 como erro no DeckCraft

- Contexto: se o XML do slide 1 está quebrado, o DeckCraft descarta o slide e renderiza o 2, gerando thumbnail do slide errado. O aviso só sai em `lost` e no stderr.
- Recomendação: imprimir a lista `lost` de `deckcraft_pptx::import_checked` e falhar quando o slide 0 estiver nela.

## D16 [usuário] Convenções do repositório

- Português do Brasil, Markdown, sem travessões, sem Python (scripts Python da sessão não são versionados; só os arquivos gerados e a descrição das ferramentas).
- Tamanhos em MB decimais com MiB entre parênteses. Relatórios antigos que chamavam MiB de MB têm nota de correção.
- Repositório pequeno: nenhum arquivo acima de 10 MB, sem Git LFS. Goldens de PDF, binários e PNGs completos viram hashes e receitas. Imagens de comparação reduzidas a 1600 px de largura.
- Código puro: sem benchmarks, binários, `target/` ou arquivos grandes (decisão posterior de Cláudio; o PDF só tem método e resultados em `PDF.md`).

## D17 [usuário] PDF avaliado em paralelo

- O `pdf-parse` do Node (getScreenshot) contra os *craft, e se o `craft-pdf` já está nos binários atuais. Cuidado por outro agente. PDF não é suportado pelos três motores por enquanto.

## D18 [usuário] CI/CD de release é etapa futura

- Gerar o binário de release para Windows e Ubuntu 24 com GitHub Actions, só depois que o binário unificado funcionar. Detalhes em `UPDATING.md`.
