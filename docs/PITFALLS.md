# Armadilhas e limitações conhecidas

Coisas que enganam quem lê os relatórios ou usa os motores. Dados de 2026-10-19.

## Armadilhas de medição e leitura

1. **MiB chamado de MB.** Vários relatórios antigos citam MiB como MB (mini Word "18,8 MB" é 18,85 MiB; mini PowerPoint "6,4 MB" é 6,36 MiB; `gc-min` "8,00 MB" é 8,39 MB). Convenção do repositório: MB decimais (1 MB = 1.000.000 bytes) com MiB entre parênteses. Valores corretos: mini Word 19.763.984 bytes (19,76 MB, 18,85 MiB), mini PowerPoint 6.673.360 bytes (6,67 MB, 6,36 MiB). Os relatórios 01, 04 e 14 têm nota de correção.
2. **Medições com 2 vCPUs compartilhadas.** Tempos de execução e de build têm ruído e foram medidos uma vez. A comparação entre motores (12 a 20 vezes) é robusta, a diferença entre variantes próximas (por exemplo 129 ms contra 134 ms) não é.
3. **Médias diferentes nos relatórios.** Algumas médias são sobre todos os arquivos (inclui falhas rápidas) e outras só sobre sucessos (DeckCraft 83 ms contra 98 ms; Word 0,105 s contra 0,124 s). `RESULTS.md` traz as duas.
4. **Compilar com `CARGO_TARGET_DIR` compartilhado serializa os builds**, e binários com o mesmo nome se sobrescrevem: copie o original para fora do `target` antes de compilar a versão modificada.
5. **O corpus é sintético.** Nenhum .docx, .pptx nem .xlsx foi feito pelo Word, PowerPoint ou Excel de verdade. Os .ppt e .odp foram gerados pelo LibreOffice (o 35 perdeu formas na reimportação), os .ppsx e .potx são o 03 com o content type trocado, e o SmartArt foi montado à mão. Documentos reais complexos (formas, WordArt, EMF e WMF, campos, OLE) não foram testados.
6. **Teste de mutação fraco.** Nas 48 mutações do DeckCraft, 46 geraram PNG idêntico ao do original; só mostra ausência de panic.
7. **`comparacao-final-tres-colunas.tsv` tem as colunas `ms_pdf-raster` e `exit_pdf-raster` trocadas.** Leia uma como a outra.
8. **O `MANIFEST.tsv` do corpus xlsx virou PNG** nas rodadas A3 e A4 do `gp-min` (58 linhas em vez de 57). Contas de sucesso excluem essa linha.
9. **Contagens diferentes por motor no Excel.** 57 arquivos no LibreOffice e no `gp-min`; 55 no `gc-min` (ignora os 2 .tsv e o manifesto). Os relatórios citam 55, 57 e 58 conforme o caso.
10. **Dimensões diferentes mesmo com a mesma paginação:** LibreOffice 795x1124, `gp-min` 1190x1683 e `gc-min` 1191x1684 (escala 2). A comparação é visual, sem contagem de pixels, exceto o RMSE de 0,6% a 1,5% entre `gp-min` e `gc-min`.

## Armadilhas dos motores

11. **DeckCraft perde o slide 0 e gera a thumbnail do slide errado.** Se o XML do slide 1 está quebrado, ele descarta o slide e renderiza o slide 2. O aviso só sai no stderr (e a lista `lost` de `deckcraft_pptx::import_checked` não é impressa pelo `dc-min`). Tratar como erro quando o slide 0 estiver em `lost`.
12. **`panic=abort` derruba o processo.** Nos builds de tamanho, uma falha interna não é capturada (perde-se o `catch_unwind` do renderizador). Rode um processo por arquivo com timeout.
13. **Texto não latino vira `?` no GridCraft** (limite do PDF: fontes padrão 14 e medição WinAnsi). Árabe, CJK, hindi, grego, russo e emoji saem como `?`. No DeckCraft o emoji sai monocromático e o árabe em outra fonte.
14. **Fontes Microsoft são substituídas por equivalentes.** Calibri e Cambria por Carlito e Caladea no sistema; no WordCraft, Calibri, Arial e Cambria saem em serifa embutida (só Courier fica monoespaçada); no GridCraft, tudo vira Helvetica, Times ou Courier. A métrica muda, e as quebras de linha podem divergir do Word. Sem Carlito e Caladea no sistema, a fidelidade do DeckCraft cai.
15. **Planilhas grandes mostram só a primeira página A4.** Sem zoom: 130 colunas mostram 10 colunas; 10 mil linhas mostram cerca de 48 linhas. O conteúdo útil pode ocupar só o canto da página; para 120x120, use crop ou trim.
16. **O LibreOffice define a página 1 como a primeira página de impressão da primeira aba visível**, em A4 retrato e escala original, respeitando a área de impressão. Não é A1:Z60. O zoom da planilha é ignorado, linhas e colunas ocultas não aparecem, e a primeira aba oculta é pulada. O `gc-min` e o `gp-min` sem zoom reproduzem isso.
17. **O LibreOffice dá exit 0 sem PNG ou com PNG em branco em entradas inválidas.** Arquivo de 0 bytes gera PNG em branco; texto puro com extensão .pptx gera PNG de texto minúsculo; arquivo truncado dá exit 0 sem PNG. Não use o exit code dele para detectar falha.
18. **Texto cortado na célula do GridCraft.** O motor não deixa texto transbordar sobre célula vizinha ocupada ("Centro-Oeste" sai "Centro-Oe"), mesma regra do PDF. Números decimais saem arredondados (`493.693` contra `493.69299` do LibreOffice, causa não investigada).
19. **Fórmulas sem valor em cache** são recalculadas pelo LibreOffice, mas outros motores só leem o cache (casos 04 e 41 do corpus xlsx).
20. **Extensão trocada.** O DeckCraft e o LibreOffice detectam o formato pelo conteúdo. O WordCraft também, exceto texto puro com extensão .docx (falha, caso 25). Um docx com extensão .txt é tratado como texto.
21. **WordCraft: PNG com fundo transparente e em Letter**, não A4 (1224x1584 a escala 2). Achate sobre branco. Só a página 1 sai no `convert`; para outras, `render --page N`.
22. **DeckCraft: escala padrão do CLI é 2, o `dc-min` testado usou 1** (960x540). O thumbcraft renderiza em alta resolução e reduz.
23. **Arquivos de texto, md e tex** não carregam imagens externas relativas.
24. **Slide oculto como primeiro slide** é renderizado normalmente (igual ao LibreOffice). O DeckCraft é alpha (README: cerca de 55% pronto para trabalho real).
25. **Diferenças visuais DeckCraft contra LibreOffice:** gráficos complexos (rosca com 1 série, área com ordem e transparência diferentes), título de gráfico sem negrito, tabela com cabeçalho em negrito só no DeckCraft, gradiente radial com brilho diferente. SmartArt sem drawing sai em branco nos dois.
26. **Chave do arquivo no Word:** o PNG do `wc-min` sai com o nome `ARQUIVO.docx.png` nos resultados originais, e o LibreOffice como `ARQUIVO.png`.

