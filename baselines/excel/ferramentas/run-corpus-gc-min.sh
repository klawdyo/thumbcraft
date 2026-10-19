#!/usr/bin/env bash
# run-corpus.sh IN_DIR OUT_DIR [extra gc-min options...]
# One PNG per workbook in IN_DIR (MANIFEST.tsv and non-spreadsheets ignored), made with gc-min,
# 30 s limit per file. OUT_DIR/results.tsv: file, exit code, milliseconds, png bytes.
set -u
IN="${1:?usage: run-corpus.sh IN_DIR OUT_DIR [gc-min options]}"; OUT="${2:?}"; shift 2
HERE="$(cd "$(dirname "$0")" && pwd)"
BIN="${GC_MIN:-$HERE/target/release/gc-min}"
[ -x "$BIN" ] || { echo "no gc-min binary at $BIN (set GC_MIN)" >&2; exit 2; }
mkdir -p "$OUT"
printf 'file\texit\tms\tpng_bytes\n' > "$OUT/results.tsv"
for f in "$IN"/*; do
  case "$f" in *.xlsx|*.xlsm|*.xls|*.ods|*.csv) ;; *) continue ;; esac
  b="$(basename "$f")"; n="${b%.*}"
  rm -f "$OUT/$n.png"
  t0=$(date +%s%N)
  timeout 30 "$BIN" "$f" "$OUT/$n.png" "$@" >/dev/null 2>"$OUT/$n.err"
  rc=$?
  ms=$(( ($(date +%s%N) - t0) / 1000000 ))
  sz=0; [ -f "$OUT/$n.png" ] && sz=$(stat -c %s "$OUT/$n.png")
  [ -s "$OUT/$n.err" ] || rm -f "$OUT/$n.err"
  printf '%s\t%s\t%s\t%s\n' "$b" "$rc" "$ms" "$sz" >> "$OUT/results.tsv"
done
awk -F'\t' 'NR>1{n++; if($2==0)ok++; t+=$3; if($3>m)m=$3} END{printf "%d files, %d ok, mean %.0f ms, max %d ms\n", n, ok, t/n, m}' "$OUT/results.tsv"
