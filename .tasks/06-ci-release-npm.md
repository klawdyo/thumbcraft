# 06 — CI e publicação no npm

## Objetivo

Criar o pipeline do GitHub Actions que roda os testes a cada alteração e, ao receber uma tag de versão, compila o
binário para Windows x64 e Linux x64, monta e publica os pacotes npm.

## Contexto

`docs/DISTRIBUTION_NPM.md` seção 6 traz um rascunho de workflow, e `docs/STATUS.md` e `docs/UPDATING.md` (item 8)
registram o CI/CD de release como pendência. Depende das tarefas 04 e 05. O dono já criou um token granular
de publicação no npm, chamado `NPM_THUMBCRAFT`; o workflow o lê de `secrets.NPM_THUMBCRAFT` (a alternativa
"trusted publishing" do npm, sem token guardado, fica como possível melhoria futura). A confirmar com o dono antes
de publicar: (1) o segredo está guardado com esse nome exato no GitHub (Settings, Secrets and variables, Actions);
(2) o token tem permissão de publicar o pacote `thumbcraft` e os pacotes do escopo; (3) o escopo (organização)
`@thumbcraft` existe no npm, necessário para os dois pacotes de plataforma da tarefa 04.

## Referências

- `docs/DISTRIBUTION_NPM.md`, `docs/STATUS.md`, `docs/UPDATING.md`, `scripts/fetch-upstream.sh`
- `docs/rules/commits-and-versioning.md` (o `versioner` ganhará suporte a `Cargo.toml` em outra tarefa, em andamento no repositório dele)

## Regras de negócio

- Versão única: a versão do `Cargo.toml` define a do binário e a dos pacotes npm; um script (não edição à mão)
  copia essa versão para os `package.json` na hora do empacotamento. A versão em si é calculada pelo `versioner`.
- Nenhum segredo no repositório; o workflow usa só o nome `NPM_THUMBCRAFT`, e o valor fica nas configurações do GitHub (o agente não o cria, não o lê e não pede o valor).
- Publicação só em tag; testes em todo push e pull request.
- O workflow não cria tag nem faz commit por conta própria.
- Cache do cargo com atenção ao tamanho (build limpo leva minutos).

## Critérios de aceitação

- [ ] Workflow de testes roda `cargo test --workspace --locked` e `npm test` em Windows e Ubuntu, e passa
- [ ] Workflow de release, em tag de teste ou com `npm publish --dry-run`, gera todos os pacotes sem publicar
- [ ] Documentação do processo de release em `docs/` (passo a passo para o dono, incluindo a conta, o escopo `@thumbcraft` e o segredo `NPM_THUMBCRAFT`)
- [ ] **UTF-8 sem BOM, LF e sem mojibake** em todos os arquivos criados/modificados (`bash scripts/check-encoding.sh`)

## Não escopo

- Publicar de verdade (pedido explícito do dono). Criar tags ou fazer push. macOS, ARM e Alpine (musl).
