#!/usr/bin/env bash
# Gera o corpus de PDFs do benchmark. Reprodutível: só depende de ferramentas locais.
# Uso: gen-corpus.sh DIR_SAIDA
# Variáveis: WORD_DIR, PPT_DIR, XLS_DIR (corpora de origem, para a conversão pelo LibreOffice).
# Arquivos em DIR_SAIDA/ vão para o git; os de DIR_SAIDA/big/ não (são grandes, só o recipe fica).
set -euo pipefail
OUT=${1:?uso: gen-corpus.sh DIR_SAIDA}
S=${S:-/tmp/claude-0/-home-claude/37e359fe-23e0-5bd2-965b-70005e330799/scratchpad}
WORD_DIR=${WORD_DIR:-$S/thumbs-test/in}
PPT_DIR=${PPT_DIR:-$S/pptx-test/in}
XLS_DIR=${XLS_DIR:-$S/grid-png/c-corpus/in}
mkdir -p "$OUT" "$OUT/big"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
# qpdf devolve 3 quando só houve avisos (reconstrução de xref); trata como sucesso.
qpdf() { local r=0; command qpdf "$@" || r=$?; [ $r -eq 3 ] && return 0; return $r; }

# ---------- 1. Conversões pelo LibreOffice (um único processo soffice por lote) ----------
mkdir -p "$T/lo" "$T/lo_out"
for f in "$WORD_DIR"/01_docx_simples_acentos.docx "$WORD_DIR"/03_docx_tabelas.docx \
         "$WORD_DIR"/04_docx_imagens_png_jpeg.docx "$WORD_DIR"/05_docx_multipagina.docx \
         "$WORD_DIR"/06_docx_fontes.docx "$WORD_DIR"/07_docx_estilos_cores.docx \
         "$WORD_DIR"/17_txt_longo_multipagina.txt \
         "$XLS_DIR"/13-condicional-escala-cor.xlsx "$XLS_DIR"/16-grafico-barras.xlsx \
         "$XLS_DIR"/18-grafico-pizza.xlsx "$XLS_DIR"/23-planilha-larga-130-colunas.xlsx \
         "$XLS_DIR"/31-texto-nao-latino-emoji.xlsx "$XLS_DIR"/34-painel-composto.xlsx \
         "$PPT_DIR"/05_formas_gradiente.pptx "$PPT_DIR"/06_sombra_transparencia.pptx \
         "$PPT_DIR"/09_grafico_barras.pptx "$PPT_DIR"/16_fundo_gradiente.pptx \
         "$PPT_DIR"/17_fundo_imagem.pptx "$PPT_DIR"/30_rtl_cjk_emoji.pptx \
         "$PPT_DIR"/25_proporcao_4x3.pptx "$PPT_DIR"/26_proporcao_16x9.pptx \
         "$PPT_DIR"/21_apresentacao_60_slides.pptx; do
  cp "$f" "$T/lo/"
