# 06 — CI e publicação no npm

## Objetivo

Criar o pipeline do GitHub Actions que roda os testes a cada alteração e, ao receber uma tag de versão, compila o
binário para Windows x64 e Linux x64, monta e publica os pacotes npm.

## Contexto

`docs/DISTRIBUTION_NPM.md` seção 6 traz um rascunho de workflow, e `docs/STATUS.md` e `docs/UPDATING.md` (item 8)
registram o CI/CD de release como pendência. Depende das tarefas 04 e 05. Para publicar no npm é preciso uma conta
no npmjs.com e uma credencial de publicação; o dono ainda não tem nenhuma das duas. Caminhos possíveis, a
decidir com o dono: (a) token granular de publicação criado no npmjs.com e guardado como segredo `NPM_TOKEN` no
GitHub; (b) "trusted publishing" do npm (OIDC com o GitHub Actions, sem token guardado; o workflow já declara
`id-token: write` e `--provenance`), que, até onde se sabe, exige que o pacote já exista e ser configurado na
página do pacote no npm. Conferir a documentação atual do npm antes de implementar. Se o desenho com pacotes de
plataforma for mantido (tarefa 04), também é preciso criar o escopo (organização) `@thumbcraft` no npm.

## Referências

- `docs/DISTRIBUTION_NPM.md`, `docs/STATUS.md`, `docs/UPDATING.md`, `scripts/fetch-upstream.sh`
- `docs/rules/commits-and-versioning.md` (o `versioner` ganhará suporte a `Cargo.toml` em outra tarefa, em andamento no repositório dele)

## Regras de negócio

- Versão única: a versão do `Cargo.toml` define a do binário e a dos pacotes npm; um script (não edição à mão)
  copia essa versão para os `package.json` na hora do empacotamento. A versão em si é calculada pelo `versioner`.
- Nenhum segredo no repositório; a credencial é configurada pelo dono nas configurações do GitHub e do npm (o agente não a cria nem a lê).
- Publicação só em tag; testes em todo push e pull request.
- O workflow não cria tag nem faz commit por conta própria.
- Cache do cargo com atenção ao tamanho (build limpo leva minutos).

## Critérios de aceitação

- [ ] Workflow de testes roda `cargo test --workspace --locked` e `npm test` em Windows e Ubuntu, e passa
- [ ] Workflow de release, em tag de teste ou com `npm publish --dry-run`, gera todos os pacotes sem publicar
- [ ] Documentação do processo de release em `docs/` (passo a passo para o dono, incluindo como criar a conta, o escopo e a credencial)
- [ ] **UTF-8 sem BOM, LF e sem mojibake** em todos os arquivos criados/modificados (`bash scripts/check-encoding.sh`)

## Não escopo

- Publicar de verdade (pedido explícito do dono). Criar tags ou fazer push. macOS, ARM e Alpine (musl).
