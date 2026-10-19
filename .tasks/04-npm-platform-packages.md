# 04 — Pacotes de plataforma e resolução do binário

## Objetivo

Criar os pacotes `@thumbcraft/win32-x64-msvc` e `@thumbcraft/linux-x64-gnu` (um binário cada) e a resolução, no
pacote principal, do caminho do binário instalado, via `optionalDependencies` com `os`, `cpu` e `libc`.

## Contexto

`docs/DISTRIBUTION_NPM.md` seções 1 a 3. Escopo de plataformas decidido em 2026-10-19: **só Windows x64 e Linux x64
(glibc)**. Depende da tarefa 02 (interface de resolução do binário). Desenho decidido pelo dono, como o `sharp`:
o pacote principal `thumbcraft` declara `optionalDependencies` para os dois pacotes de plataforma, e o npm instala
só o que combina com `os`, `cpu` e `libc`. Cada pacote de plataforma tem seu `package.json`, com versão igual à do
principal. Quem desenvolve no Windows e roda em container Linux declara `supportedArchitectures` (npm 10 ou mais
novo) no projeto dele para instalar os dois binários; isso é só documentado aqui, não implementado.

## Referências

- `docs/DISTRIBUTION_NPM.md`, `docs/BUILD.md`, `docs/SIZES.md`, tarefa 02

## Regras de negócio

- O pacote de plataforma contém só o binário e o `package.json`; nenhum arquivo do corpus ou dos baselines.
- Plataforma sem pacote correspondente: erro claro, em português, dizendo qual plataforma e como usar `THUMBCRAFT_BIN`.
- Binários nunca entram no git; são gerados no empacotamento.

## Detalhes técnicos

- Script de empacotamento em `packages/` que monta cada pacote de plataforma a partir de um binário informado e de uma versão.
- Resolução por `require.resolve` do pacote de plataforma, atrás da interface injetável da tarefa 02.

## Critérios de aceitação

- [ ] `npm test` passa, com testes escritos antes (resolução simulada por plataforma)
- [ ] O script monta o pacote Windows x64 e o Linux x64 a partir de binários locais; `npm pack --dry-run` lista só `package.json` e o binário
- [ ] Instalar o pacote principal e o de plataforma em um projeto de teste e rodar `Thumbcraft.fromFile(...)` gera a imagem, no Windows
- [ ] Plataforma sem pacote produz o erro descrito
- [ ] A documentação mostra o `supportedArchitectures` para desenvolver no Windows e rodar no Linux
- [ ] **UTF-8 sem BOM, LF e sem mojibake** em todos os arquivos criados/modificados (`bash scripts/check-encoding.sh`)

## Não escopo

- Publicação no npm e CI (tarefa 06). macOS, ARM e Alpine (musl).
