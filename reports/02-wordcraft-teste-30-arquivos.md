# WordCraft: teste de thumbnails com 30 arquivos contra o LibreOffice

> Arquivo de relatório. Origem: subagente `a9c8d6827203c05f5`, turno 2 (de 2). Sessão de 2026-10-19.
> Tema: Teste do `wc-min` em 30 arquivos de formatos variados e comparação com o LibreOffice.
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

## Relatório: thumbnails com `wc-min` em 30 arquivos de teste

**1. Formatos de entrada (README e `crates/engine/src/io.rs`, `io_ext.rs`)**
- O README declara: .docx, .doc (Word 97-2003), .odt, .rtf, .html, .md, .tex e .txt.
- O código também aceita docm, dotx, dotm, dot, xml (WordML e flat OPC), json, htm e xhtml.
- O README apresenta .doc como "import" (só leitura). O código confirma: `io_ext.rs` tem só leitura para `.doc`/`.dot`, e a escrita em .doc não existe.
- Os demais formatos têm leitura e escrita, e o export de PNG é separado.
- A extensão não decide tudo: se a assinatura contradiz a extensão, o `open_by_content` detecta ZIP, OLE, `{\rtf`, WordML, página web ou texto.

**2. Resultados por arquivo (`wc-min`, página 1, escala 2)**

| Arquivo (caso) | Resultado | Tempo | Observação |
|---|---|---|---|
| 01 docx texto com acentos | ok | 0,19 s | |
| 02 docx títulos e listas | ok | 0,12 s | |
| 03 docx tabelas com e sem borda | ok | 0,09 s | bordas corretas |
| 04 docx imagens PNG e JPEG | ok | 0,15 s | |
| 05 docx multipágina (40 capítulos) | ok | 0,23 s | só a página 1 |
| 06 docx fontes Calibri, Times, Arial, Courier, Cambria | degradado | 0,10 s | Calibri, Arial e Cambria saem em serifa embutida; só Courier é mono |
| 07 docx negrito, itálico, cores | ok | 0,08 s | |
| 08 docx cabeçalho e rodapé | ok | 0,20 s | |
| 09 doc 97 com imagem e tabela | degradado | 0,15 s | tabela perdeu as bordas (no LibreOffice a tabela tem bordas) |
| 10 doc 97 títulos e listas | ok | 0,14 s | |
| 11 odt | ok | 0,12 s | |
| 12 rtf | ok | 0,11 s | |
| 13 html | ok | 0,10 s | imagem embutida (data URI) renderizada |
| 14 md | degradado | 0,08 s | imagem ausente porque o arquivo referenciado não estava ao lado do .md (limitação do meu teste) |
| 15 tex | degradado | 0,08 s | imagem vira texto "[Picture: img.png]" |
| 16 txt simples | ok | 0,06 s | |
| 17 txt longo (10+ páginas) | ok | 0,17 s | |
| 18 odt multipágina | ok | 0,12 s | |
| 19 md multipágina | ok | 0,13 s | |
| 20 doc 97 multipágina | ok | 0,25 s | |
| 21 docx vazio (0 bytes) | falhou, exit 1 | 0,01 s | mensagem "not a valid zip package" |
| 22 txt vazio | ok | 0,05 s | página em branco |
| 23 docx truncado | falhou, exit 1 | 0,01 s | erro limpo |
| 24 doc truncado | falhou, exit 1 | 0,01 s | erro limpo ("Malformed FAT") |
| 25 texto puro com extensão .docx | falhou, exit 1 | 0,01 s | o fallback por conteúdo não rodou; o `wordcraft-cli` falha igual |
| 26 docx com extensão .txt | degradado | 0,11 s | tratado como texto, sai lixo binário legível |
| 27 docx com extensão .doc | ok | 0,08 s | detectou ZIP pelo conteúdo |
| 28 md com extensão .pdf | degradado | 0,10 s | renderizado como texto bruto (markdown não interpretado) |
| 29 html com extensão .doc | ok | 0,11 s | detectado como página web |
| 30 lixo binário .docx | falhou, exit 1 | 0,01 s | erro limpo |

**3. Métricas**
- Taxa de sucesso técnica: 25 de 30 (83%).
- As 5 falhas são todas entradas inválidas (vazio, truncado, lixo, texto com extensão .docx) e saíram com exit 1 e mensagem de erro limpa.
- Não houve panic, travamento nem timeout (limite de 30 s por arquivo).
- Tempo médio nos sucessos: 0,12 s. Máximo: 0,245 s (doc multipágina).
- Qualidade: 18 ok, 7 degradados (06, 09, 14, 15, 26, 28 e o caso 22 em branco, que é esperado para arquivo vazio) e 5 falharam.
- Para comparação, o LibreOffice (`soffice --headless --convert-to png`, processo novo por arquivo) levou 1,48 s em média, cerca de 12 vezes mais lento. Ele gerou 25 PNGs de 30.
- Comparei lado a lado com o LibreOffice nos casos 03, 06, 08, 09, 11 e 12:
  - O layout de docx, odt e rtf ficou muito próximo.
  - O único desvio de layout relevante foi a tabela do .doc sem bordas (caso 09).
  - Nas fontes, o LibreOffice neste sandbox também substituiu as fontes, então o caso 06 não é uma diferença entre os dois.

**4. Ressalvas para uso em thumbnail**
- Só a página 1 é gerada (`wc-min in out.png [página]`). O PNG sai em Letter (1224x1584) com fundo transparente, e eu o achatei em branco nas miniaturas. Em A4 depende da região ou do documento.
- Fontes Microsoft (Calibri, Arial e outras) não são embutidas. Quando não estão no sistema, caem em fontes OFL (Source Serif, Source Sans, Inter), então a métrica muda e as quebras de linha podem divergir do Word.
- A extensão errada é tolerada na maioria dos casos (27 e 29), mas texto puro com extensão .docx falha (caso 25).
- Arquivos de texto, md e tex não carregam imagens externas (referência relativa).
- Meu teste cobre documentos simples gerados por python-docx, pandoc e LibreOffice. Não testei documentos reais complexos (formas, WordArt, EMF/WMF, campos, OLE).

**Caminhos**
- Montagem (30 miniaturas com nome; as falhas aparecem como tiles vermelhos "FALHOU"): `S/thumbs-test/contact_sheet.png`
- PNGs individuais em tamanho cheio (25): `S/thumbs-test/png/`
- Miniaturas e tiles de falha: `S/thumbs-test/thumbs/`
- Comparação com o LibreOffice (6 casos): `S/thumbs-test/compare.png`
- Entradas de teste (30): `S/thumbs-test/in/`
- Tempos e códigos de saída: `results.tsv` na mesma pasta `thumbs-test/`

Nota para a memória do coordenador: o build pedido na tarefa anterior permanece em `S/target-small/release/wc-min` (18,8 MB).
