#!/usr/bin/env bash
# Uso: run-corpus.sh IN_DIR OUT_DIR  (gera OUT_DIR/<nome>.png e OUT_DIR/results.tsv)
set -u
IN="${1:?uso: run-corpus.sh IN_DIR OUT_DIR}"; OUT="${2:?uso: run-corpus.sh IN_DIR OUT_DIR}"
BIN="${GP_MIN:-S/grid-png/target/release/gp-min}"
mkdir -p "$OUT"
printf 'arquivo\tms\tcodigo\tpng_bytes\tmensagem\n' > "$OUT/results.tsv"
for f in "$IN"/*; do
  [ -f "$f" ] || continue
  b=$(basename "$f"); n="${b%.*}"; png="$OUT/$n.png"; rm -f "$png"
  t0=$(date +%s%N)
  timeout 30 "$BIN" "$f" "$png" 2>"$OUT/.err" >/dev/null
  code=$?
  msg=$(tail -n 1 "$OUT/.err" | tr "\t\n" "  ")
  t1=$(date +%s%N)
  sz=0; [ -f "$png" ] && sz=$(stat -c %s "$png")
  printf '%s\t%d\t%d\t%d\t%s\n' "$b" $(( (t1-t0)/1000000 )) "$code" "$sz" "$msg" >> "$OUT/results.tsv"
done
rm -f "$OUT/.err"
echo "ok: $(awk -F'\t' 'NR>1&&$3==0' "$OUT/results.tsv" | wc -l) de $(($(wc -l < "$OUT/results.tsv")-1)) arquivos; ver $OUT/results.tsv"
