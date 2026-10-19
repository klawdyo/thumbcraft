# WordCraft: conversão para PNG pela CLI e binário mínimo

> Arquivo de relatório. Origem: subagente `a9c8d6827203c05f5`, turno 1 (de 2). Sessão de 2026-10-19.
> Tema: Pergunta 1 do usuário: o WordCraft converte doc em PNG pela CLI só com o core? Pergunta 2: qual o tamanho do binário mínimo?
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

## Relatório: WordCraft, conversão doc/docx para PNG via CLI

**Veredito da premissa: confirmada, com ressalvas de nomenclatura.**
- Não existe um crate "core" isolado. O que existe é o crate de aplicação `wordcraft-cli` (`apps/wordcraft-cli`). Ele não tem GUI nem servidor e não depende de `ui-egui`, `eframe` nem `egui`.
- O README documenta `wordcraft-cli convert report.docx report.pdf`, e a lista de formatos inclui `png`. No código (`apps/wordcraft-cli/src/main.rs`):
  - `convert in out.png` chama `file.exportPng` e gera só a página 1.
  - `render in out.png [--page N] [--scale S]` escolhe a página e a escala (padrão 2.0).
- `.docx` é lido e `.doc` (Word 97-2003) é importado, só leitura.
- O rasterizador é puro Rust (`vello_cpu`, no crate `wordcraft-render`), sem GPU.
- Parcial em um ponto: o `wordcraft-cli` carrega também `wordcraft-mcp` e `wordcraft-zotero`, porque o MCP e o Zotero ficam no mesmo binário.
- Não há `[features]` em nenhum Cargo.toml do workspace, então não há flags para desligar. O corte só é possível escrevendo outro binário que dependa de `wordcraft-engine` + `wordcraft-doc`, sem `mcp`, `zotero` e `sys-locale`.
- Testei isso com um binário próprio de 15 linhas (`apps/wc-min`, dentro do clone). O repositório não traz esse binário.

**Comando de build (`wordcraft-cli`, sem alteração de código):**
```
cargo build --release -p wordcraft-cli
```
Com otimizações de tamanho (variáveis de ambiente):
```
CARGO_PROFILE_RELEASE_OPT_LEVEL=z CARGO_PROFILE_RELEASE_LTO=fat \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_PROFILE_RELEASE_PANIC=abort \
CARGO_PROFILE_RELEASE_STRIP=symbols \
cargo build --release -p wordcraft-cli -p wc-min
```

**Tamanhos medidos (x86_64 Linux, rustc 1.97, tudo compilado de fato):**

| Variante | Bytes | MB |
|---|---|---|
| `wordcraft-cli`, release padrão do repo (thin LTO, cgu=16, sem strip) | 42.603.432 | 40,6 |
| mesmo, só com `strip` | 35.087.928 | 33,5 |
| `wordcraft-cli`, opt-level z + fat LTO + cgu=1 + panic=abort + strip | 19.985.520 | 19,1 |
| **`wc-min` (engine + doc, sem mcp/zotero), mesmas otimizações** | **19.763.984** | **18,8** |

[Nota de correção do arquivista: a coluna "MB" desta tabela está em MiB (1 MiB = 1.048.576 bytes), não em MB decimais. Em MB decimais (1 MB = 1.000.000 bytes): 42.603.432 bytes = 42,60 MB (40,63 MiB); 35.087.928 = 35,09 MB (33,46 MiB); 19.985.520 = 19,99 MB (19,06 MiB); 19.763.984 = 19,76 MB (18,85 MiB). O mini Word (`wc-min`) tem 19,76 MB decimais.]

- Resposta à pergunta 2: **cerca de 19 MB (18,8 MB)** para o binário somente com o necessário.

[Nota de correção do arquivista: onde se lê "cerca de 19 MB (18,8 MB)", o valor correto é 19,76 MB decimais, que equivale a 18,85 MiB. O "18,8 MB" do texto é MiB.]
- Cortar `mcp` e `zotero` economiza só cerca de 0,2 MB.
- Não medi o ganho de strip isolado nas otimizações de tamanho (a faixa de ~19 MB junta tudo).
- O tamanho vem sobretudo do que `wordcraft-engine` já puxa:
  - o conjunto completo de comandos;
  - leitura e escrita de docx, odt, rtf e outros;
  - PDF (krilla);
  - fontes embutidas (`assets/fonts`, 4,2 MB);

[Nota de correção do arquivista: a medição do relatório 05 dá 4.305.160 bytes para as 13 fontes embutidas, ou seja, 4,31 MB decimais (4,11 MiB). O "4,2 MB" é aproximado.]
  - dicionários e hifenização (`wordcraft-proof`).
- Para ficar menor seria preciso alterar os crates internos, o que não tentei.

**Teste funcional:**
- Criei um docx com pandoc e um `.doc` com o LibreOffice, ambos com título, negrito, itálico, acentos e tabela.
- Os dois binários otimizados geraram um PNG válido de 1224x1584 (Letter, escala 2).
- Conferi a imagem visualmente: texto, acentos e negrito saíram corretos, e a tabela saiu sem bordas.
- A saída de `wc-min` e a do `wordcraft-cli` para o mesmo `.doc` são byte a byte idênticas.

**Dependências de runtime:**
- Apenas `libc`, `libm` e `libgcc_s` (`ldd` lista só essas). Não há libs gráficas, OpenSSL nem fontontconfig obrigatórios.
- As fontes bundled OFL (Source Sans 3, Source Serif 4, Inter, JetBrains Mono) estão dentro do binário. Fontes do sistema são usadas se existirem, mas não são exigidas.
- A fonte Calibri do docx foi substituída pelas bundled, então a fidelidade tipográfica ao Word não é exata.

**Ressalvas:**
- Um build limpo leva cerca de 7 a 8 minutos em 2 vCPUs.
- É preciso Rust 1.95 ou superior (edition 2024).
- O build baixa o fork `craft-libs` do GitHub (dependência `git`), que o allowlist permitiu.
- Os testes foram só em Linux x86_64 e com um documento simples. Documentos com imagens, WMF ou EMF e `.doc` complexos não foram testados.
- O `convert` para PNG exporta só a página 1. Para as demais páginas, use `render --page N`.

Arquivos relevantes (tudo em `S`):
- `repo/apps/wordcraft-cli/src/main.rs`
- `repo/apps/wc-min/src/main.rs`, o binário mínimo que escrevi
- `target-small/release/wc-min`, o binário medido, de 19,8 MB
