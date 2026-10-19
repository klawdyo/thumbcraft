#!/bin/bash
# Uso: run-lo.sh IN_DIR OUT_DIR  (um processo soffice novo por arquivo, timeout 60 s)
IN="$1"; OUT="$2"; PROF="${LO_PROFILE:-/tmp/lo-profile-baseline}"
mkdir -p "$OUT"; printf 'arquivo\texit\tms\ttamanho_png_bytes\n' > "$OUT/results.tsv"
for f in "$IN"/*; do
  b=$(basename "$f"); [ "$b" = MANIFEST.tsv ] && continue
  s=$(date +%s%N)
  timeout 60 nice -n 10 soffice -env:UserInstallation=file://$PROF --headless --convert-to png --outdir "$OUT" "$f" >"$OUT/.last.log" 2>&1
  ec=$?; e=$(date +%s%N); stem="${b%.*}"
  sz=0; [ -f "$OUT/$stem.png" ] && sz=$(stat -c %s "$OUT/$stem.png")
  printf '%s\t%s\t%s\t%s\n' "$b" "$ec" $(( (e-s)/1000000 )) "$sz" >> "$OUT/results.tsv"
done
rm -f "$OUT/.last.log"
