# Nomenclatura, idioma e comentários

> Regras padrão do dono. Um projeto só as altera se declarar a exceção no
> próprio `CLAUDE.md`/`AGENTS.md`.

## Idioma

| O que | Idioma |
|-------|--------|
| Identificadores (variáveis, funções, classes, arquivos, pastas, rotas, campos JSON) | **Inglês** |
| Comentários de código e doc comments | **Português (BR)** |
| Mensagens de erro (usuário, logs, exceptions) | **Português (BR)** |
| Texto de UI (labels, botões, placeholders) | **Português (BR)** |
| Respostas do agente no chat | **Português (BR)** |
| Mensagens de commit | **Português (BR)** (tipo e escopo em inglês, ver `commits-and-versioning.md`) |

## Densidade de comentários

**Comente tudo, inclusive o óbvio.** Toda classe pública, campo, getter,
setter e método recebe doc comment: `///` em Dart, `/** */` em TypeScript.
Isto é exigência explícita do dono e **substitui** o padrão "comente só o
não-óbvio". Comentário explica o *porquê* e o contrato; não repete o nome.

## Campos de DTO/JSON

- `camelCase`, **idêntico nos dois lados** (`stockMovementId`, `createdAt`,
  `isActive`). Nunca traduzir nem reformatar nome de campo na camada de
  dados.
- Datas: ver `api-contract.md` (UTC ISO 8601 com `Z`).

## Convenções por linguagem

| Item | Dart/Flutter | TypeScript/NestJS | Vue/TS |
|------|--------------|-------------------|--------|
| Pasta de módulo | `snake_case` | `kebab-case` | `kebab-case` |
| Arquivo | `snake_case.dart` | `kebab-case.tipo.ts` | `PascalCase.vue` (componentes), `kebab-case.ts` |
| Classe | `PascalCase` | `PascalCase` | `PascalCase` |
| Interface/contrato | classe abstrata sem prefixo (`ProductService`) | `I` + `PascalCase` (`IProduct`) | `I` + `PascalCase` |
| Implementação | `ProductServiceImpl` | `ProductService` (concreto Nest) | — |
| Entidade ORM | — | `ProductEntity` | — |
| DTO | `CreateProductDto` | `CreateProductDto` | `CreateProductDto` |
| Teste | `<nome>_test.dart` | `<nome>.spec.ts` / `.e2e-spec.ts` | `<nome>.spec.ts` |
| Provider/store | `productServiceProvider` | — | `useProductStore` |
| Pasta auxiliar de módulo | sem prefixo (`domain/`, `data/`) | prefixo `_` (`_dtos`, `_interfaces`) | — |

## Regras de vocabulário

- Não importar sigla de programa/órgão específico para código genérico
  (ex.: o SEMI proíbe "PNAE"/"FNDE"). Use termos genéricos do domínio.
- Rotas em inglês, plural, `kebab-case`: `/tenant/:tenantId/products`.
  Exceção para sistema de consulta central de propósito único (`/cnpj/:cnpj`, `/search`):
  ver `api-contract.md`, seção Rotas.

## Texto de interface

- **Proibido texto TODO EM MAIÚSCULAS** na UI (cabeçalhos de tabela, badges,
  botões, títulos). Use *Sentence case* ou *Title Case*
  (`Emissão`, `Pendente`, `Salvar proposta`).
- Única exceção: códigos/siglas curtos de identificação física (ex.:
  `LOT-2026-089`).
- Não usar `text-transform: uppercase` nem equivalente.
