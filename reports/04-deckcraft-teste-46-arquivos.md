# DeckCraft: teste de thumbnails com 46 arquivos contra o LibreOffice

> Arquivo de relatório. Origem: subagente `a50f015b868bc6891`, turno final (o turno intermediário curto foi ignorado). Sessão de 2026-10-19.
> Tema: Binário mínimo `dc-min`, 46 arquivos, comparação com o LibreOffice, teste de mutação de 48 variantes.
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

Relatório do irmão de PowerPoint do wordcraft.

## Repositório
- **DeckCraft**: https://github.com/storytold/deckcraft (versão 0.8.0, alpha, MIT ou Apache-2.0).
- Não está linkado no README do wordcraft. O README do próprio deckcraft o lista entre os "Crafting Apps" como "Presentations and slide shows".
- O nome `slidecraft` redireciona para o mesmo repositório (mesmo hash de commit).
- O README confirma renderização de .pptx e exportação PNG/JPEG pela CLI `deckcraft-cli`.
- Commit testado: `010941ec`.

## Formatos de entrada e comando de PNG
Pelo código de abertura (`crates/engine/src/cmd/file.rs`):
- **Lê:** .pptx, .potx e .ppsx (reconhecidos pelo conteúdo, não pela extensão), o formato nativo `.deckcraft`, e .txt/.md (viram slides, só na CLI completa).
- **Não lê:** .ppt legado e .odp. Há só detecção de arquivo criptografado, que gera mensagem de erro.
- **Exportar PNG:** `deckcraft-cli render IN.pptx OUT.png --slide N --scale S`. O slide é indexado a partir de 0, o padrão é o slide 0, e a escala padrão é 2 (px por pt). `--all` gera uma pasta com um PNG por slide.
- **Binário mínimo:** `dc-min IN OUT.png [ESCALA] [SLIDE]`, escrito por mim, depende só de `deckcraft-pptx`, `deckcraft-format`, `deckcraft-render` e `deckcraft-fonts`. Fica em `S/pptx-test/dc-min/`. Ele gera PNG byte a byte idêntico ao da CLI completa nos 36 arquivos válidos comparados.

## Build
- Rust exigido: 1.95 (edition 2024). Usei o 1.97.0 já instalado.
- Padrão da CLI: `cargo build --release --locked -p deckcraft-cli`. O perfil do repo é opt-level 3, lto thin e codegen-units 16.
- Otimizado e mínimo: as mesmas variáveis `CARGO_PROFILE_RELEASE_{OPT_LEVEL=z,LTO=fat,CODEGEN_UNITS=1,PANIC=abort,STRIP=symbols}`. A CLI otimizada usa o `-p deckcraft-cli` acima. O mínimo é `cargo build --release` dentro de `dc-min/`, com `CARGO_TARGET_DIR` próprio.
- Não precisou de ALSA nem de rede além do GitHub (a dependência git `craft-libs` baixou normalmente).
- Tempos de build com 2 núcleos: CLI padrão 753 s (12,5 min), CLI otimizada 770 s, mínimo 523 s. Estes tempos estão inflados porque construí dois binários ao mesmo tempo e o outro agente também estava compilando na mesma máquina.

| Binário | Bytes | MB |
|---|---|---|
| CLI padrão (release do repo) | 35.296.984 | 33,7 |
| CLI otimizada | 14.504.104 | 13,8 |
| **dc-min (mínimo)** | **6.673.360** | **6,4** |
| LibreOffice instalado (14 pacotes dpkg, para comparação) | cerca de 316 MB | 316 |

[Nota de correção do arquivista: a coluna "MB" desta tabela está em MiB. Em MB decimais: 35.296.984 bytes = 35,30 MB (33,66 MiB); 14.504.104 = 14,50 MB (13,83 MiB); 6.673.360 = 6,67 MB (6,36 MiB). O "316 MB" do LibreOffice instalado tem unidade não confirmada pelo agente (provavelmente MiB, vindo de `du` ou `dpkg`), A razão "cerca de 47 vezes menor" bate com 316 / 6,67.]

O mínimo é cerca de 47 vezes menor que o LibreOffice instalado.

## Resultados (46 arquivos, timeout 30 s, 1 processo por arquivo)
| | Rust (dc-min) | LibreOffice 24.2.7 |
|---|---|---|
| PNG gerado | 38 de 46 | 45 de 46 |
| Sucesso útil (ok ou degradado) | 38 | 43 |
| Tempo médio / máximo | 83 ms / 970 ms | 1.792 ms / 2.977 ms |
| ok / degradado / falhou | 34 / 4 / 8 | 40 / 3 / 3 |

