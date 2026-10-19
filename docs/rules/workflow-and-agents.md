# Fluxo de trabalho e papéis de agentes

> Modelo usado no SEMI. Adapte ao tamanho do projeto: em projeto pequeno,
> um único agente cumpre vários papéis, **mas as regras de cada papel
> continuam valendo** (não escrever código antes da tarefa, testes antes da
> implementação etc.).

## Papéis

| Papel | Faz | Nunca faz |
|-------|-----|-----------|
| **CTO-Master** (só em sistemas multi-app) | Recebe a demanda, avalia contra o código real, decide quem é afetado (front/back/ambos), guarda e atualiza o contrato compartilhado, delega | Escrever código de produção ou teste; permitir mudança de contrato sem registrar |
| **CTO do app** (backend ou frontend) | Lê código real, cria `.tasks/NN-slug.md` e atualiza o manifesto, define contratos (interfaces/skeletons), delega | Escrever código de produção ou teste; decidir domínio sem o Master |
| **QA** | Lê a tarefa, escreve testes antes da implementação a partir dos critérios, valida no fim | Implementar produção; inventar contrato |
| **Senior** | Implementa até os testes do QA passarem, seguindo SOLID/hexagonal/DI | Decidir o que construir; escrever testes do zero; redefinir interfaces |
| **Versionador** | Executa `versioner` sob pedido explícito | Agir sem pedido |

Demanda recebida **direto do dono** por um CTO especialista → **parar e
escalar** ao CTO-Master antes de criar tarefa ou escrever código,
principalmente se tocar domínio compartilhado.

## Guardião do contrato

Exigem validação e registro **antes** de qualquer implementação:

- Entidade/tabela (nova, alterada, removida) — **toda alteração em entity
  precisa de aprovação explícita do dono**, mesmo que pequena;
- DTO de entrada/saída (campos, tipos, obrigatoriedade);
- Rota de API (nova, alterada, removida, query params);
- Enum/constante compartilhada;
- Envelope de resposta e formato de erro;
- Regra de autenticação ou permissão.

## Toda tarefa formalizada antes do código

Arquivo `.tasks/NN-slug.md` (`NN` sequencial) + linha em
`.tasks/00-manifest.md` (colunas `Seq | File | Objetivo | Status | Depends on`;
status `todo`, `doing`, `done`, `blocked`). Modelo em
`templates/task.md`.

Seções obrigatórias: **Objetivo**, **Contexto**, **Referências**,
**Critérios de aceitação** (sempre verificáveis: comando, rota ou teste).
Quando couber: **Regras de negócio**, **Não escopo**, detalhes técnicos.

Critério de aceitação universal (em toda tarefa): arquivos em UTF-8 sem
BOM, LF, sem *mojibake* (ver `encoding-and-files.md`).

## Fluxo

```
Dono → CTO-Master → CTO do app → cria tarefa
                                → QA escreve testes (red)
                                → Senior implementa (green)
                                → CTO atualiza manifesto
```

## Antes de atualizar specs a partir do código

Verifique a branch atual do app (`git branch --show-current`) e registre na
spec a **data** da verificação, nunca o nome da branch.

## Restrições de autorização comuns

- **Nenhum agente gera commit com `!` ou `BREAKING CHANGE:`** sem autorização
  expressa do dono; commits sempre atômicos e mínimos
  (`commits-and-versioning.md`).

Cada projeto declara no seu `CLAUDE.md` as zonas protegidas. Padrões usados
no SEMI (herdar quando o projeto declarar):

- **Agentes nunca criam nem executam migrations de banco** — são do usuário.
- Pastas de infraestrutura compartilhada (`core/`, `lib/` no NestJS) só
  mudam com autorização explícita **pedida antes** de editar.
- Nunca editar `package.json`/`pubspec.yaml` à mão para dependências: use
  `npm install/uninstall`, `fvm flutter pub add/remove`.
