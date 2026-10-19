# Corpora com o binário atual (PDF, craft-libs único, glifo no skrifa 0.47)

Rodada de 2026-10-19 com `scripts/run-corpus.sh`, comparável à de `../thumbcraft-run-2026-10-19/` (mesmas opções:
`-W 120 -H 120 --fit cover` e `-W 480 -H 480 --fit contain`), mais o corpus de PDF (`-W 120 -H 120 --fit cover`).
`windows/`: Windows 11 x64, binário de release. `linux/`: Docker `rust:1.99` (Debian 13, glibc 2.41) no WSL2.

Contra a rodada original: mesmos códigos de saída em todos os arquivos, mesmas dimensões, taxas iguais (Word 25 de 30,
PowerPoint 37 de 41 mais 5 não suportados, Excel 54 de 57), e os PNGs do Excel idênticos byte a byte (57 de 57) nas duas
plataformas. Word e PowerPoint diferem em bytes porque as fontes do sistema são outras (a rodada original foi em outra
máquina Linux); a troca do `glifo` 0.44 pelo 0.47 foi testada à parte e deu PNGs idênticos (25 de 25 Word, 37 de 37
PowerPoint, Windows, binários de debug com e sem o patch).
