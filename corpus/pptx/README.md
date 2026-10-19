# PowerPoint corpus (DeckCraft and similar)

A set of 46 input files used to test thumbnail generation (slide 1 as PNG) with the DeckCraft engine and to compare it with LibreOffice. Assembled on 2026-10-19.

- Total size: 5.4 decimal MB (46 input files 3.3 MB, plus `mutacao-48/` 2.2 MB).
- Index: `MANIFEST.tsv` (file, format, case, category, how it was generated, DeckCraft and LibreOffice results).
- Per-file results: `../../baselines/powerpoint/`.
- How each file was generated: `../../docs/TESTS.md`, section "Corpus PowerPoint".

## Composition

| Category | Files |
|---|---|
| Valid .pptx (01 to 32, including 12b) | 33 |
| Alternative OOXML formats (.ppsx and .potx, 33 and 34) | 2 |
| Formats DeckCraft does not read (.ppt and .odp, 35 to 39) | 5 |
| Invalid (0 bytes, truncated, corrupted bytes, plain text, broken XML, swapped extension, 40 to 46) | 7 |

Number 24 does not exist: the vertical text case and the rotated text case were merged into 23.

## Read with care

- The .ppsx and .potx files (33 and 34) are 03 with the content type swapped, not files made by PowerPoint.
- The .ppt and .odp files (35 to 39) were produced by LibreOffice itself from pptx. File 35 lost shapes when LibreOffice re-imported it, so it proves nothing about a .ppt made by PowerPoint 97.
- The SmartArt files 12 and 12b were assembled by hand, not taken from a real PowerPoint.
- The corpus has no .pptx produced by a real PowerPoint.

## mutacao-48/

48 mutated variants of 6 valid pptx files (09, 12, 18, 05, 03 and 23), 8 per seed, used in DeckCraft's robustness test. The modes, in cyclic order by mutation index k (k mod 4): 0 overwrites 5 random bytes, 1 truncates the file, 2 removes a stretch of 1 to 49 bytes, 3 replaces `val="` and `cx="` values with absurd numbers. In each mutant a single XML file of the package (slide, chart, diagram, master or layout) was changed. Random generator seed: 7. Per-file result in `mutacao-48/RESULTADOS.tsv`.
