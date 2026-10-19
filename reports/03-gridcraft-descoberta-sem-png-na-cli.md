# GridCraft: descoberta de que não há PNG pela CLI

> Arquivo de relatório. Origem: subagente `a361871d26ddb3c94`, turno 1 (único). Sessão de 2026-10-19.
> Tema: Primeira tentativa de testar o irmão de Excel: o agente parou ao constatar que o GridCraft não gera PNG pela CLI.
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

**Resultado: não existe CLI do GridCraft que gere PNG. Parei no passo 1 e não compilei nem testei nada.** As pastas `in/`, `out-rust/`, `out-lo/`, `thumbs/` e `repo/` foram criadas e estão vazias, e o único conteúdo novo é o clone do repositório.

**Repositório irmão de Excel: GridCraft**, em https://github.com/storytold/gridcraft (versão 0.8.0, Rust 2024, exige rustc 1.90 ou mais novo). O README o descreve como uma reimplementação "clean-room" do Excel em Rust puro, da equipe ArtCraft. Ele aparece na tabela "The Crafting Apps" do próprio README. O README do wordcraft não cita um irmão de Excel nem de PowerPoint, então cheguei nele pela busca web. O `gh api` da organização é bloqueado pelo proxy (HTTP 403), e adivinhar nomes via curl também deu 403. O clone por `git clone` funcionou. Está em `S/xlsx-test/repo/gridcraft`.

**O que o `gridcraft-cli` faz** (conforme `docs/cli.md` e `apps/gridcraft-cli/src/main.rs`):
- **Subcomandos:** `info`, `convert`, `eval`, `cat`, `check`, `run`, `commands`, `functions`, `mcp` e `send`.
- **Saídas do `convert`:** `.xlsx`, `.csv`, `.tsv`, `.json` e `.html`. O `file.saveAs` do motor também aceita `.pdf`, e o PDF é um desenho próprio feito pelo crate `gridcraft-pdf`.
- **PNG:** não há em nenhum subcomando. Procurei no código do CLI, no crate do motor (`engine`) e na documentação de CLI, MCP e protocolo de controle. Um grep por "png" só encontra:
  - o comando `screenshot`/`ui.screenshot` do MCP, que fotografa a janela do app desktop já aberto (`mcp --connect`);
  - o exemplo `crates/ui-egui/examples/snapshot.rs`, que renderiza a janela inteira com wgpu fora da tela.
- **Formatos de entrada:**
  - **Leitura e escrita:** `.xlsx` (também `.xlsm` no `check`).
  - **Só leitura (importação):** `.xls` (BIFF8, sem fontes, fundos, bordas nem gráficos), `.xlsb` e `.ods` (os dois só com dados; fórmulas ficam com o último valor salvo e a formatação é omitida).
  - **Texto:** `.csv` e `.tsv`.
  - Não há seleção de escala. A aba vem de `--sheet` no `convert`.

**Por que parei:** a tarefa dizia para parar se o CLI não gerasse PNG. Os dois caminhos que existem não servem como um binário mínimo de thumbnail:
1. O `snapshot` é um exemplo do crate `ui-egui`. Ele depende de egui, wgpu e do `craft-ui`, uma dependência git de `storytold/craft-libs`. Isso significa GPU ou lavapipe, e o binário fica muito maior que o do LibreOffice, o oposto do que o usuário quer.
2. Exportar PDF com `gridcraft-cli run --in x.xlsx --cmd 'file.exportPdf={"path":"x.pdf"}'` e rasterizar depois exigiria outra ferramenta, como `pdftoppm`, ou código novo. Isso fugiria do objetivo de usar só o binário.

Pelo que li, o GridCraft tem um rasterizador de gráficos sem toolkit (`gridcraft-chart::rasterize`, "para exportação PNG e testes"), mas nada que desenhe uma aba inteira em PNG sem a UI. Eu **não** verifiquei se o caminho CLI até PDF funciona de fato.

Se você quiser, posso fazer uma destas duas coisas:
- Medir o fluxo `gridcraft-cli` para PDF e rasterizar com `pdftoppm` ou ImageMagick, deixando claro que não é um binário só.
- Tentar compilar o `snapshot` e medir tamanho e dependências de GPU.

Para memória: GridCraft é o irmão de Excel (storytold/gridcraft). Ele exporta PDF mas não PNG pelo CLI, e o PNG só existe via screenshot da janela do app ou pelo exemplo `snapshot` com wgpu.
