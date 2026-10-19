# 06 — CI e publicação no npm

## Objetivo

Criar o workflow `.github/workflows/release.yml`: a cada push na `main` roda os testes (Windows e Ubuntu) e, se a
versão do `Cargo.toml` ainda não estiver no npm, compila o binário para Windows x64 e Linux x64, monta os três
pacotes e os publica automaticamente, sem tag e sem token, pela publicação confiável (OIDC) do npm.

## Contexto

`docs/DISTRIBUTION_NPM.md` seções 6 a 8 trazem o desenho, o rascunho do workflow e o passo a passo do npm.
Decisões do dono (2026-10-19): publicação automática a cada push na `main`; sem token do npm (publicação
confiável); só Windows x64 e Linux x64. Como o npm recusa a mesma versão duas vezes, o workflow só publica quando a
versão do `Cargo.toml` não existe no npm, o que acontece com o commit `release: X.Y.Z` do `versioner`; os outros
commits só rodam os testes. Depende das tarefas 04 (pacotes e script `build:packages`) e 05. Pendências do dono, no
npmjs.com, antes do primeiro release automático (seção 8 do plano): criar a organização `thumbcraft`, publicar à
mão a primeira versão dos três pacotes e configurar o Trusted Publisher de cada um com o repositório e o nome
`release.yml`. O segredo `NPM_THUMBCRAFT` criado no GitHub não é mais necessário.

## Referências

- `docs/DISTRIBUTION_NPM.md`, `docs/STATUS.md`, `docs/UPDATING.md`, `scripts/fetch-upstream.sh`
- `docs/rules/commits-and-versioning.md` (o `versioner` ganhará suporte a `Cargo.toml` em outra tarefa, em andamento no repositório dele)

## Regras de negócio

- Versão única: a versão do `Cargo.toml` define a do binário e a dos pacotes npm; um script (não edição à mão)
  copia essa versão para os `package.json` na hora do empacotamento. A versão em si é calculada pelo `versioner`.
- Nenhum segredo nem `NODE_AUTH_TOKEN`: a autenticação é o `id-token: write` do job de publicação. O agente não pede
  nem lê credenciais.
- Publicar só em push na `main` e só com versão ainda inexistente no npm; pull requests só rodam testes.
- Ordem de publicação: pacotes de plataforma primeiro, pacote principal por último.
- Requisitos: runner hospedado pelo GitHub, Node 22.14 ou mais novo, npm 11.5.1 ou mais novo, `repository.url`
  igual ao do repositório em todos os `package.json`.
- O workflow não cria tag nem faz commit por conta própria.
- Cache do cargo com atenção ao tamanho (build limpo leva minutos).

## Critérios de aceitação

- [ ] Em push e pull request, o job de testes roda `cargo test --workspace --locked` e `npm test` em Windows e Ubuntu, e passa
- [ ] Com a versão do `Cargo.toml` já publicada no npm, o push na `main` roda só os testes e não tenta publicar
- [ ] Com uma versão nova, o workflow compila os dois binários, monta os três pacotes e publica na ordem certa (verificado primeiro com `npm publish --dry-run`)
- [ ] Documentação do release em `docs/` (seção 8 do plano): organização, primeira publicação manual, configuração dos três pacotes e como fazer um release
- [ ] **UTF-8 sem BOM, LF e sem mojibake** em todos os arquivos criados/modificados (`bash scripts/check-encoding.sh`)

## Não escopo

- Fazer o release de verdade (pedido explícito do dono). Aumentar a versão automaticamente a cada commit. Criar tags. macOS, ARM e Alpine (musl).
