# 01 — Entrada e saída por stdin e stdout na CLI

## Objetivo

Permitir `thumbcraft - -`: ler os bytes do documento da stdin e gravar a imagem na stdout, para o wrapper Node
chamar o binário sem arquivos temporários. Também aceitar `-` só na entrada ou só na saída.

## Contexto

`docs/DISTRIBUTION_NPM.md` (seção 4 e passo 2 da seção 7) usa `fromBuffer(...).toImageBytes()`, que depende disso.
Hoje a CLI só lê e grava arquivos (`crates/thumbcraft/src/main.rs`, `open.rs`). O roteamento usa extensão e
conteúdo (`crates/core/src/route.rs`); sem nome de arquivo, só o conteúdo decide. Os motores abrem por caminho
(`Word::open(path)` etc.): verificar se aceitam bytes; se não, usar arquivo temporário interno como adaptador.

## Referências

- `docs/ARCHITECTURE.md` (códigos de saída, regras de erro), `docs/DISTRIBUTION_NPM.md`
- `crates/thumbcraft/src/cli.rs`, `main.rs`, `open.rs`, `encode.rs`
- `docs/rules/testing.md`, `docs/rules/principles.md` (injeção de dependências: ler e gravar por abstração para testar sem I/O real)

## Regras de negócio

- `-` como ENTRADA lê toda a stdin em memória, com limite de tamanho (valor a decidir e documentar; acima dele, erro 4).
- `-` como SAÍDA grava só os bytes da imagem na stdout; nada mais pode sair na stdout (mensagens continuam na stderr).
- Opção nova `--input-name <nome>`: dica de nome (extensão) para o roteamento quando a entrada é `-`. Sem ela,
  decide só o conteúdo, como já ocorre com extensão desconhecida.
- `--detect -` também lê da stdin (usado por `isValid` com buffer, tarefa 03).
- Códigos de saída inalterados (2, 3, 4, 5). Falha ao ler a stdin ou ao gravar a stdout é erro 5.
- Contrato novo: registrar em `docs/ARCHITECTURE.md` e no `--help` antes da implementação.

## Detalhes técnicos

- Separar a leitura e a gravação em uma abstração injetável (fonte de bytes e destino), com `Fake` nos testes.
- O modo `--native-scale` também aceita `-` na saída.

## Critérios de aceitação

- [ ] Testes escritos antes e falhando pelo motivo certo, depois passando: `cargo test --workspace --locked`
- [ ] `thumbcraft - - -W 120 < corpus/docx/01_docx_simples_acentos.docx > out.png` gera um PNG válido de 120 px de largura
- [ ] Mesma entrada por arquivo e por stdin produz imagens idênticas byte a byte
- [ ] `--input-name x.pptx` roteia para o DeckCraft quando o conteúdo é ambíguo
- [ ] `thumbcraft --detect - < arquivo` imprime o mesmo motor que `--detect arquivo`
- [ ] Entrada vazia na stdin dá código 4 e nada na stdout; saída para stdout fechada dá código 5
- [ ] A stdout contém somente a imagem (teste e2e confere os primeiros bytes e que a stderr tem as mensagens)
- [ ] `thumbcraft --help` e `docs/ARCHITECTURE.md` descrevem `-` e `--input-name`
- [ ] **UTF-8 sem BOM, LF e sem mojibake** em todos os arquivos criados/modificados (`bash scripts/check-encoding.sh`)

## Não escopo

- Streaming incremental (a imagem só é gravada pronta). Vários arquivos por processo. Mudar códigos de saída.
