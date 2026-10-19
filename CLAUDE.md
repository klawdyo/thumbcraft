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
| Motores | `third_party/{wordcraft,gridcraft,deckcraft}` em SHAs fixos (`UPSTREAM.md`), gerados por `scripts/fetch-upstream.sh` |
| Saída de build | `outputs/` (`.cargo/config.toml`), ignorada pelo git |

## Domínio

Detalhes em `docs/ARCHITECTURE.md` (módulos, pipeline, códigos de saída), `docs/BUILD.md`, `docs/PITFALLS.md` e
`docs/STATUS.md`. Resumo: o `core` define a porta `Render` e o pipeline (render, crop, trim, ajuste, redução); cada
motor é um adaptador no binário; o roteamento é por conteúdo e extensão.

## Exceções às specs compartilhadas

1. **`versioner` ainda não cobre Rust.** Ele só atualiza `package.json`, `pubspec.yaml`, `pyproject.toml` e `go.mod`; a
   versão deste projeto está em `Cargo.toml` (`[workspace.package]`). O suporte está sendo feito no repositório do
   versioner; até ficar pronto, não usar o `versioner` aqui e não editar versão à mão sem pedido do dono.
2. **Regras de API, auth e erros HTTP** não se aplicam (CLI). Códigos de saída: ver `docs/ARCHITECTURE.md`.
3. **Documentação em `docs/`** é em português; o `README.md` é em inglês por pedido do dono.

## Zonas protegidas

- `third_party/`: gerado, nunca editado (mudança vai por `patches/` e `scripts/fetch-upstream.sh`).
- `patches/gridcraft/`: só pelo roteiro de `docs/UPDATING.md`.
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
