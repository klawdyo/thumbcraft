#!/usr/bin/env bash
# Contact sheet (ImageMagick): miniaturas lado a lado com o nome sob cada uma.
# Arquivos sem PNG (falha) viram um tile vermelho "FALHOU".
# Uso: scripts/sheets.sh DIR_PNGS results.tsv SAÍDA.png [LARGURA_TILE=240] [COLUNAS=6]
set -euo pipefail
D="${1:?uso: sheets.sh DIR_PNGS results.tsv SAIDA.png [tile] [colunas]}"
TSV="${2:?}"; OUT="${3:?}"; T="${4:-240}"; C="${5:-6}"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
i=0
tail -n +2 "$TSV" | while IFS=$'\t' read -r rel motor rc ms bytes dim; do
  i=$((i + 1)); png="$D/$(echo "$rel" | tr '/' '_').png"
  nome="${rel##*/}"; nome="${nome:0:30}"
  tile="$TMP/$(printf '%04d' "$i").png"
  if [[ "$rc" == 0 && -s "$png" ]]; then
    convert "$png" -background white -alpha remove -resize "${T}x${T}" -gravity center -extent "${T}x${T}" \
      -bordercolor '#cccccc' -border 1 "$tile"
  else
    convert -size "${T}x${T}" xc:'#c62828' -gravity center -fill white -pointsize 28 -annotate 0 "FALHOU\n(código $rc)" "$tile"
  fi
  convert "$tile" -background white -gravity south -splice 0x22 -font DejaVu-Sans -pointsize 12 -fill black -annotate +0+4 "$nome" "$tile"
done
montage "$TMP"/*.png -tile "${C}x" -geometry +6+6 -background white "$OUT"
echo "$OUT ($(identify -format '%wx%h' "$OUT"))"
