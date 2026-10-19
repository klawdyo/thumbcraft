# Codificação e fim de linha

> Vale para todos os repositórios.

## Regra

| O que | Padrão |
|-------|--------|
| Charset de todo arquivo (código, config, doc) | **UTF-8 sem BOM** |
| Fim de linha | **LF**, nunca CRLF |
| Acentos e símbolos (`º`, `ª`, `•`, `·`, `ç`) | Gravar direto como Unicode real |

Todo repositório tem na raiz:

- `.editorconfig` com `charset = utf-8`, `end_of_line = lf`,
  `insert_final_newline = true`, `indent_style = space`.
- `.gitattributes` com `* text=auto eol=lf`.

Templates prontos em `templates/` deste repositório.

## Regras para agentes

- Nunca introduzir BOM (bytes `0xEF 0xBB 0xBF` no início).
- Nunca usar CRLF.
- Nunca colar texto vindo de terminal com codepage errada. Se achar
  *mojibake* (`Ã£`, `Ã§`, `â€¢`, `Âº`, `Â`) em arquivo existente, corrija no
  mesmo commit.
- No Windows, ferramentas como PowerShell 5 (`Set-Content`, `>`) podem gravar
  BOM/ANSI: sempre passe `-Encoding utf8NoBOM` (PowerShell 7) ou use as
  ferramentas de escrita do agente.

## Critério de aceitação universal (toda tarefa)

Antes de declarar qualquer tarefa concluída:

1. Nenhum arquivo criado/modificado começa com BOM.
2. Nenhum arquivo criado/modificado contém *mojibake*.
3. Nenhum arquivo criado/modificado contém CRLF.

Essa checagem entra como critério em **toda** tarefa (ver
`workflow-and-agents.md`) e é **regra de aceitação não negociável**: sem
rodar e passar, a tarefa não está concluída — vale para código, docs,
specs, configs e arquivos gerados por agente.

## Como verificar

Script pronto em `templates/check-encoding.sh` (copie para o projeto, ex.:
`scripts/check-encoding.sh`):

```bash
bash scripts/check-encoding.sh            # arquivos modificados/novos do git
bash scripts/check-encoding.sh a.ts b.md  # arquivos específicos
```

Ele falha (exit 1) se achar BOM, CRLF ou mojibake. O agente **roda o script
e cola o resultado** no encerramento da tarefa.
