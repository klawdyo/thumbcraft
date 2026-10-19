# thumbcraft

> CLI em Rust que gera miniaturas (PNG, JPEG ou WebP) de documentos Word, planilhas e apresentações, unindo os
> motores WordCraft, GridCraft e DeckCraft (organização storytold) em um binário só. Usado por um fluxo de upload de arquivos.

## Specs de arquitetura

Cópia local em `docs/rules/` (índice em `docs/rules/README.md`); repositório central: `F:\DEV\.skills`.
Antes de implementar, leia `docs/rules/README.md` e os arquivos que ele lista. Este arquivo só registra o que é
específico do projeto. Precedência: este arquivo, depois `docs/rules/`.

## Stack e versões

| Item | Valor |
|------|-------|
| Linguagem | Rust, edition 2024, `rust-version` 1.95 (testado com 1.99.1 no Windows e 1.97.0 no Linux) |
| Workspace | `crates/core` (núcleo sem motores, testável com `Fake`) e `crates/thumbcraft` (binário) |
| Motores | `third_party/{wordcraft,gridcraft,deckcraft,craft-libs}` em SHAs fixos (`UPSTREAM.md`), gerados por `scripts/fetch-upstream.sh`; PDF pelo Hayro (`craft-pdf`, do craft-libs) |
| Saída de build | `outputs/` (`.cargo/config.toml`), ignorada pelo git |

## Domínio

Detalhes em `docs/ARCHITECTURE.md` (módulos, pipeline, códigos de saída), `docs/BUILD.md`, `docs/PITFALLS.md` e
`docs/STATUS.md`. Resumo: o `core` define a porta `Render` e o pipeline (render, crop, trim, ajuste, redução); cada
motor é um adaptador no binário; o roteamento é por conteúdo e extensão.

## Exceções às specs compartilhadas

1. **Versão em `Cargo.toml`.** O `versioner` (0.6.0 ou mais novo) atualiza `[workspace.package] version`, mas
   não toca o `Cargo.lock`: depois de um release, rode `cargo update --workspace` e commite à parte (o build usa
   `--locked`). O `versioner` só roda sob pedido explícito do dono; a base do cálculo vem do histórico git
   (`release: X.Y.Z`) por decisão do dono, não do manifesto: o primeiro release leva o `Cargo.toml` de 0.1.0 para
   o que o histórico calcular.
2. **Regras de API, auth e erros HTTP** não se aplicam (CLI). Códigos de saída: ver `docs/ARCHITECTURE.md`.
3. **Idioma da documentação.** A documentação em `docs/` e os relatórios são em português, mas **todo `README.md` é em
   inglês**, em qualquer pasta (decisão do dono, 2026-10-19), inclusive os dos pacotes npm.

## Zonas protegidas

- `third_party/`: gerado, nunca editado (mudança vai por `patches/` e `scripts/fetch-upstream.sh`).
- `patches/`: só pelo roteiro de `docs/UPDATING.md`. Toda mudança nos projetos importados é um patch pequeno aqui, sem reescrever o núcleo deles, e fica registrada em `docs/UPSTREAM-CHANGES.md`.
- **Uma versão por pacote:** o objetivo do projeto é um binário pequeno, padronizado e eficiente; não aceite duas versões do mesmo crate no binário (`cargo tree -p thumbcraft -d`).
- `corpus/` e `baselines/`: dados históricos de 2026-10-19, nunca sobrescritos; resultados novos vão em pasta datada.
- Benchmarks, binários, `node_modules` e `target/` não entram no git (`docs/STATUS.md`); ficam em `benchmarks/` e
  `outputs/`, ignoradas.

## Codificação e fim de linha

LF sempre e UTF-8 sem BOM, mesmo no Windows. `git config core.autocrlf` deve ser `false`. Antes de qualquer commit:
`bash scripts/check-encoding.sh` (cole o resultado ao encerrar a tarefa) e `git diff --cached --check`.

## Comandos úteis

```bash
cargo test --locked                 # núcleo (rápido, sem compilar os motores)
cargo test --workspace --locked     # inclui encode e e2e da CLI (compila os motores)
scripts/fetch-upstream.sh           # baixa os motores e aplica o patch do GridCraft
cargo build --release --locked -p thumbcraft
bash scripts/check-encoding.sh
```

No Windows, o Acesso Controlado a Pastas do Defender pode bloquear scripts de build dentro do projeto até cada
executável ser aprovado.

## Tarefas

Backlog em `.tasks/00-manifest.md` (formato em `docs/rules/task-template.md`).
