#!/usr/bin/env bash
# Formal PDF regression: PDFs produced by the ORIGINAL code (golden) against PDFs produced by the
# refactored copy, in three layers; the first layer that fails is a regression.
#   A) bytes (cmp). Only the volatile fields listed in VOLATILE_SED are normalised, on both sides.
#   B) when A fails: decompressed content of each page (qpdf --qdf), diff of the operations.
#   C) pixels: both PDFs rasterised with pdftoppm at the same resolution, `compare -metric AE`
#      page by page (expected: 0).
# Usage:
#   regress-pdf.sh golden   regenerate golden-pdf/ with the original binary (once, before refactoring)
#   regress-pdf.sh [check]  rebuild the copy (unless NO_BUILD=1), regenerate every PDF, compare
# Exit status: 0 only when every case is PASS.
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
SCR="$(cd "$HERE/.." && pwd)"                         # .../grid-png
ROOT="${ROOT:-$HERE}"
COPY="${COPY:-$HERE/gridcraft}"
SAMPLES="${SAMPLES:-$HERE/samples}"
CORPUS="${CORPUS:-$SCR/c-corpus/in}"
GOLD="$HERE/golden-pdf"
NEW="${NEW:-$HERE/new-pdf}"
ORIG_BIN="${ORIG_BIN:-$GOLD/bin/gridcraft-cli.orig}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$SCR/target}"
NEW_BIN="${NEW_BIN:-$CARGO_TARGET_DIR/debug/gridcraft-cli}"
DPI="${DPI:-50}"
MODE="${1:-check}"

# Nothing volatile is expected: the writer emits no date, no /ID and no UUID (verified on the
# golden set by `grep -a -c -E 'CreationDate|ModDate|/ID|UUID'`). The list stays empty on purpose,
# so nothing can be hidden by normalisation.
VOLATILE_SED=()

