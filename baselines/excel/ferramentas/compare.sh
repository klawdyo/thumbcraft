#!/bin/bash
# compare.sh DIR_A DIR_B DIR_C OUT_DIR [ROTULO_A ROTULO_B ROTULO_C]
# Rotulos padrao: libreoffice pdf-raster refatoracao. Requer ImageMagick (convert, identify).
# Saidas em OUT_DIR: contact-<rotulo>.png, side-by-side-NN.png (lotes de 7 linhas), results.tsv
# Nomes de base = uniao dos PNGs das 3 pastas (ou de results.tsv, para detectar ausencias).
set -u
if [ $# -lt 4 ]; then echo "uso: $0 DIR_A DIR_B DIR_C OUT_DIR [rotulos]" >&2; exit 2; fi
D=("$1" "$2" "$3"); OUT="$4"
L=("${5:-libreoffice}" "${6:-pdf-raster}" "${7:-refatoracao}")
BATCH="${BATCH:-7}"; TW=360; TH=240; FONT="${FONT:-DejaVu-Sans}"
mkdir -p "$OUT"; W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
# lista de nomes de base: PNGs + entradas de results.tsv (arquivo sem extensao)
{ for d in "${D[@]}"; do
    ls "$d" 2>/dev/null | sed -n 's/\.png$//p'
    [ -f "$d/results.tsv" ] && awk -F'\t' 'NR>1{n=$1; sub(/\.[^.]*$/,"",n); print n}' "$d/results.tsv"
  done; } | sort -u > "$W/names.txt"
N=$(wc -l < "$W/names.txt"); echo "arquivos: $N"
# tile: imagem (ou FALHOU) com rotulo sob ela, tamanho fixo TWx(TH+22)
tile() { # $1 png|"" $2 rotulo $3 saida
  if [ -n "$1" ] && [ -s "$1" ] && identify -quiet "$1[0]" >/dev/null 2>&1; then
    convert "$1[0]" -background white -alpha remove -alpha off +repage -resize "${TW}x${TH}>" -gravity center -background '#f4f4f4' -extent ${TW}x${TH} \
      -bordercolor '#999999' -border 1 \
      -gravity south -background white -font "$FONT" -pointsize 12 -splice 0x22 -fill black -annotate +0+4 "$2" "$3" 2>/dev/null
  else
    convert -size ${TW}x${TH} xc:'#d32f2f' -gravity center -font "$FONT" -pointsize 36 -fill white -annotate +0+0 "FALHOU" \
      -gravity south -background white -font "$FONT" -pointsize 12 -splice 0x22 -fill black -annotate +0+4 "$2" "$3"
  fi
}
short() { local s="$1"; [ ${#s} -gt 46 ] && s="${s:0:43}..."; echo "$s"; }
png() { local f="$1/$2.png"; [ -s "$f" ] && echo "$f"; }
# (a) contact sheets
for i in 0 1 2; do
  mkdir -p "$W/c$i"; k=0
  while read -r n; do
    k=$((k+1)); tile "$(png "${D[$i]}" "$n")" "$(short "$n")" "$W/c$i/$(printf %04d $k).png"
  done < "$W/names.txt"
  montage "$W/c$i"/*.png -tile 5x -geometry +6+6 -background white "$OUT/contact-${L[$i]}.png" 2>/dev/null \
    || echo "montage falhou para ${L[$i]}" >&2
done
# (b) lado a lado: cabecalho com rotulos + 1 linha por arquivo, lotes de BATCH
k=0; b=0; rows=()
flush() {
  [ ${#rows[@]} -eq 0 ] && return
  b=$((b+1)); f=$(printf "%s/side-by-side-%02d.png" "$OUT" $b)
  convert "${rows[@]}" -append +repage "$f"; rows=()
}
while read -r n; do
  k=$((k+1)); r="$W/r$k"
  for i in 0 1 2; do tile "$(png "${D[$i]}" "$n")" "${L[$i]}: $(short "$n")" "$r-$i.png"; done
  convert "$r-0.png" "$r-1.png" "$r-2.png" -background white +append +repage "$r.png"
  rows+=("$r.png")
  [ ${#rows[@]} -ge "$BATCH" ] && flush
done < "$W/names.txt"
flush
# (c) results.tsv consolidado
dim() { local f="$1/$2.png"; if [ -s "$f" ]; then identify -quiet -format '%wx%h' "$f[0]" 2>/dev/null || echo "invalido"; else echo "-"; fi; }
ex() { [ -s "$1/$2.png" ] && echo sim || echo nao; }
tm() { # tempo (ms) e exit de results.tsv da pasta, casando por nome de base
  [ -f "$1/results.tsv" ] || { echo "-"; return; }
  awk -F'\t' -v n="$2" 'NR>1{b=$1; sub(/\.[^.]*$/,"",b); if(b==n){print $3; f=1; exit}} END{if(!f)print "-"}' "$1/results.tsv"; }
ec() { [ -f "$1/results.tsv" ] || { echo "-"; return; }
  awk -F'\t' -v n="$2" 'NR>1{b=$1; sub(/\.[^.]*$/,"",b); if(b==n){print $2; f=1; exit}} END{if(!f)print "-"}' "$1/results.tsv"; }
{
  printf 'arquivo'
  for i in 0 1 2; do printf '\texiste_%s\tpx_%s\tms_%s\texit_%s' "${L[$i]}" "${L[$i]}" "${L[$i]}" "${L[$i]}"; done
  printf '\n'
  while read -r n; do
    printf '%s' "$n"
    for i in 0 1 2; do printf '\t%s\t%s\t%s\t%s' "$(ex "${D[$i]}" "$n")" "$(dim "${D[$i]}" "$n")" "$(tm "${D[$i]}" "$n")" "$(ec "${D[$i]}" "$n")"; done
    printf '\n'
  done < "$W/names.txt"
} > "$OUT/results.tsv"
echo "saida: $OUT ($(ls "$OUT" | wc -l) arquivos, $b imagens lado a lado)"
