# Regressão do PDF do GridCraft (fase 1 da refatoração)

O que é: o teste que provou que a refatoração do `print.rs` do GridCraft (fase 1, só um arquivo) não muda a saída do PDF. Compara PDFs gerados pelo binário do código original (os "goldens") contra os gerados pela cópia refatorada, em três camadas. Resultado final: 325 de 325 casos PASS, 306 de 306 PDFs idênticos byte a byte, 0 pixels diferentes.

De quando: 2026-10-19. Serve para qualquer atualização futura do GridCraft: se o PDF do upstream mudar de propósito, os goldens precisam ser refeitos com o novo commit original.

## Versões

| Item | Valor |
|---|---|
| Commit original do GridCraft (goldens) | `66ce6be4743f4227047f76f56815520f824e6d81`, árvore sem alterações (ver `ORIGIN.txt`) |
| Binário original | `gridcraft-cli` compilado com `cargo build -p gridcraft-cli` (perfil dev otimizado do próprio repositório, com debuginfo, 221 MB), sha256 `8d22cb4dbcef502d6d204c9d91b1a39d6b593e974e00b0ffd99032ab36eb3c3c` |
| Rust | 1.97.0 |
| Ferramentas | `cmp`, `qpdf` 11.9.0 (camada B), `pdftoppm` 24.02.0 e ImageMagick `compare` 6.9.12 (camada C) |

## As três camadas

- A, bytes: `cmp` entre o PDF golden e o novo. Nenhuma normalização (o gravador não emite data, `/ID` nem UUID, conferido nos 306 PDFs). A lista de normalização do script é vazia de propósito.
- B, streams de conteúdo: só roda quando A falha. `qpdf --qdf --object-streams=disable` nos dois PDFs e diff das operações.
- C, pixels: `pdftoppm` a 50 dpi, página a página, `compare -metric AE`. Esperado: 0.
- Além delas, o texto de saída e o código de saída de cada caso devem ser iguais, inclusive nos 19 casos sem PDF (erros).

## Cobertura

64 planilhas de entrada: as 16 de `../../corpus/xlsx/regressao-amostras/` (prefixo `s-`) mais as 48 .xlsx de `../../corpus/xlsx/` (prefixo `c-`). Cada uma em 5 variantes: `exportPdf` padrão (`.d`), `fitToPage` (`.fit`), `range` B2:F30 (`.rng`), `sheets:"all"` (`.all`) e `file.print` (`.prn`). Mais 5 casos de erro com parâmetros inválidos (`err.*`). Total: 64 x 5 + 5 = 325 casos, 306 deles com PDF.

## Conteúdo

| Arquivo | O que é |
|---|---|
| `regress-pdf.sh` | o script (bash). `golden` gera os goldens com o binário original, `check` (padrão) recompila a cópia, regenera e compara. Sai com código diferente de zero se houver qualquer FAIL |
| `tabela-final-325-de-325.txt` | tabela por caso da execução final (o resultado aceito) |
| `tabela-teste-de-sanidade-do-detector.txt` | teste de sanidade: foi injetado um defeito (deslocar o `x` de todo `FillRect` em 1 unidade) e o detector acusou FAIL em 99 casos (226 de 325 PASS), com as três camadas acusando, e depois de reverter voltou a 325 de 325 |
| `SHA256SUMS` | sha256 dos 306 PDFs golden (145 conteúdos distintos, porque variantes iguais geram PDFs iguais) |
| `ORIGIN.txt` | commit original, indicador de árvore limpa (0) e sha256 do binário original |
| `fase1-print-rs.patch` | o patch da fase 1: só `crates/engine/src/cmd/print.rs`, +118 e -14 linhas (`enum Op`, `struct Rec`, `fn replay`) |

Os PDFs golden (19 MB, 306 arquivos) e o binário original (221 MB) não foram copiados. Os hashes bastam para provar a identidade.

## Como regenerar os goldens e repetir o teste

1. Obtenha o GridCraft no commit `66ce6be4743f4227047f76f56815520f824e6d81` (`scripts/fetch-upstream.sh` faz isso em `third_party/gridcraft`, mas já com o patch aplicado: use um clone limpo para o original) e compile o binário original: `cargo build -p gridcraft-cli`. Guarde uma cópia do binário fora do `target`, porque o binário patchado tem o mesmo nome.
2. Copie esta pasta para um diretório de trabalho fora do repositório (o script grava `golden-pdf/` e `new-pdf/` ao lado dele, e os PDFs somam dezenas de MB).
3. Defina as variáveis e gere os goldens: `SAMPLES=<repo>/corpus/xlsx/regressao-amostras CORPUS=<repo>/corpus/xlsx ORIG_BIN=<binário original> ./regress-pdf.sh golden`.
4. Confira com o `SHA256SUMS` daqui: `cd golden-pdf/pdf && sha256sum -c <repo>/baselines/gridcraft-pdf-regression/SHA256SUMS`. Todos devem dar OK se o commit e o Rust forem os mesmos.
5. Aplique o patch (ou a versão nova) em um clone, e rode `COPY=<clone patchado> SAMPLES=... CORPUS=... ORIG_BIN=... ./regress-pdf.sh check`. Requer `qpdf`, `pdftoppm` e `compare`.

O script aceita também `NO_BUILD=1`, `NEW_BIN`, `CARGO_TARGET_DIR` e `DPI`. Sem `NEW_BIN`, ele procura o binário em `$CARGO_TARGET_DIR/debug/gridcraft-cli`.

O gerador das 16 planilhas de amostra era um script Python não versionado. As planilhas geradas estão em `../../corpus/xlsx/regressao-amostras/`, e `../../docs/TESTS.md` descreve a cobertura.