done
# Texto não latino (árabe, CJK, emoji) escrito em HTML e convertido.
cat > "$T/lo/nao_latino.html" <<'HTML'
<html><head><meta charset="utf-8"></head><body style="font-family:'Noto Sans'">
<h1>Texto não latino</h1>
<p dir="rtl" style="font-family:'Noto Naskh Arabic','DejaVu Sans'">مرحبا بالعالم، هذا اختبار لعرض النص العربي في الصورة المصغرة.</p>
<p style="font-family:'Noto Sans CJK SC'">你好，世界。这是一个中文缩略图渲染测试。</p>
<p style="font-family:'Noto Sans CJK JP'">こんにちは世界。日本語のサムネイルテストです。</p>
<p style="font-family:'Noto Sans CJK KR'">안녕하세요 세계. 한국어 썸네일 테스트입니다.</p>
<p style="font-family:'Noto Color Emoji'">Emoji: 😀 🚀 📄 🎉 ✅</p>
<p>Acentos: ação, coração, órgão, ÀÉÎÕÜ, çñ</p>
</body></html>
HTML
(cd "$T/lo" && soffice --headless --convert-to pdf --outdir "$T/lo_out" ./* >/dev/null 2>&1) || true
(cd "$T/lo" && soffice --headless --convert-to pdf:writer_web_pdf_Export --outdir "$T/lo_out" nao_latino.html >/dev/null 2>&1) || true

i=0
mv_lo() { # origem sem extensão -> nome final
  local src="$T/lo_out/$1.pdf" dst="$OUT/$2.pdf"
  [ -f "$src" ] && cp "$src" "$dst" || echo "AVISO: faltou $1" >&2
}
mv_lo 01_docx_simples_acentos   10_word_simples_acentos
mv_lo 03_docx_tabelas           11_word_tabelas
mv_lo 04_docx_imagens_png_jpeg  12_word_imagens
mv_lo 05_docx_multipagina       13_word_multipagina
mv_lo 06_docx_fontes            14_word_fontes
mv_lo 07_docx_estilos_cores     15_word_estilos_cores
mv_lo 13-condicional-escala-cor 20_excel_condicional_cor
mv_lo 16-grafico-barras         21_excel_grafico_barras
mv_lo 18-grafico-pizza          22_excel_grafico_pizza
mv_lo 23-planilha-larga-130-colunas 23_excel_planilha_larga
mv_lo 31-texto-nao-latino-emoji 24_excel_nao_latino_emoji
mv_lo 34-painel-composto        25_excel_painel_composto
mv_lo 05_formas_gradiente       30_ppt_formas_gradiente
mv_lo 06_sombra_transparencia   31_ppt_sombra_transparencia
mv_lo 09_grafico_barras         32_ppt_grafico_barras
mv_lo 16_fundo_gradiente        33_ppt_fundo_gradiente
mv_lo 17_fundo_imagem           34_ppt_fundo_imagem
mv_lo 30_rtl_cjk_emoji          35_ppt_rtl_cjk_emoji
mv_lo 25_proporcao_4x3          36_ppt_4x3
mv_lo 26_proporcao_16x9         37_ppt_16x9_paisagem
mv_lo 21_apresentacao_60_slides 38_ppt_60_slides
mv_lo nao_latino                40_texto_nao_latino_arabe_cjk_emoji

# ---------- 2. PDFs escritos à mão (qpdf reconstrói a tabela xref) ----------
fix() { qpdf "$1" "$2" 2>/dev/null || cp "$1" "$2"; } # qpdf recupera xref imprecisa

# Fontes não embutidas: base 14 e TrueType citada por nome (Arial, Calibri) sem programa de fonte.
cat > "$T/nf.pdf" <<'PDF'
%PDF-1.4
1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj
2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj
3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 595 842]/Contents 4 0 R/Resources<</Font<</F1 5 0 R/F2 6 0 R/F3 7 0 R/F4 8 0 R/F5 9 0 R>>>>>>endobj
4 0 obj<</Length 0>>stream
BT /F1 22 Tf 50 780 Td (Helvetica base 14: a\347\343o, cora\347\343o, \311\340\365) Tj ET
BT /F2 22 Tf 50 740 Td (Times-Roman base 14: texto serifado) Tj ET
BT /F3 22 Tf 50 700 Td (Courier base 14: monoespacado 0123) Tj ET
BT /F4 22 Tf 50 660 Td (Arial n\343o embutida: fonte do Windows) Tj ET
BT /F5 22 Tf 50 620 Td (Calibri n\343o embutida: fonte do Office) Tj ET
endstream
endobj
5 0 obj<</Type/Font/Subtype/Type1/BaseFont/Helvetica/Encoding/WinAnsiEncoding>>endobj
6 0 obj<</Type/Font/Subtype/Type1/BaseFont/Times-Roman/Encoding/WinAnsiEncoding>>endobj
7 0 obj<</Type/Font/Subtype/Type1/BaseFont/Courier/Encoding/WinAnsiEncoding>>endobj
8 0 obj<</Type/Font/Subtype/TrueType/BaseFont/Arial/Encoding/WinAnsiEncoding/FirstChar 32/LastChar 255>>endobj
9 0 obj<</Type/Font/Subtype/TrueType/BaseFont/Calibri/Encoding/WinAnsiEncoding>>endobj
trailer<</Root 1 0 R>>
PDF
fix "$T/nf.pdf" "$OUT/41_fontes_nao_embutidas.pdf"

# Uma página vazia (sem conteúdo).
cat > "$T/empty.pdf" <<'PDF'
%PDF-1.4
1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj
2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj
3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 595 842]>>endobj
trailer<</Root 1 0 R>>
PDF
fix "$T/empty.pdf" "$OUT/42_pagina_vazia.pdf"

# Zero páginas (Count 0).
cat > "$T/zero.pdf" <<'PDF'
%PDF-1.4
1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj
2 0 obj<</Type/Pages/Kids[]/Count 0>>endobj
trailer<</Root 1 0 R>>
PDF
fix "$T/zero.pdf" "$OUT/43_zero_paginas.pdf"

# Transparência (CA/ca), blend Multiply, gradiente axial e radial.
cat > "$T/tr.pdf" <<'PDF'
%PDF-1.4
1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj
2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj
3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 595 420]/Contents 4 0 R/Resources<</Font<</F1 5 0 R>>/ExtGState<</A5<</ca 0.5/CA 0.5>>/MUL<</BM/Multiply>>>>/Shading<</SA 6 0 R/SR 7 0 R>>>>>>endobj
4 0 obj<</Length 0>>stream
q 30 230 250 160 re W n /SA sh Q
q 320 230 250 160 re W n /SR sh Q
q /A5 gs 1 0 0 rg 60 60 140 140 re f 0 0.6 0 rg 130 90 140 140 re f 0 0 1 rg 100 30 140 140 re f Q
q /MUL gs 1 1 0 rg 330 60 120 120 re f 0 1 1 rg 390 90 120 120 re f 1 0 1 rg 360 30 120 120 re f Q
BT /F1 18 Tf 0 g 40 400 Td (Gradientes e transpar\352ncia) Tj ET
endstream
endobj
5 0 obj<</Type/Font/Subtype/Type1/BaseFont/Helvetica/Encoding/WinAnsiEncoding>>endobj
6 0 obj<</ShadingType 2/ColorSpace/DeviceRGB/Coords[30 0 280 0]/Function 8 0 R/Extend[true true]>>endobj
7 0 obj<</ShadingType 3/ColorSpace/DeviceRGB/Coords[445 310 5 445 310 90]/Function 9 0 R/Extend[true true]>>endobj
8 0 obj<</FunctionType 2/Domain[0 1]/C0[1 0.8 0]/C1[0.1 0.1 0.8]/N 1>>endobj
9 0 obj<</FunctionType 2/Domain[0 1]/C0[1 1 1]/C1[0.8 0 0.3]/N 1>>endobj
trailer<</Root 1 0 R>>
PDF
fix "$T/tr.pdf" "$OUT/44_transparencia_gradientes.pdf"

# Formulário AcroForm: campo de texto, caixa de seleção e botão, com streams de aparência.
cat > "$T/form.pdf" <<'PDF'
%PDF-1.7
1 0 obj<</Type/Catalog/Pages 2 0 R/AcroForm<</Fields[6 0 R 7 0 R 8 0 R]/DR<</Font<</Helv 5 0 R>>>>/DA(/Helv 12 Tf 0 g)/NeedAppearances false>>>>endobj
2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj
3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 595 420]/Contents 4 0 R/Resources<</Font<</Helv 5 0 R>>>>/Annots[6 0 R 7 0 R 8 0 R]>>endobj
4 0 obj<</Length 0>>stream
BT /Helv 20 Tf 40 380 Td (Formul\341rio de inscri\347\343o) Tj ET
BT /Helv 12 Tf 40 340 Td (Nome:) Tj ET
BT /Helv 12 Tf 40 300 Td (Aceito os termos:) Tj ET
endstream
endobj
5 0 obj<</Type/Font/Subtype/Type1/BaseFont/Helvetica/Encoding/WinAnsiEncoding>>endobj
6 0 obj<</Type/Annot/Subtype/Widget/FT/Tx/T(nome)/V(Cl\341udio Medeiros)/DA(/Helv 12 Tf 0 g)/Rect[100 332 400 352]/F 4/MK<</BC[0 0 0]/BG[0.93 0.95 1]>>/AP<</N 9 0 R>>>>endobj
7 0 obj<</Type/Annot/Subtype/Widget/FT/Btn/T(aceito)/V/Yes/AS/Yes/Rect[160 296 176 312]/F 4/MK<</BC[0 0 0]>>/AP<</N<</Yes 10 0 R/Off 11 0 R>>>>>>endobj
8 0 obj<</Type/Annot/Subtype/Widget/FT/Btn/Ff 65536/T(enviar)/Rect[40 240 140 268]/F 4/MK<</BC[0 0 0]/BG[0.8 0.9 0.8]/CA(Enviar)>>/AP<</N 12 0 R>>>>endobj
9 0 obj<</Type/XObject/Subtype/Form/BBox[0 0 300 20]/Resources<</Font<</Helv 5 0 R>>>>/Length 0>>stream
0.93 0.95 1 rg 0 0 300 20 re f 0 G 0 0 300 20 re S BT /Helv 12 Tf 0 g 4 6 Td (Cl\341udio Medeiros) Tj ET
endstream
endobj
10 0 obj<</Type/XObject/Subtype/Form/BBox[0 0 16 16]/Length 0>>stream
0 G 0 0 16 16 re S 2 w 3 8 m 7 3 l 13 13 l S
endstream
endobj
11 0 obj<</Type/XObject/Subtype/Form/BBox[0 0 16 16]/Length 0>>stream
0 G 0 0 16 16 re S
endstream
endobj
12 0 obj<</Type/XObject/Subtype/Form/BBox[0 0 100 28]/Resources<</Font<</Helv 5 0 R>>>>/Length 0>>stream
0.8 0.9 0.8 rg 0 0 100 28 re f 0 G 0 0 100 28 re S BT /Helv 12 Tf 0 g 30 9 Td (Enviar) Tj ET
endstream
endobj
trailer<</Root 1 0 R>>
PDF
fix "$T/form.pdf" "$OUT/45_formulario_acroform.pdf"

# Página gigante (A0, 2384 x 3370 pt) com vetores e texto, e CropBox menor que o MediaBox.
cat > "$T/a0.pdf" <<'PDF'
%PDF-1.4
1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj
2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj
3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 2384 3370]/Contents 4 0 R/Resources<</Font<</F1 5 0 R>>>>>>endobj
4 0 obj<</Length 0>>stream
0.9 0.95 1 rg 0 0 2384 3370 re f 0.1 0.3 0.7 rg 100 100 2184 400 re f
1 1 1 rg BT /F1 160 Tf 160 260 Td (Cartaz A0) Tj ET
0 g BT /F1 60 Tf 160 3100 Td (Pagina gigante, texto pequeno em relacao a pagina) Tj ET
1 0 0 RG 12 w 200 800 m 2184 3000 l S 0 0.5 0 RG 200 3000 m 2184 800 l S
endstream
endobj
5 0 obj<</Type/Font/Subtype/Type1/BaseFont/Helvetica-Bold>>endobj
trailer<</Root 1 0 R>>
PDF
fix "$T/a0.pdf" "$OUT/46_pagina_gigante_A0.pdf"

# Cartão pequeno (216 x 360 pt) seguido de páginas A4 e A3: tamanhos mistos, primeira página pequena.
cat > "$T/card.pdf" <<'PDF'
%PDF-1.4
1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj
2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj
3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 216 360]/Contents 4 0 R/Resources<</Font<</F1 5 0 R>>>>>>endobj
4 0 obj<</Length 0>>stream
0.95 0.9 0.8 rg 0 0 216 360 re f 0.6 0.2 0.1 rg 10 10 196 340 re S BT /F1 18 Tf 0 g 30 300 Td (Cart\343o 3x5) Tj ET
BT /F1 11 Tf 30 270 Td (Primeira p\341gina pequena) Tj ET
endstream
endobj
5 0 obj<</Type/Font/Subtype/Type1/BaseFont/Helvetica/Encoding/WinAnsiEncoding>>endobj
trailer<</Root 1 0 R>>
PDF
fix "$T/card.pdf" "$T/card_ok.pdf"
qpdf --empty --pages "$T/card_ok.pdf" 1 "$OUT/10_word_simples_acentos.pdf" 1 "$OUT/37_ppt_16x9_paisagem.pdf" 1 "$OUT/46_pagina_gigante_A0.pdf" 1 -- "$OUT/47_tamanhos_mistos.pdf"

# Vetorial pesado: 12.000 curvas de Bézier coloridas, tipo mapa (awk com semente fixa).
awk 'BEGIN{srand(42);
 print "%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj";
 print "3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 1190 842]/Contents 4 0 R>>endobj\n4 0 obj<</Length 0>>stream";
 print "0.96 0.94 0.88 rg 0 0 1190 842 re f";
 for(i=0;i<12000;i++){x=rand()*1190;y=rand()*842;
  printf "%.2f %.2f %.2f RG %.2f %.2f %.2f rg %.2f w\n",rand()*0.6,rand()*0.6+0.2,rand()*0.8,rand(),rand()*0.5+0.5,rand()*0.6,rand()*1.2+0.2;
  printf "%.1f %.1f m %.1f %.1f %.1f %.1f %.1f %.1f c %s\n",x,y,x+rand()*60-30,y+rand()*60-30,x+rand()*60-30,y+rand()*60-30,x+rand()*80-40,y+rand()*80-40,(i%3==0?"b":"S")}
 print "endstream\nendobj\ntrailer<</Root 1 0 R>>"}' > "$T/vec.pdf"
fix "$T/vec.pdf" "$OUT/48_vetorial_pesado_mapa.pdf"

# ---------- 3. Derivados com qpdf ----------
qpdf --rotate=+90:1 "$OUT/10_word_simples_acentos.pdf" "$OUT/49_pagina_rotacionada_90.pdf"
qpdf --empty --pages "$OUT/10_word_simples_acentos.pdf" 1 -- --encrypt abrir123 dono456 256 -- "$OUT/50_cripto_senha_aes256.pdf"
qpdf --allow-weak-crypto --empty --pages "$OUT/10_word_simples_acentos.pdf" 1 -- --encrypt "" dono456 128 --print=none --extract=n --modify=none -- "$OUT/51_cripto_sem_senha_abrir_aes128.pdf"
qpdf --allow-weak-crypto --empty --pages "$OUT/10_word_simples_acentos.pdf" 1 -- --encrypt "" dono456 40 -- "$OUT/52_cripto_sem_senha_abrir_rc4_40.pdf"

# ---------- 4. Escaneado (só imagem), com ImageMagick ----------
pdftoppm -r 150 -f 1 -l 1 -png "$OUT/13_word_multipagina.pdf" "$T/scan"
SRC=$(ls "$T"/scan*.png | head -1)
convert "$SRC" -colorspace Gray -rotate 0.6 -attenuate 0.35 +noise Gaussian -blur 0x0.6 -background '#d8d4cc' -quality 70 "$T/scan.jpg"
convert "$T/scan.jpg" -density 150 "$OUT/60_escaneado_so_imagem.pdf" 2>/dev/null || img2pdf_fail=1
# Segunda variante: colorida e de alta resolução (300 dpi), 4 páginas.
pdftoppm -r 300 -f 1 -l 1 -png "$OUT/12_word_imagens.pdf" "$T/scan2"
SRC2=$(ls "$T"/scan2*.png | head -1)
convert "$SRC2" -attenuate 0.2 +noise Gaussian -quality 80 "$T/scan2.jpg"
convert "$T/scan2.jpg" "$T/scan2.jpg" "$T/scan2.jpg" "$T/scan2.jpg" -density 300 "$OUT/61_escaneado_cor_300dpi_4pag.pdf" 2>/dev/null || true

# ---------- 5. Quebrados ----------
SZ=$(stat -c %s "$OUT/12_word_imagens.pdf")
head -c $((SZ/2)) "$OUT/12_word_imagens.pdf" > "$OUT/70_truncado_50pct.pdf"
SZ=$(stat -c %s "$OUT/13_word_multipagina.pdf")
head -c $((SZ*9/10)) "$OUT/13_word_multipagina.pdf" > "$OUT/71_truncado_90pct.pdf"
# Corrompido: 64 bytes aleatórios fixos no meio do arquivo.
cp "$OUT/12_word_imagens.pdf" "$OUT/72_corrompido_meio.pdf"
SZ=$(stat -c %s "$OUT/72_corrompido_meio.pdf")
head -c 64 /dev/zero | tr '\0' 'Z' | dd of="$OUT/72_corrompido_meio.pdf" bs=1 seek=$((SZ/3)) conv=notrunc 2>/dev/null
# Corrompido: trailer/startxref destruído (recuperável por reconstrução de xref).
sed 's/startxref/startxreg/' "$OUT/11_word_tabelas.pdf" | head -c -40 > "$OUT/73_corrompido_xref.pdf"
# Lixo de 1 KB antes do cabeçalho %PDF-.
(head -c 1024 /dev/urandom | tr -d '\0%' ; cat "$OUT/10_word_simples_acentos.pdf") > "$OUT/74_lixo_antes_do_cabecalho.pdf"
: > "$OUT/75_zero_bytes.pdf"
cp "$WORD_DIR/14_md_rico.md" "$OUT/76_texto_puro_com_extensao_pdf.pdf" 2>/dev/null || echo "texto puro" > "$OUT/76_texto_puro_com_extensao_pdf.pdf"
cp "$WORD_DIR/13_html_rico.html" "$OUT/77_html_com_extensao_pdf.pdf" 2>/dev/null || true
convert -size 400x300 gradient:red-blue "$OUT/78_jpeg_com_extensao_pdf.pdf.jpg" && mv "$OUT/78_jpeg_com_extensao_pdf.pdf.jpg" "$OUT/78_jpeg_com_extensao_pdf.pdf"

# ---------- 6. Grandes (fora do git) ----------
# 6a. 300+ páginas de texto: o txt longo repetido, convertido pelo LibreOffice.
mkdir -p "$T/g1"
for k in $(seq 1 60); do cat "$WORD_DIR/17_txt_longo_multipagina.txt"; done > "$T/g1/livro_grande.txt"
(cd "$T/g1" && soffice --headless --convert-to pdf livro_grande.txt >/dev/null 2>&1) || true
[ -f "$T/g1/livro_grande.pdf" ] && cp "$T/g1/livro_grande.pdf" "$OUT/big/90_grande_texto_200p_mais.pdf"
# 6b. Aprox. 40 MB: 28 páginas digitalizadas com ruído (JPEG ~1,4 MB cada, imagens distintas).
for n in $(seq 1 28); do
  convert -size 2480x3508 plasma:fractal -attenuate 0.6 +noise Gaussian -quality 92 "$T/p$n.jpg"
done
convert "$T"/p?.jpg "$T"/p??.jpg -density 300 "$OUT/big/91_grande_escaneado_40mb.pdf" 2>/dev/null || true
echo "corpus gerado em $OUT"
