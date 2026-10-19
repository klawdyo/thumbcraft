# Baseline PowerPoint (DeckCraft contra LibreOffice)

O que é: resultados do teste de thumbnail (slide 1 em PNG) em 46 arquivos de entrada (`../../corpus/pptx/`), com o motor do DeckCraft (binário mínimo `dc-min`) e com o LibreOffice. Referência para comparar versões futuras do DeckCraft e o binário unificado.

De quando: 2026-10-19.

## Binários e versões

| Item | Versão |
|---|---|
| DeckCraft | commit `010941ec60f81f1eece31ec1ea83825c9a396217` (2026-10-18), versão 0.8.0 (alpha) |
| `dc-min` | `prototipo-dc-min/main.rs`, depende só de `deckcraft-pptx`, `deckcraft-format`, `deckcraft-render` e `deckcraft-fonts` |
| Perfil de build | `opt-level=z`, LTO fat, `codegen-units=1`, `panic=abort`, `strip=symbols` |
| Tamanho do `dc-min` | 6.673.360 bytes = 6,67 MB = 6,36 MiB |
| Outros tamanhos | CLI padrão do repositório 35.296.984 bytes (35,30 MB, 33,66 MiB); CLI com perfil de tamanho 14.504.104 bytes (14,50 MB, 13,83 MiB) |
| Rust | 1.97.0 |
| LibreOffice | 24.2.7.2, `soffice --headless --convert-to png` |
| Fontes | do sistema (Carlito e Caladea como equivalentes métricos de Calibri e Cambria); o `craft-fonts` externo não foi usado |

## Como foi gerado

- Rust: `dc-min ENTRADA SAIDA.png 1` (escala 1, slide 0), um processo por arquivo, timeout de 30 s. O PNG tem 960x540 px. O script é `ferramentas/run.sh`, que roda os dois motores no mesmo laço (perfil dedicado do LibreOffice e `pkill` do processo ao fim de cada arquivo).
- LibreOffice: PNG de 1280x720 px, um processo novo por arquivo, timeout de 30 s.
- Cache de disco quente, 2 vCPUs compartilhadas com outro agente que compilava (tempos de build inflados).

## Conteúdo

| Caminho | O que é |
|---|---|
| `resultados-deckcraft-vs-libreoffice.tsv` | uma linha por arquivo, com exit, ms, bytes do PNG e classe (ok, degradado, falhou) para os dois motores, e notas |
| `resultados-brutos.tsv` | duas linhas por arquivo (uma por motor), com o stderr truncado; caminhos absolutos da sessão ainda aparecem na coluna stderr |
| `imagens/contact-sheet-deckcraft-46-arquivos.png` | contact sheet dos 46 casos (reduzida a 1600 px) |
| `imagens/comparacao-deckcraft-vs-libreoffice-12-casos.png` | 12 casos lado a lado (reduzida a 1600 px) |
| `amostras/` | 7 casos reduzidos a 800 px de largura, um por motor |
| `prototipo-dc-min/` | fonte do binário mínimo |
| `ferramentas/` | `run.sh` e os scripts de build (`build-default.sh`, `build-opt.sh`) |
| `logs-de-build/` | logs dos três builds (CLI padrão, CLI otimizada e mínimo) |

A pasta `../../corpus/pptx/mutacao-48/` tem o teste de robustez com 48 variantes mutadas, com seus resultados.

## Resumo dos números

Tabelas completas em `../../docs/RESULTS.md`.

| | DeckCraft (`dc-min`) | LibreOffice |
|---|---|---|
| PNG gerado | 38 de 46 | 45 de 46 |
| ok, degradado, falhou | 34, 4, 8 | 40, 3, 3 |
| Tempo médio sobre os 46 | 83 ms (98 ms só nos 38 sucessos) | 1792 ms |
| Tempo máximo | 970 ms (arquivo com árabe, CJK e emoji) | 2977 ms |

Das 8 falhas do DeckCraft, 5 são formatos que ele não lê (.ppt e .odp) e 3 são entradas inválidas rejeitadas com exit 1. O LibreOffice dá exit 0 com PNG em branco ou de texto minúsculo no arquivo de 0 bytes e no texto puro (falso sucesso).

## Como repetir

Ver `../../docs/TESTS.md` (seção "Thumbnails PowerPoint") e `../../docs/UPDATING.md`.
