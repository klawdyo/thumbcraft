# 07 — Identificadores e nomes de testes em inglês

## Objetivo

Passar para inglês todos os identificadores do código Rust do projeto que hoje estão em português (nomes de testes
e outros), conforme `docs/rules/naming-and-language.md`, sem mudar comportamento.

## Contexto

O código existente foi escrito com nomes de testes em português; as regras do dono pedem identificadores em inglês e
comentários e mensagens em português. Decisão de 2026-10-19: corrigir também o código existente (a exceção que o
`CLAUDE.md` registrava foi removida). Comentários, doc comments, mensagens de erro e textos de ajuda continuam em português.

## Referências

- `docs/rules/naming-and-language.md`, `docs/rules/testing.md`
- `crates/core/src/*.rs`, `crates/core/tests/pipeline.rs`, `crates/thumbcraft/src/*.rs`, `crates/thumbcraft/tests/cli.rs`

## Regras de negócio

- Refatoração pura: mesmos testes, mesmos resultados; nenhum teste removido nem enfraquecido.
- Só identificadores (funções de teste, variáveis, constantes, tipos, campos); texto visível ao usuário e comentários não mudam.
- `third_party/` não é tocado.

## Critérios de aceitação

- [x] Nenhum identificador com palavra portuguesa nos crates (revisão por busca)
- [x] `cargo test --workspace --locked` passa com a mesma contagem de testes (80 do núcleo, 10 de pipeline, 10 da CLI e encode, 17 e2e)
- [x] **UTF-8 sem BOM, LF e sem mojibake** em todos os arquivos criados/modificados (`bash scripts/check-encoding.sh`)

## Não escopo

- Traduzir comentários, mensagens de erro ou documentação.
