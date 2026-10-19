# Word corpus (WordCraft and similar)

A set of 30 input files used to test thumbnail generation (page 1 as PNG) with the WordCraft engine and to compare it with LibreOffice. Assembled on 2026-10-19.

- Total size: 836 KB (30 files, plus `fontes-geracao/`).
- Index: `MANIFEST.tsv` (file, case, category, how it was generated, WordCraft result, whether LibreOffice produced a PNG).
- Per-file results: `../../baselines/word/`.
- How each file was generated: `../../docs/TESTS.md`, section "Corpus Word".

## Composition

| Category | Files | Range |
|---|---|---|
| Valid .docx | 8 | 01 to 08 |
| Alternative formats (.doc 97, .odt, .rtf, .html, .md, .tex, .txt) | 12 | 09 to 20 |
| Invalid (empty, truncated, garbage, swapped extension) | 10 | 21 to 30 |

## Notes

- Files 25 to 29 are byte-for-byte copies of other corpus files with the extension swapped (see the `geracao` column of the manifest).
- All the valid documents are synthetic and simple (python-docx, pandoc and LibreOffice). There is no real Word document with shapes, WordArt, EMF/WMF, fields or OLE. This is a known limitation of the corpus.
- The corpus was not generated with committed scripts: the scripts were Python and, by project rule, are not versioned. The `fontes-geracao/` folder keeps the non-Python inputs (source Markdown and images) to rebuild the files with pandoc and LibreOffice.
- The image in `14_md_rico.md` references `img.png` by a relative path. In the original test the file was not next to the .md, so the image did not show up (a limitation of the test, not of the engine). To repeat it with the image, copy `fontes-geracao/img.png` to the same folder.