cases() {   # id|input|command|json   (@OUT@ = output path of the case)
  local f b
  for f in "$SAMPLES"/[0-9]*.xlsx "$CORPUS"/*.xlsx; do
    [ -f "$f" ] || continue
    b="$(basename "$f" .xlsx)"
    [ "$(dirname "$f")" = "$CORPUS" ] && b="c-$b" || b="s-$b"
    echo "$b.d|$f|file.exportPdf|{\"path\":\"@OUT@\"}"
    echo "$b.fit|$f|file.exportPdf|{\"path\":\"@OUT@\",\"fitToPage\":true}"
    echo "$b.rng|$f|file.exportPdf|{\"path\":\"@OUT@\",\"range\":\"B2:F30\"}"
    echo "$b.all|$f|file.exportPdf|{\"path\":\"@OUT@\",\"sheets\":\"all\"}"
    echo "$b.prn|$f|file.print|{\"path\":\"@OUT@\"}"
  done
  local first
  first="$(ls "$SAMPLES"/[0-9]*.xlsx | head -1)"
  echo "err.range|$first|file.exportPdf|{\"path\":\"@OUT@\",\"range\":\"not a range\"}"
  echo "err.sheets|$first|file.exportPdf|{\"path\":\"@OUT@\",\"sheets\":\"bogus\"}"
  echo "err.dir|$first|file.exportPdf|{\"path\":\"/nonexistent-dir/x.pdf\"}"
  echo "err.empty|$first|file.exportPdf|{}"
  echo "err.big|$first|file.exportPdf|{\"path\":\"@OUT@\",\"range\":\"A1:XFD1048576\"}"
}

run_case() {  # bin outdir id input cmd json -> writes outdir/id.pdf and outdir/id.txt (stdout, stderr, exit code)
  local bin="$1" out="$2" id="$3" in="$4" cmd="$5" js="$6" pdf
  pdf="$out/$id.pdf"
  js="${js//@OUT@/$pdf}"
  timeout 120 "$bin" run --in "$in" --cmd "$cmd=$js" --quiet >"$out/$id.out" 2>"$out/$id.err"
  echo "exit=$?" >"$out/$id.txt"
  # paths differ between the two trees' output dirs: neutralise only the output directory name
  sed -e "s#$out#OUTDIR#g" "$out/$id.out" "$out/$id.err" >>"$out/$id.txt"
  rm -f "$out/$id.out" "$out/$id.err"
}

gen_all() {  # bin outdir
  local bin="$1" out="$2" line
  mkdir -p "$out"
  cases | while IFS='|' read -r id in cmd js; do
    run_case "$bin" "$out" "$id" "$in" "$cmd" "$js"
  done
}

if [ "$MODE" = golden ]; then
  [ -x "$ORIG_BIN" ] || { echo "no original binary at $ORIG_BIN" >&2; exit 2; }
  rm -rf "$GOLD/pdf"; mkdir -p "$GOLD"
  gen_all "$ORIG_BIN" "$GOLD/pdf"
  ( cd "$GOLD/pdf" && sha256sum *.pdf > ../SHA256SUMS )
  echo "golden: $(ls "$GOLD/pdf"/*.pdf | wc -l) pdfs in $GOLD/pdf"
  exit 0
fi

if [ "${NO_BUILD:-0}" != 1 ]; then
  ( cd "$COPY" && cargo build -p gridcraft-cli 2>&1 | tail -3 ) || { echo "build failed" >&2; exit 2; }
fi
[ -x "$NEW_BIN" ] || { echo "no binary at $NEW_BIN" >&2; exit 2; }
rm -rf "$NEW"; mkdir -p "$NEW"
gen_all "$NEW_BIN" "$NEW"

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
norm() { if [ ${#VOLATILE_SED[@]} -gt 0 ]; then sed "${VOLATILE_SED[@]}" "$1"; else cat "$1"; fi; }
pass=0; fail=0
printf '%-26s %-6s %-7s %-12s %s\n' ID BYTES STDOUT "PIXELS(diff)" RESULT
cases | cut -d'|' -f1 | while read -r id; do
  g="$GOLD/pdf/$id.pdf"; n="$NEW/$id.pdf"; res=PASS; bytes=n/a; so=same; px=n/a
  cmp -s "$GOLD/pdf/$id.txt" "$NEW/$id.txt" || { so=DIFF; res=FAIL; }
  if [ -f "$g" ] || [ -f "$n" ]; then
    if [ -f "$g" ] && [ -f "$n" ]; then
      if cmp -s <(norm "$g") <(norm "$n"); then
        bytes=yes
      else
        bytes=NO; res=FAIL
        # layer B: decompressed content, diff of operations
        qpdf --qdf --object-streams=disable "$g" "$TMP/g.qdf" 2>/dev/null; qpdf --qdf --object-streams=disable "$n" "$TMP/n.qdf" 2>/dev/null
        diff -a "$TMP/g.qdf" "$TMP/n.qdf" | head -20 | sed 's/^/    B| /'
      fi
      # layer C: pixels
      rm -f "$TMP"/g-*.png "$TMP"/n-*.png
      pdftoppm -r "$DPI" -png "$g" "$TMP/g" 2>/dev/null; pdftoppm -r "$DPI" -png "$n" "$TMP/n" 2>/dev/null
      px=""; total=0
      gp=$(ls "$TMP"/g-*.png 2>/dev/null | wc -l); np=$(ls "$TMP"/n-*.png 2>/dev/null | wc -l)
      if [ "$gp" != "$np" ]; then px="pages $gp/$np"; res=FAIL
      else
        for gf in "$TMP"/g-*.png; do
          [ -f "$gf" ] || continue
          nf="$TMP/n-${gf##*/g-}"
          d=$(compare -metric AE "$gf" "$nf" null: 2>&1 | awk '{print $1}')
          total=$((total + ${d:-1}))
          [ "${d:-1}" != 0 ] && { px="$px p${gf##*g-}=$d"; res=FAIL; }
        done
        [ -z "$px" ] && px="0 (${gp}p)"
      fi
    else
      bytes=MISSING; res=FAIL
    fi
  fi
  printf '%-26s %-6s %-7s %-12s %s\n' "$id" "$bytes" "$so" "$px" "$res"
done | tee "$HERE/regress-table.txt"
total=$(grep -c -E ' (PASS|FAIL)$' "$HERE/regress-table.txt")
npass=$(grep -c ' PASS$' "$HERE/regress-table.txt")
nbyte=$(awk '$2=="yes"' "$HERE/regress-table.txt" | wc -l)
npdf=$(awk '$2=="yes" || $2=="NO" || $2=="MISSING"' "$HERE/regress-table.txt" | wc -l)
echo "SUMMARY: $npass of $total PASS; layer A byte-identical: $nbyte of $npdf pdf cases"
[ "$npass" = "$total" ]
