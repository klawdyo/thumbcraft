# 02 — Wrapper TypeScript: API, opções e execução do binário

## Objetivo

Criar o pacote `packages/thumbcraft` (TypeScript) com a API `Thumbcraft.fromFile`, `fromBuffer`, `toBuffer` e
`toFile`, que executa o binário `thumbcraft` com um processo por arquivo, timeout e tratamento dos códigos de saída.

## Contexto

Plano em `docs/DISTRIBUTION_NPM.md` (seções 1 e 4), já corrigido em 2026-10-19: `trimPad` em pixels (no lugar de
`trimTolerance`), `gravity` padrão `top` no cover e `center` no contain, `scale` padrão `'auto'`. Nada do pacote
existe ainda. `fromBuffer` sem arquivo temporário depende da tarefa 01; até lá `fromFile` funciona sozinho.

## Referências

- `docs/DISTRIBUTION_NPM.md`, `docs/ARCHITECTURE.md` (códigos de saída, `panic = "abort"`), `crates/thumbcraft/src/cli.rs` (opções reais)
- `docs/rules/principles.md` (DI), `testing.md`, `naming-and-language.md` (arquivos em kebab-case, interfaces com `I`, testes `.spec.ts`)
- Guias TypeScript do repositório central (`F:\DEV\.skills\nestjs\` e `vue\`) como referência de estrutura, sem herdar o que for de NestJS ou Vue

## Regras de negócio

- Um processo `thumbcraft` por chamada, nunca reaproveitado. Timeout padrão de 30 s, configurável.
- Código de saída 2, 3, 4 e 5 vira erro tipado (`UsageError`, `UnsupportedFormatError`, `InvalidFileError`,
  `InternalError`); qualquer outro status (inclui 134) vira `ProcessCrashError`, e o kill por timeout vira `TimeoutError`.
- A mensagem de erro do binário (stderr, em português) é preservada no erro.
- Caminho do binário resolvido por uma abstração injetável (a tarefa 04 fornece a implementação real); aceitar também a variável de ambiente `THUMBCRAFT_BIN` para desenvolvimento.

## Detalhes técnicos

- Estrutura por feature em `packages/thumbcraft/src/`, execução de processo atrás de uma interface injetável (`IProcessRunner`) para testar sem processo real.
- Tipos de `RenderOptions` espelham as opções da CLI (tabela da seção 4.4 do plano).
- Node 20 ou mais novo; ESM com tipos; sem dependências de runtime além de módulos `node:`.

## Critérios de aceitação

- [x] `npm test` em `packages/thumbcraft` passa, com testes escritos antes, sem processo real nos unitários
- [x] Teste de integração (roda com `THUMBCRAFT_BIN`) gera PNG de `corpus/docx/01_docx_simples_acentos.docx` com 120 x 120
- [x] Cada código de saída 2, 3, 4 e 5 vira o erro tipado correspondente; 134 e timeout viram os erros de processo
- [x] `npm run build` gera `.d.ts` e o pacote não declara dependências de runtime
- [x] **UTF-8 sem BOM, LF e sem mojibake** em todos os arquivos criados/modificados (`bash scripts/check-encoding.sh`)

## Não escopo

- `isValid` e `detect` (tarefa 03). Pacotes de plataforma e resolução do binário instalado (tarefa 04). CI (tarefa 06).
