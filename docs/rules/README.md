# The owner's rules applied to this project

A copy, made on 2026-10-19, of the rules from `F:\DEV\.skills` (the owner's central repository of specs) that apply to
thumbcraft, a Rust project with no HTTP API, no database and no graphical interface. The files are faithful copies
(in Portuguese); deliberate deviations are recorded in `CLAUDE.md` (section "Exceções às specs compartilhadas") and are
never edited here. Order of precedence: the project's `CLAUDE.md`, then these files.

| File | Origin | Summary |
|---|---|---|
| [principles.md](principles.md) | `shared/principles.md` | Modules by feature, dependency injection, rules outside the presentation layer, layers |
| [naming-and-language.md](naming-and-language.md) | `shared/naming-and-language.md` | Identifiers in English, comments and messages in Portuguese, comment everything |
| [encoding-and-files.md](encoding-and-files.md) | `shared/encoding-and-files.md` | UTF-8 without BOM, LF, verification with `scripts/check-encoding.sh` |
| [commits-and-versioning.md](commits-and-versioning.md) | `shared/commits-and-versioning.md` | Conventional Commits, atomic commits, no breaking change without authorization, `versioner` |
| [testing.md](testing.md) | `shared/testing.md` | TDD, no real I/O in unit tests |
| [workflow-and-agents.md](workflow-and-agents.md) | `shared/workflow-and-agents.md` | Roles, tasks in `.tasks/`, protected zones |
| [task-template.md](task-template.md) | `templates/task.md` | Task template |
| [tasks-manifest-template.md](tasks-manifest-template.md) | `templates/tasks-manifest.md` | Task manifest template |

Also taken from `templates/`: `.editorconfig` and `.gitattributes` (repository root, with `indent_size = 4` for `.rs` and `-text` on patches) and `scripts/check-encoding.sh` (with a single adaptation: its ignore list includes `docs/rules/encoding-and-files.md`, which quotes examples of mojibake).

## What was not copied, and why

| Source rule | Reason |
|---|---|
| `shared/api-contract.md`, `auth-and-permissions.md`, `errors-and-exceptions.md` | They cover HTTP APIs, JWT and the error envelope; thumbcraft is a CLI with its own exit codes (see `docs/ARCHITECTURE.md`) |
| `shared/project-layout.md` | Multi-repository; only what applies was adopted (`CLAUDE.md`, `AGENTS.md`, `.tasks/`, `.editorconfig`, `.gitattributes`) |
| `flutter/`, `nestjs/`, `dart-api/`, `vue/` | Guides for stacks the project does not use. The TypeScript ones (`nestjs/`, `vue/`) may apply to the npm libraries; see `.tasks/` when that work is done |

Updating: if the source changes, copy the file again and record the date here; do not edit the copies.
