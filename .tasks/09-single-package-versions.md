# 09 — Uma versão por pacote: eliminar as duplicações que restam no binário

## Objetivo

Fazer `cargo tree -p thumbcraft -d -e normal --locked` listar só o que é impossível unificar, tirando do binário as
duplicações de `skrifa`, `read-fonts`, `font-types` e `miniz_oxide`, com o mínimo de mudança nos projetos importados.

## Contexto

Regra do dono (2026-10-19): o thumbcraft junta projetos para ficar pequeno, padronizado e eficiente; versões diferentes
do mesmo pacote não cabem em nenhum dos três. A tarefa 08 tirou as duplicações que o PDF introduziu (`craft-pdf`,
`vello_cpu`, `vello_common`, `glifo`). Sobram as que já existiam (causas em `docs/UPSTREAM-CHANGES.md`):

| Pacote | Versões | Quem puxa |
|---|---|---|
| `skrifa` | 0.42.1, 0.44.0, 0.47.0 | 0.42.1: `krilla` e `subsetter`; 0.44.0: `glifo`; 0.47.0: motores e `hayro-interpret` |
| `read-fonts` | 0.39.2, 0.41.0, 0.43.3, 0.44.0 | acompanha `skrifa` |
| `font-types` | 0.11.3, 0.12.6 | acompanha `skrifa` |
| `miniz_oxide` | 0.8.9, 0.9.1 | 0.8.9: `png`, `wordcraft-docbin`, `wordcraft-proof`; 0.9.1: `flate2` |

`syn` 2 e 3 são só de tempo de compilação e ficam fora.

## Referências

- `docs/UPSTREAM-CHANGES.md`, `docs/UPDATING.md`, `.tasks/08-pdf-engine.md` (estudo do `krilla`: ~270 KiB de código)
- `patches/craft-libs/0001-hayro-vello-cpu-0.2.patch` (modelo de patch mínimo)
- `docs/rules/testing.md`, `principles.md`

## Decisão do dono (2026-10-19): opção A

Feita: ver "Resultado da opção A" no fim. Opções consideradas:

- **A. Substituto vazio do `krilla`** (o thumbcraft nunca exporta PDF): tira `krilla`, `subsetter` e o `skrifa` 0.42.1
  com seus `read-fonts` e `font-types`. Exige patch no `craft-pdf` (feature `write`) e no `wordcraft-pdf`.
- **B. Alinhar `krilla` e `glifo` ao `skrifa` 0.47** por patch de versão: mantém o código, muda só manifestos e
  chamadas que quebrarem.
- **C. `miniz_oxide`:** tentar `cargo update` ou fixar versões; se a faixa de `png`/`flate2` não permitir, registrar
  como inevitável.

## Regras de negócio

- Cada mudança no código importado é um patch pequeno em `patches/<projeto>/`, registrado em
  `docs/UPSTREAM-CHANGES.md` (o que, por quê, como reaplicar, como saber se ainda é preciso).
- Sem regressão: testes do workspace, corpus e baselines iguais; tamanho do binário não maior que 26.701.824 bytes.

## Critérios de aceitação

- [x] Opção escolhida pelo dono e registrada aqui (A)
- [x] Testes do substituto e `cargo test --workspace --locked` passam
- [ ] (parcial) `cargo tree -p thumbcraft -d -e normal --locked` não lista `skrifa`, `read-fonts` nem `font-types`; o que sobrar de
      `miniz_oxide` está justificado em `docs/UPSTREAM-CHANGES.md`
- [x] Binário de release medido e comparado com 26.701.824 bytes (25.655.296)
- [x] `docs/UPSTREAM-CHANGES.md` atualizado com cada patch novo
- [x] **UTF-8 sem BOM, LF e sem mojibake** (`bash scripts/check-encoding.sh`)

## Resultado da opção A (2026-10-19)

`stubs/wordcraft-pdf` (crate do thumbcraft, com 2 testes) mais `patches/wordcraft/0001-wordcraft-pdf-stub.patch`
(uma linha no `Cargo.toml` do WordCraft). Saíram do binário `krilla`, `subsetter`, o `craft-pdf` com `write`,
`skrifa` 0.42.1, `font-types` (as duas versões) e `read-fonts` 0.39.2, e os remendos de `krilla` e `craft-pdf` da raiz.
Binário de release: **25.655.296 bytes, 1.046.528 a menos** (−3,9%) que os 26.701.824 de antes; contra os 23.995.392 sem
PDF, o motor de PDF passa a custar **+1,66 MB**. Testes: 21 + 26 + 81 + 10 + 2 do substituto, todos verdes.

**Falta para fechar** (status segue `doing`):

- `skrifa` ainda tem 0.44.0 (do `glifo` do `vello_cpu` 0.2.0) e 0.47.0 (motores e hayro), e `read-fonts` tem 3 versões:
  opção B, alinhar o `glifo` ao `skrifa` 0.47 por patch no `vello_cpu`/`glifo`, que não é do craft-libs e sim do
  crates.io (exigiria vendorizar o `glifo` ou o `vello_cpu`: decidir com o dono se vale).
- `miniz_oxide` 0.8.9 (png, wordcraft-docbin/proof) e 0.9.1 (flate2): opção C, ainda não tentada.

## Não escopo

- Subir revisões dos projetos importados; reescrever o núcleo deles.
