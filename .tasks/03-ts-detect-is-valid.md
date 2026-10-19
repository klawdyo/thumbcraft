# 03 — `detect` e `isValid` por `thumbcraft --detect`

## Objetivo

Oferecer `Thumbcraft.detectFile`, `detect` (buffer), `isValidFile` e `isValid`, que dizem se o arquivo é suportado
e por qual motor, **sem reimplementar o roteamento em TypeScript**: a função apenas chama `thumbcraft --detect`.

## Contexto

O plano original previa uma verificação síncrona em TypeScript espelhando `route.rs`. Isso obrigaria a ler
a estrutura interna de arquivos .docx, .xlsx e .pptx (que são pacotes ZIP) em TypeScript, duplicando lógica do
binário e podendo divergir dele. Decisão de 2026-10-19: não duplicar; o binário já responde a `--detect`
(`crates/thumbcraft/src/main.rs`, `open.rs`). Custo: criar um processo por consulta (dezenas de milissegundos) e
API assíncrona. Depende da tarefa 02 (execução do binário) e, para buffer, da tarefa 01 (`--detect -`).

## Referências

- `crates/thumbcraft/src/main.rs` (`--detect`), `docs/ARCHITECTURE.md`, `docs/DISTRIBUTION_NPM.md` (seção 4.1)
- `docs/rules/testing.md`, `naming-and-language.md`

## Regras de negócio

- `detect*` devolve `'word' | 'grid' | 'deck' | 'pdf'` (o PDF passou a ter motor na tarefa 08) ou `null` quando o binário recusa com código 3 (formato não suportado).
- Código 4 (arquivo ilegível ou inexistente) e qualquer outro erro de processo propagam o erro tipado da tarefa 02; não viram `null`.
- `isValid*` é `detect*` diferente de `null`.
- Nenhuma leitura de ZIP, tabela de assinaturas ou lista de extensões no código TypeScript.

## Critérios de aceitação

- [x] `npm test` passa, com testes escritos antes, usando `IProcessRunner` falso
- [x] Teste de integração (com `THUMBCRAFT_BIN`): para todo arquivo de `corpus/docx`, `corpus/xlsx`, `corpus/pptx` e `corpus/pdf`, o resultado de `detectFile` bate com o texto impresso por `thumbcraft --detect`
- [x] Arquivo `.ppt` retorna `null` e PDF retorna `'pdf'`; arquivo inexistente lança `InvalidFileError`
- [x] **UTF-8 sem BOM, LF e sem mojibake** em todos os arquivos criados/modificados (`bash scripts/check-encoding.sh`)

## Não escopo

- Validar se o documento renderiza (isso é o binário). Qualquer análise de ZIP em TypeScript.

## Resultado (2026-10-19)

`Thumbcraft.detectFile/detect/isValidFile/isValid` em `src/thumbcraft.ts`; o runner passou a capturar a stdout e a aceitar stdin.
No corpus, um arquivo vazio em disco é roteado pela extensão, mas o mesmo conteúdo pela stdin é erro 4 (contrato da tarefa 01); o teste de
integração trata esse caso. `npm test` com `THUMBCRAFT_BIN`: 99 testes verdes.