- O LibreOffice é cerca de 20 vezes mais lento. Em 45 dos 46 arquivos o Rust levou menos de 220 ms. O pico de 970 ms foi o arquivo com árabe, CJK e emoji (varredura de fontes do sistema).
- "Falhou" do Rust inclui 3 arquivos inválidos que ele rejeita corretamente (exit 1 com mensagem clara). O LibreOffice deu exit 0 com PNG em branco ou de texto minúsculo para o arquivo de 0 bytes e para o texto puro, ou seja, falso sucesso.
- Teste extra de robustez: 48 variantes mutadas de pptx válidos (XML alterado, truncado, números absurdos) deram 48 PNGs, sem panic nem travamento.

## Tabela por arquivo
Tempo em ms (Rust / LibreOffice). Classificação: ok, deg (degradado) ou F (falhou).

| Arquivo | Formato | Caso | Rust | LO | ms |
|---|---|---|---|---|---|
| 01 | pptx | título com acentos | ok | ok | 60 / 1700 |
| 02 | pptx | bullets e níveis | ok | ok | 101 / 1689 |
| 03 | pptx | imagens PNG e JPEG | ok | ok | 90 / 1891 |
| 04 | pptx | formas, setas e linhas | ok | ok | 150 / 2080 |
| 05 | pptx | gradientes | ok | ok | 124 / 1716 |
| 06 | pptx | sombra e transparência | ok | ok | 127 / 1613 |
| 07 | pptx | tabela com bordas | ok | ok | 40 / 1479 |
| 08 | pptx | tabela sem bordas | ok | ok | 37 / 1627 |
| 09 | pptx | gráfico de barras | ok | ok | 50 / 1652 |
| 10 | pptx | gráfico de linhas | ok | ok | 43 / 1543 |
| 11 | pptx | gráfico de pizza | ok | ok | 46 / 1628 |
| 12 | pptx | SmartArt com drawing | ok | ok | 35 / 1555 |
| 12b | pptx | SmartArt sem drawing | deg | deg | 40 / 1706 |
| 13 | pptx | texto formatado | ok | ok | 54 / 1846 |
| 14 | pptx | fontes | ok | ok | 67 / 1625 |
| 15 | pptx | fundo sólido | ok | ok | 31 / 1550 |
| 16 | pptx | fundo gradiente | ok | ok | 51 / 1511 |
| 17 | pptx | fundo imagem | ok | ok | 77 / 1494 |
| 18 | pptx | mestre e layout com logo | ok | ok | 50 / 1773 |
| 19 | pptx | notas do orador | ok | ok | 45 / 1566 |
| 20 | pptx | transições e animações | ok | ok | 36 / 1594 |
| 21 | pptx | 60 slides | ok | ok | 60 / 1922 |
| 22 | pptx | texto com overflow | ok | ok | 118 / 2107 |
| 23 | pptx | vertical e rotacionado | ok | ok | 101 / 1842 |
| 25 | pptx | proporção 4:3 | ok | ok | 39 / 2119 |
| 26 | pptx | proporção 16:9 | ok | ok | 36 / 2049 |
| 27 | pptx | vídeo (placeholder) | ok | ok | 124 / 1838 |
| 28 | pptx | hiperlinks | ok | ok | 55 / 2033 |
| 29 | pptx | primeiro slide oculto | ok | ok | 41 / 1860 |
| 30 | pptx | RTL, CJK e emoji | deg | ok | 970 / 2134 |
| 31 | pptx | imagem em tela cheia e recorte | ok | ok | 210 / 1904 |
| 32 | pptx | vários gráficos | deg | ok | 74 / 1679 |
| 33 | ppsx | slideshow | ok | ok | 82 / 1443 |
| 34 | potx | modelo | ok | ok | 82 / 1480 |
| 35 | ppt | legado com formas | F | deg | 16 / 1426 |
| 36 | ppt | legado com imagem | F | ok | 11 / 1444 |
| 37 | odp | formas com gradiente | F | ok | 16 / 2263 |
| 38 | odp | gráfico | F | ok | 12 / 1837 |
| 39 | ppt | legado com gráfico | F | ok | 13 / 1507 |
| 40 | pptx | vazio (0 bytes) | F | F | 11 / 1663 |
| 41 | pptx | truncado | F | F | 10 / 1623 |
| 42 | pptx | bytes corrompidos | ok | ok | 93 / 1703 |
| 43 | pptx | texto puro | F | F | 12 / 1596 |
| 44 | pptx | XML do slide 1 quebrado | deg | deg | 41 / 2977 |
| 45 | ppt | pptx com extensão .ppt | ok | ok | 123 / 2864 |
| 46 | docx | pptx com extensão .docx | ok | ok | 119 / 2292 |

