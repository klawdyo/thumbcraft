# Regras do dono aplicadas a este projeto

Cópia, em 2026-10-19, das regras de `F:\DEV\.skills` (repositório central de specs do dono) que se aplicam ao
thumbcraft, um projeto Rust sem API HTTP, sem banco e sem interface gráfica. Os arquivos são cópias fiéis;
desvios deliberados ficam em `CLAUDE.md` (seção "Exceções"), nunca editados aqui. Ordem de precedência:
`CLAUDE.md` do projeto, depois estes arquivos.

| Arquivo | Origem | Resumo |
|---------|--------|--------|
| [principles.md](principles.md) | `shared/principles.md` | Módulos por feature, injeção de dependências, regra fora da apresentação, camadas |
| [naming-and-language.md](naming-and-language.md) | `shared/naming-and-language.md` | Identificadores em inglês, comentários e mensagens em português, comentar tudo |
| [encoding-and-files.md](encoding-and-files.md) | `shared/encoding-and-files.md` | UTF-8 sem BOM, LF, verificação com `scripts/check-encoding.sh` |
| [commits-and-versioning.md](commits-and-versioning.md) | `shared/commits-and-versioning.md` | Conventional Commits, commits atômicos, sem breaking change sem autorização, `versioner` |
| [testing.md](testing.md) | `shared/testing.md` | TDD, sem I/O real em teste unitário |
| [workflow-and-agents.md](workflow-and-agents.md) | `shared/workflow-and-agents.md` | Papéis, tarefas em `.tasks/`, zonas protegidas |
| [task-template.md](task-template.md) | `templates/task.md` | Modelo de tarefa |
| [tasks-manifest-template.md](tasks-manifest-template.md) | `templates/tasks-manifest.md` | Modelo do manifesto de tarefas |

Também vieram de `templates/`: `.editorconfig` e `.gitattributes` (raiz, com `indent_size = 4` para `.rs` e `-text` nos patches) e `scripts/check-encoding.sh` (com uma única adaptação: a lista de arquivos ignorados inclui `docs/rules/encoding-and-files.md`, que cita exemplos de mojibake).

## O que não foi copiado, e por quê

| Regra de origem | Motivo |
|-----------------|--------|
| `shared/api-contract.md`, `auth-and-permissions.md`, `errors-and-exceptions.md` | Tratam de API HTTP, JWT e envelope de erro; o thumbcraft é uma CLI com códigos de saída próprios (ver `docs/ARCHITECTURE.md`) |
| `shared/project-layout.md` | Multi-repositório; só o que vale foi aplicado (`CLAUDE.md`, `AGENTS.md`, `.tasks/`, `.editorconfig`, `.gitattributes`) |
| `flutter/`, `nestjs/`, `dart-api/`, `vue/` | Guias de stacks que o projeto não usa. Os de TypeScript (`nestjs/`, `vue/`) podem valer para as libs npm; ver `.tasks/` quando a tarefa existir |

Atualizar: se a origem mudar, recopie o arquivo e registre a data aqui; não edite as cópias.
