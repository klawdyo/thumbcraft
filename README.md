# thumbcraft

Um binário simples que recebe um arquivo de documento, planilha ou apresentação e devolve uma imagem (thumbnail). Reúne em um único executável os motores de três projetos da organização storytold, escritos em Rust e sem dependência de LibreOffice, GPU ou serviços externos:

| Motor | Projeto | Entrada |
|---|---|---|
| WordCraft | https://github.com/storytold/wordcraft | Word e afins |
| GridCraft | https://github.com/storytold/gridcraft | planilhas (Excel) |
| DeckCraft | https://github.com/storytold/deckcraft | apresentações (PowerPoint) |

O thumbcraft será usado em outro projeto, em um fluxo de upload de arquivos, junto com outros tipos de arquivo já cobertos. O motor é escolhido pelo conteúdo (e pela extensão) do arquivo.

## Objetivo e opções de imagem

O binário aceita opções de cortar (`--trim` e `--crop`), redimensionar (`-W` e `-H`) e ajustar o conteúdo à caixa de saída (`--fit contain` ou `--fit cover`, com `--gravity` e `--background`). Por isso a renderização interna é feita em alta resolução (os *craft renderizam nativamente, e o thumbcraft reduz o resultado), e só depois a imagem é cortada e reduzida.

## Por que este projeto existe: uma ponte

Os *craft continuam evoluindo. O thumbcraft existe como ponte enquanto isso: por exemplo, o GridCraft ainda não gera PNG, então o thumbcraft aplica um patch (`patches/gridcraft/`) que adiciona `file.exportPng`. Se o GridCraft ganhar PNG nativo, o patch sai. O repositório guarda por isso o que foi feito, o que foi testado, com quais arquivos de exemplo e os resultados comparativos (tempo, tamanho, taxa de sucesso e qualidade visual) do código original dos *craft contra o LibreOffice e contra as alternativas, para permitir reavaliar a atualização das versões.

## Formatos

| Motor | Lê | Não lê |
|---|---|---|
| WordCraft | .docx, .doc (Word 97-2003), .odt, .rtf, .html, .md, .tex, .txt (e docm, dotx, dotm, dot, WordML) | PDF |
| GridCraft (com patch de PNG) | .xlsx, .xlsm, .xls (só dados, fórmulas e formatos), .ods e .xlsb (só dados), .csv, .tsv | PDF |
| DeckCraft | .pptx, .potx, .ppsx | **.ppt, .odp**, PDF |

PDF não é suportado pelos três motores por enquanto (a avaliação de alternativas está em `docs/PDF.md`, quando existir). Limitações por motor: `docs/PITFALLS.md`.

## Uso

Rascunho do contrato da CLI, **a confirmar com `thumbcraft --help`** (o construtor ainda ajusta a implementação):

```
thumbcraft ENTRADA SAÍDA [-W px] [-H px] [--fit cover|contain]
           [--gravity top|center|...] [--background branco|transparent|#RRGGBB]
           [--trim] [--trim-pad px] [--crop x,y,l,a]   (frações de 0 a 1)
           [--page N] [--render-scale auto|px/pt] [--native-scale S]
thumbcraft --formats | --version | --help
```

Códigos de saída: 0 ok, 2 uso, 3 formato não suportado, 4 arquivo inválido, 5 erro interno.

Exemplo (a confirmar): `thumbcraft relatorio.docx miniatura.png -W 120 -H 120 --fit cover --gravity top`.

## Como compilar

Ver `docs/BUILD.md`. Em resumo, `scripts/build.sh` busca os três projetos upstream em SHAs fixos (`scripts/fetch-upstream.sh`, em `third_party/`), aplica o patch do GridCraft e compila em release com perfil de tamanho. Exige Rust 1.95 ou mais novo (edition 2024) e acesso ao GitHub. Um build limpo leva dezenas de minutos em 2 vCPUs. Versões em uso: `UPSTREAM.md`. Tamanhos: `docs/SIZES.md`.

## Estrutura de pastas

| Caminho | Conteúdo |
|---|---|
| `crates/` | código do thumbcraft |
| `scripts/` | build, busca dos upstream, execução dos corpora, comparação nativa e resumo |
| `patches/gridcraft/` | patch de exportação PNG do GridCraft (ver `docs/PR-GRIDCRAFT.md`) |
| `third_party/` | upstream baixados em SHAs fixos (gerado, não versionado) |
| `corpus/docx/`, `corpus/xlsx/`, `corpus/pptx/` | arquivos de entrada de teste (30, 57 e 46 arquivos) com `MANIFEST.tsv`, mais 48 variantes mutadas de pptx e 16 planilhas da regressão do PDF |
| `baselines/word/`, `baselines/excel/`, `baselines/powerpoint/` | resultados de 2026-10-19: *craft contra LibreOffice contra protótipos, imagens reduzidas, amostras e fontes dos protótipos |
| `baselines/gridcraft-pdf-regression/` | teste de regressão do PDF do GridCraft (325 de 325 casos) |
| `reports/` | relatórios finais dos subagentes da sessão de origem, com índice |
| `docs/` | `HISTORY.md` (narrativa), `DECISIONS.md`, `TESTS.md`, `RESULTS.md`, `CRAFT-FAMILY.md`, `PR-GRIDCRAFT.md`, `UPDATING.md`, `ENVIRONMENT.md`, `PITFALLS.md`, `STATUS.md`, e os de construção (`BUILD.md`, `SIZES.md`, `ARCHITECTURE.md`, `PDF.md`) |

## Como retomar

Para avaliar a viabilidade de atualizar os *craft, siga `docs/UPDATING.md` (playbook passo a passo: buscar os novos SHAs, ver o que mudou, reaplicar patches, rodar os corpora, comparar com `baselines/` e registrar o resultado). Leia antes `docs/HISTORY.md` e `docs/PITFALLS.md`.

## Aviso sobre os dados guardados

O corpus e os baselines são de 2026-10-19 (Linux x86_64, 2 vCPUs compartilhadas, LibreOffice 24.2.7.2, Rust 1.97.0), sobre os commits `117aea31` (WordCraft), `66ce6be4` (GridCraft) e `010941ec` (DeckCraft). Os tempos têm ruído. O repositório não guarda binários, PNGs completos, PDFs golden nem clones dos *craft: apenas hashes, receitas e amostras reduzidas. Tamanhos em MB decimais (1 MB = 1.000.000 bytes) com MiB entre parênteses.

## Licença

Apache License 2.0 (arquivo `LICENSE`). Os projetos *craft em `third_party/` têm as próprias licenças (MIT ou Apache-2.0).
