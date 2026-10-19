# Manifesto de tarefas

Formato em `docs/rules/task-template.md` (fluxo em `docs/rules/workflow-and-agents.md`).

| Seq | File | Objetivo (curto) | Status | Depends on |
|-----|------|------------------|--------|------------|
| 01 | 01-cli-stdin-stdout.md | `thumbcraft - -`: entrada por stdin e saída por stdout na CLI | done | — |
| 02 | 02-ts-wrapper-core.md | Pacote `packages/thumbcraft`: API, opções, execução do binário, erros | done | 01 (só `fromBuffer`) |
| 03 | 03-ts-detect-is-valid.md | `detect` e `isValid` por `thumbcraft --detect`, sem ZIP em TypeScript | done | 02, 01 (buffer) |
| 04 | 04-npm-platform-packages.md | Pacotes `@thumbcraft/*` (Windows x64, Linux x64) e resolução do binário | done | 02 |
| 05 | 05-platform-builds.md | Build e testes em Windows e Linux, tabela por plataforma | todo | — |
| 06 | 06-ci-release-npm.md | CI de testes e release no npm | todo | 04, 05 |
| 07 | 07-english-identifiers.md | Identificadores e nomes de testes em inglês no código existente | done | — |
| 08 | 08-pdf-engine.md | Motor de PDF (hayro via craft-pdf) no binário unificado, com medição de proteções e tamanho | done | — |
| 09 | 09-single-package-versions.md | Eliminar as duplicações de skrifa, read-fonts, font-types e miniz_oxide no binário | done | 08 |

Status: `todo`, `doing`, `done`, `blocked`.