O número 24 não existe: o caso de texto vertical e o de rotacionado ficaram no mesmo arquivo (23).

## Principais diferenças visuais
Em quase todos os casos válidos o resultado é muito próximo, inclusive gráficos nativos, gradientes (linear, multi-stop e radial), sombras, transparência, fundo com imagem, mestre com logotipo, recorte de imagem e fonte Calibri/Cambria (substituídas por Carlito/Caladea). Diferenças:
- **Gráficos (arquivo 32):** o Rust desenha a rosca com uma série só, e a área com ordem e transparência diferentes do LibreOffice. Gráficos simples (barras, linhas, pizza) saíram corretos.
- **Gráficos simples:** o Rust usa título sem negrito e eixos mais leves.
- **Texto exótico (arquivo 30):** o emoji sai monocromático e o árabe em outra fonte e tamanho.
- **Tabela (arquivo 07):** o cabeçalho sai em negrito só no Rust.
- **SmartArt:** só com drawing (fallback de desenho que o PowerPoint grava) funciona. Sem ele, os dois deixam o diagrama em branco. O SmartArt foi montado por mim à mão, não veio de um PowerPoint real.

## Falhas e causas
- **.ppt e .odp:** o Rust não suporta (erro "not a presentation deckcraft can read"). O LibreOffice abre. Os .ppt e .odp foram gerados pelo próprio LibreOffice, então o 35 perdeu formas na reimportação e não prova nada sobre .ppt do PowerPoint 97.
- **0 bytes, truncado e texto puro:** o Rust sai com código 1 e mensagem clara. O LibreOffice dá exit 0 com PNG em branco no de 0 bytes e com texto minúsculo no texto puro (o truncado ele rejeita sem gerar PNG).
- **Slide 1 com XML quebrado (arquivo 44):** o Rust descarta o slide danificado e renderiza o slide 2, o que gera uma miniatura do slide errado. O aviso só sai no stderr e o `dc-min` atual o ignora. O LibreOffice mostra o slide 1 sem o corpo.

## Ressalvas para uso como thumbnail
1. Só funciona para OOXML (.pptx, .potx, .ppsx). Para .ppt e .odp é preciso manter o LibreOffice como fallback.
2. No `dc-min`, imprimir a lista `lost` devolvida por `deckcraft_pptx::import_checked` e tratar como erro quando o slide 0 estiver nela. Isso evita thumbnail do slide errado.
3. Fontes vêm das fontes do sistema (aqui havia Carlito e Caladea, equivalentes métricos de Calibri e Cambria). Sem elas, a fidelidade de texto cai. O README sugere clonar `craft-fonts` ao lado do repo.
4. Com `panic=abort`, uma falha interna derruba o processo. Isso é seguro se você rodar um processo por arquivo com timeout, mas perde o `catch_unwind` interno do renderizador.
5. O slide oculto como primeiro slide é renderizado normalmente, igual ao LibreOffice. O repositório é alpha (README: cerca de 55% pronto para trabalho real), então vale testar com arquivos reais do usuário.
6. Medições de tempo foram em máquina de 2 núcleos, compartilhada com outro agente fazendo builds, e com cache de disco quente.

## Arquivos
- Contact sheet do Rust: `S/pptx-test/thumbs/contact_sheet_rust.png`
- Comparação Rust x LibreOffice (12 casos): `S/pptx-test/thumbs/comparacao_rust_vs_libreoffice.png`
- PNGs do Rust: `S/pptx-test/out-rust/`
- PNGs do LibreOffice: `S/pptx-test/out-lo/`
- Resultados com classificação: `S/pptx-test/results.tsv` (dados brutos em `results-raw.tsv`)
- Arquivos de entrada: `S/pptx-test/in/`
- Binário mínimo: `S/pptx-test/target-min/release/dc-min`

Para a memória do usuário: o irmão de PowerPoint é o DeckCraft (storytold/deckcraft). O binário mínimo de thumbnail tem 6,4 MB e é cerca de 20 vezes mais rápido que o LibreOffice, mas não lê .ppt nem .odp.

[Nota de correção do arquivista: 6,4 MB é MiB. O mini PowerPoint tem 6,67 MB decimais (6,36 MiB).]