## Armadilhas de dependências e build

27. **`craft-pdf` com `raster` só no `main` do craft-libs** (a partir do commit `c07b3fa`). O `c3487320`, pinado por WordCraft e DeckCraft, só tem o writer. Subir a revisão não foi testado contra o `krilla`.
28. **Patches do hayro em `vendor/` do craft-libs.** O `[patch.crates-io]` de uma dependência é ignorado: o crate raiz precisa repeti-lo (`hayro`, `hayro-interpret`, `hayro-syntax`, `hayro-jbig2`, no mesmo rev).
29. **Duas versões de `vello_cpu` no GridCraft com GUI** (0.1.0 do epaint e 0.2.0 do raster); só no binário sem a GUI elas deduplicam.
30. **Motor com `file.exportPng` registrado já puxa o backend PNG inteiro** (1,95 MB), mesmo sem chamá-lo, até existir uma feature para isso.
31. **Fontes do GridCraft vieram do npm** porque o GitHub do `craft-fonts` deu 403 (ver `ENVIRONMENT.md`). O `AGENTS.md` do GridCraft manda manter fontes fora do repositório.
32. **Build limpo dura dezenas de minutos** (WordCraft 7 a 8 min, DeckCraft 9 a 13 min, `gp-min` 5 a 7 min, combinado até 14 min, em 2 vCPUs compartilhadas). Rust mínimo: 1.95 (edition 2024).
33. **Versão do Rust e `--locked`.** Os `Cargo.lock` dos upstream são respeitados com `--locked`; o lock do `gp-min` foi copiado do GridCraft e só ganhou o que faltava.

## Limitações por motor

### WordCraft

- Lê .docx, .doc 97 (só leitura), .odt, .rtf, .html, .md, .tex, .txt e outros (docm, dotx, dotm, dot, WordML, json). Não lê PDF.
- .doc complexo: a tabela perdeu as bordas no teste (caso 09). Imagens, WMF, EMF e campos não foram testados.
- Só página 1 no `convert`. Fonte Microsoft substituída. Markdown com extensão trocada vira texto bruto.
- Marcas de edição (`Mark`, `MarkText`) só aparecem no PNG, não no PDF.

### GridCraft

- Sem PNG nativo (só com o patch). Lê .xlsx e .xlsm (completo), .xls (sem fontes, fundos, bordas nem gráficos), .xlsb e .ods (só dados; fórmulas com o último valor salvo, formatação omitida), .csv e .tsv. Latin-1 sem detecção de codificação.
- Texto não latino vira `?`; só fontes padrão 14 (Helvetica, Times, Courier); itálico usa a largura do reto e fora do ASCII há aproximação nas métricas.
- Sem recorte, sem JPEG e sem emoji no PNG do patch. Máximo de 65535 px por lado e 64 milhões de pixels.
- Painéis congelados, sparklines, rotação de texto, RTL, comentários e hiperlinks: só parte foi conferida visualmente no `gp-min`; o `gc-min` foi conferido em bordas, gráficos, texto rotacionado, imagens, mesclagem, formatação condicional, quebra, transbordo, primeira aba oculta e área de impressão. O caso 35 (comentários e hiperlinks) deu PNG quase vazio no `gp-min` e não foi investigado.
- Não confirmado: o comportamento com .xlsb (nenhuma ferramenta local grava XLSB).

### DeckCraft

- Lê .pptx, .potx e .ppsx (por conteúdo), `.deckcraft` e (só na CLI completa) .txt e .md. **Não lê .ppt nem .odp.**
- Alpha. Gráficos complexos divergem do LibreOffice. SmartArt só com drawing de fallback. Emoji monocromático. Texto vertical e rotacionado funcionam no corpus.
- Procura fontes do sistema (o arquivo com árabe, CJK e emoji levou 970 ms por causa disso).

### Todos

- Windows: o `git` global com `core.autocrlf=true` converte os clones de `third_party/` para CRLF e os patches não
  aplicam; o `scripts/fetch-upstream.sh` fixa `autocrlf=false` sozinho. O hash da árvore do GridCraft também depende do
  bit de execução, que o git do Windows não grava; o script o copia do commit (`docs/UPSTREAM-CHANGES.md`).

- PDF como entrada: nenhum dos três renderiza PDF (ver `PDF.md`).
- Linux x86_64 apenas nos testes.
