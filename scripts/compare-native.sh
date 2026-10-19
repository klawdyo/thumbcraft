#!/usr/bin/env bash
# Prova que a unificação não mudou a renderização: roda `thumbcraft --native-scale 2` e o mini
# correspondente (wc-min, dc-min, gc-min) em cada arquivo dos três corpora e compara byte a byte.
#
# Uso: scripts/compare-native.sh [DIR_WORD DIR_PPT DIR_XLSX]   (padrão: corpus/docx corpus/pptx corpus/xlsx)
# Variáveis: WC_MIN, DC_MIN, GC_MIN (caminhos dos minis), SAIDA (pasta de trabalho), RECURSIVO=1.
# Resultado por arquivo: PASS (bytes iguais), PASS-ERRO (os dois falham), ESPERADO (diferença
# conhecida e intencional: slide danificado no PPTX) ou FAIL. Sai com 1 se houver FAIL.
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/env.sh"
SCRATCH="${SCRATCH:-/tmp/claude-0/-home-claude/37e359fe-23e0-5bd2-965b-70005e330799/scratchpad}"
WC_MIN="${WC_MIN:-$SCRATCH/target-small/release/wc-min}"
DC_MIN="${DC_MIN:-$SCRATCH/pptx-test/target-min/release/dc-min}"
GC_MIN="${GC_MIN:-$SCRATCH/grid-png/a-refactor/feasibility/target/release/gc-min}"
DW="${1:-$RAIZ/corpus/docx}"; DP="${2:-$RAIZ/corpus/pptx}"; DX="${3:-$RAIZ/corpus/xlsx}"
W="${SAIDA:-$(mktemp -d)}"; mkdir -p "$W"
prof=1; [[ "${RECURSIVO:-0}" == 1 ]] && prof=9
declare -A tot pass fail
fails=0
printf '%-8s %-44s %-10s %s\n' motor arquivo resultado detalhe

cmp_one() { # motor arquivo mini_cmd...
  local motor="$1" f="$2"; shift 2
  local rel base a b rca rcb res det=""
  rel="$(basename "$f")"; base="$W/$motor-$(echo "$rel" | tr '/ ' '__')"
  a="$base.mini.png"; b="$base.tc.png"
  rm -f "$a" "$b"
  timeout 60 "$@" "$f" "$a" >/dev/null 2>&1; rca=$?
  timeout 60 "$TC_BIN" "$f" "$b" --native-scale 2 >/dev/null 2>&1; rcb=$?
  if [[ $rca -eq 0 && $rcb -eq 0 ]]; then
    if cmp -s "$a" "$b"; then res=PASS; else res=FAIL; det="bytes diferentes ($(stat -c %s "$a") x $(stat -c %s "$b"))"; fi
  elif [[ $rca -ne 0 && $rcb -ne 0 ]]; then res=PASS-ERRO; det="mini=$rca thumbcraft=$rcb"
  elif [[ $motor == deck && $rca -eq 0 && $rcb -eq 4 ]]; then res=ESPERADO; det="o mini renderiza outro slide quando o slide 1 está danificado; o thumbcraft recusa (código 4)"
  else res=FAIL; det="mini=$rca thumbcraft=$rcb"; fi
  tot[$motor]=$(( ${tot[$motor]:-0} + 1 ))
  [[ $res == FAIL ]] && { fails=$((fails + 1)); fail[$motor]=$(( ${fail[$motor]:-0} + 1 )); } || pass[$motor]=$(( ${pass[$motor]:-0} + 1 ))
  printf '%-8s %-44s %-10s %s\n' "$motor" "$rel" "$res" "$det"
  [[ $res == PASS ]] && rm -f "$a" "$b"
}

list() { find "$1" -maxdepth "$prof" -type f ! -name MANIFEST.tsv ! -name 'README*' | sort; }
while read -r f; do cmp_one word "$f" "$WC_MIN"; done < <(list "$DW")
while read -r f; do cmp_one deck "$f" "$DC_MIN"; done < <(list "$DP") 2>/dev/null
while read -r f; do cmp_one grid "$f" "$GC_MIN"; done < <(list "$DX")
echo
for m in word deck grid; do
  echo "$m: ${pass[$m]:-0} de ${tot[$m]:-0} sem divergência (FAIL: ${fail[$m]:-0})"
done
exit $(( fails > 0 ))
