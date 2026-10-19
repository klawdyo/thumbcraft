# Baseline Word (WordCraft contra LibreOffice)

O que é: resultados do teste de thumbnail (página 1 em PNG) em 30 arquivos de entrada (`../../corpus/docx/`), com o motor do WordCraft (binário mínimo `wc-min`) e com o LibreOffice. Serve de referência para comparar versões futuras do WordCraft e o binário unificado.

De quando: 2026-10-19.

## Binários e versões

| Item | Versão |
|---|---|
| WordCraft | commit `117aea31a8e61bd3e306e9210800cdc42897e3c1` (2026-10-19), clone da organização storytold |
| `wc-min` | 15 linhas em `prototipo-wc-min/main.rs`, dentro do workspace do WordCraft (`apps/wc-min`), depende só de `wordcraft-engine`, `wordcraft-doc` e `serde_json` |
| Perfil de build | `opt-level=z`, LTO fat, `codegen-units=1`, `panic=abort`, `strip=symbols` (variáveis `CARGO_PROFILE_RELEASE_*`) |
| Tamanho do `wc-min` | 19.763.984 bytes = 19,76 MB = 18,85 MiB |
| Rust | 1.97.0 |
| LibreOffice | 24.2.7.2, `soffice --headless --convert-to png`, processo novo por arquivo |
| Máquina | 2 vCPUs compartilhadas com outros agentes (ver `../../docs/ENVIRONMENT.md`) |

## Como foi gerado

- Rust: `wc-min ENTRADA SAIDA.png 1`, página 1, escala 2 (px por pt), um processo por arquivo, limite de 30 s. O PNG sai em Letter (1224x1584) com fundo transparente. As miniaturas e amostras deste diretório foram achatadas sobre branco.
- LibreOffice: um processo `soffice` novo por arquivo. Todas as 30 execuções terminaram com exit 0, mas só 25 geraram PNG.
- Tempos medidos em segundos pelo relógio da parede, com cache de disco quente.

## Conteúdo

| Caminho | O que é |
|---|---|
| `resultados-wordcraft-wc-min.tsv` | por arquivo: exit, segundos, bytes do PNG e mensagem de erro |
| `resultados-libreoffice.tsv` | por arquivo: exit, segundos, bytes do PNG (0 quando não gerou) |
| `imagens/contact-sheet-wordcraft-30-arquivos.png` | contact sheet dos 30 casos, falhas em vermelho (reduzida a 1600 px de largura) |
| `imagens/comparacao-wordcraft-vs-libreoffice-6-casos.png` | lado a lado nos casos 03, 06, 08, 09, 11 e 12 (reduzida a 1600 px) |
| `amostras/` | 7 casos em PNG reduzido a 700 px de largura, um por motor (`.wordcraft.png` e `.libreoffice.png`) |
| `prototipo-wc-min/` | fonte do binário mínimo (`Cargo.toml.txt` é o `Cargo.toml`, renomeado para não ser lido pelo cargo) |

## Resumo dos números

Tabelas completas em `../../docs/RESULTS.md`.

| | WordCraft (`wc-min`) | LibreOffice |
|---|---|---|
| PNG gerado | 25 de 30 | 25 de 30 (não são os mesmos 25) |
| Tempo médio | 0,124 s nos sucessos (0,105 s sobre os 30) | 1,48 s sobre os 30 (1,52 s sobre os 25 com PNG) |
| Tempo máximo | 0,245 s | 4,09 s |
| Qualidade (WordCraft) | 18 ok, 7 degradados, 5 falhas | não classificada arquivo a arquivo |

As 5 falhas do WordCraft são entradas inválidas, com exit 1 e mensagem de erro limpa. O LibreOffice dá exit 0 mesmo sem gerar PNG (13, 23, 24, 26 e 29) e gera PNG em entradas inválidas (por exemplo texto puro e lixo binário).

## Como repetir

Ver `../../docs/TESTS.md` (seção "Thumbnails Word") e `../../docs/UPDATING.md`.
