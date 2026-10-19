# 05 — Builds e testes em Windows e Linux

## Objetivo

Comprovar e registrar que o thumbcraft compila e passa nos testes nas duas plataformas suportadas (Windows x64 e
Linux x64 glibc), com tamanho do binário e resultado dos corpora.

## Contexto

Linux x64 foi exercitado só na avaliação de 2026-10-19 (compilação e corpora) e o Windows x64 em 2026-10-19
(`cargo test --workspace --locked` e build de release de 23.995.392 bytes). Falta uma tabela comparável das duas
e a conferência do Linux com o código atual (stdin/stdout, JPEG e WebP, testes novos). Escopo decidido em
2026-10-19: macOS, ARM e musl ficam de fora.

## Referências

- `docs/BUILD.md`, `docs/ENVIRONMENT.md`, `docs/PITFALLS.md`, `docs/DISTRIBUTION_NPM.md`

## Regras de negócio

- Cada plataforma é validada com `cargo test --workspace --locked` e `scripts/run-corpus.sh` sobre os corpora, sem panic.
- Resultados novos vão em pasta datada de `baselines/`; nunca sobrescrever os de 2026-10-19.
- No Windows, a conferência do hash da árvore do GridCraft em `scripts/fetch-upstream.sh` divergiu (obtido
  `bd767219…`, esperado `0b8e31e3…`): investigar a causa (modo de arquivo, fim de linha) e registrar em `docs/PITFALLS.md`.

## Critérios de aceitação

- [x] Tabela em `docs/PLATFORMS.md` com plataforma, versão do Rust, resultado dos testes, tamanho do binário de release e data
- [x] Windows x64 e Linux x64 glibc: testes passam (os e2e usam o `corpus/`); `run-corpus.sh` contra os baselines rodado nas duas (`baselines/thumbcraft-run-2026-10-19-pdf-unificado/`)
- [x] Divergência do hash no Windows explicada e tratada (bit de execução; `docs/PITFALLS.md`)
- [x] **UTF-8 sem BOM, LF e sem mojibake** em todos os arquivos criados/modificados (`bash scripts/check-encoding.sh`)

## Não escopo

- macOS, ARM e Alpine (musl).
