#!/usr/bin/env bash
# Resume um results.tsv por motor: taxa de sucesso, tempo médio e máximo (ms), PNG médio (bytes).
# Uso: scripts/summarize.sh results.tsv
set -euo pipefail
printf '%-6s %5s %5s %7s %9s %9s %12s  %s\n' motor total ok taxa ms_medio ms_max png_medio codigos
awk -F'\t' 'NR>1 {
  m=$2; n[m]++; if ($3==0) {ok[m]++; sb[m]+=$5} t[m]+=$4; if ($4>mx[m]) mx[m]=$4
  codes[m, $3]++
}
END {
  for (m in n) {
    c=""; for (k in codes) { split(k, p, SUBSEP); if (p[1]==m) c = c " " p[2] "x" codes[k] }
    printf "%-6s %5d %5d %6.1f%% %9.0f %9d %12.0f  %s\n", m, n[m], ok[m]+0, 100*(ok[m]+0)/n[m], t[m]/n[m], mx[m], (ok[m]?sb[m]/ok[m]:0), c
  }
}' "$1" | sort
