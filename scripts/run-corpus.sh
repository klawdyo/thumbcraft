#!/usr/bin/env bash
# Roda o thumbcraft em todos os arquivos de um diretório.
# Uso: scripts/run-corpus.sh DIR_ENTRADA DIR_SAÍDA [opções do thumbcraft...]
#   RECURSIVO=1 inclui subpastas; TIMEOUT=30 muda o limite por arquivo (s).
# Saída: DIR_SAÍDA/<arquivo>.png e DIR_SAÍDA/results.tsv
#   (arquivo, motor, código de saída, ms, bytes do PNG, dimensões).
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/env.sh"
IN="${1:?uso: run-corpus.sh DIR_ENTRADA DIR_SAÍDA [opções]}"
OUT="${2:?uso: run-corpus.sh DIR_ENTRADA DIR_SAÍDA [opções]}"
shift 2
TIMEOUT="${TIMEOUT:-30}"
mkdir -p "$OUT"
TSV="$OUT/results.tsv"
printf 'arquivo\tmotor\tsaida\tms\tbytes\tdimensoes\n' > "$TSV"
prof=1; [[ "${RECURSIVO:-0}" == 1 ]] && prof=9
find "$IN" -maxdepth "$prof" -type f ! -name MANIFEST.tsv ! -name 'README*' | sort | while read -r f; do
  rel="${f#"$IN"/}"
  dst="$OUT/$(echo "$rel" | tr '/' '_').png"
  motor="$("$TC_BIN" --detect "$f" 2>/dev/null || true)"
  [[ -z "$motor" ]] && motor="-"
  t0=$(date +%s%N)
  timeout "$TIMEOUT" "$TC_BIN" "$f" "$dst" "$@" 2>/dev/null
  rc=$?
  ms=$(( ($(date +%s%N) - t0) / 1000000 ))
  [[ $rc -eq 124 ]] && rc=124
  bytes=0; dim="-"
  if [[ $rc -eq 0 && -s "$dst" ]]; then
    bytes=$(stat -c %s "$dst")
    dim=$(od -An -tu1 -j16 -N8 "$dst" | awk '{printf "%dx%d", $1*16777216+$2*65536+$3*256+$4, $5*16777216+$6*65536+$7*256+$8}')
  else
    rm -f "$dst"
  fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$rel" "$motor" "$rc" "$ms" "$bytes" "$dim" >> "$TSV"
done
"$(dirname "${BASH_SOURCE[0]}")/summarize.sh" "$TSV"
