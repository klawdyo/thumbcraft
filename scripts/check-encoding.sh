#!/usr/bin/env bash
# Verificação de aceitação universal: UTF-8 sem BOM, LF e sem mojibake.
# Uso: check-encoding.sh [arquivos...]
#   Sem argumentos, verifica os arquivos modificados/novos do git
#   (git diff --name-only HEAD + não rastreados).
# Sai com código 1 se encontrar qualquer problema.
set -u

if [ "$#" -gt 0 ]; then
  files=("$@")
else
  mapfile -t files < <( { git diff --name-only HEAD; git ls-files --others --exclude-standard; } | sort -u )
fi

fail=0
for f in "${files[@]}"; do
  [ -f "$f" ] || continue
  # Ignora binários e este próprio script (contém os padrões de mojibake).
  # (a spec de encoding cita os padrões de mojibake como exemplo).
  case "$f" in *check-encoding.sh|*shared/encoding-and-files.md|*rules/encoding-and-files.md) continue ;; esac
  if ! grep -Iq . "$f" 2>/dev/null; then continue; fi

  # 1. BOM no início do arquivo.
  if [ "$(head -c3 "$f" | od -An -tx1 | tr -d ' \n')" = "efbbbf" ]; then
    echo "BOM: $f"; fail=1
  fi
  # 2. CRLF.
  if grep -q $'\r' "$f"; then
    echo "CRLF: $f"; fail=1
  fi
  # 3. Mojibake (UTF-8 lido como Windows-1252).
  if grep -qE 'Ã[£§¡©ª³µº¢]|â€|Â[ºª•·°]|ï»¿|�' "$f"; then
    echo "MOJIBAKE: $f"; grep -nE 'Ã[£§¡©ª³µº¢]|â€|Â[ºª•·°]|ï»¿|�' "$f" | head -3
    fail=1
  fi
done

if [ "$fail" -eq 0 ]; then echo "OK: UTF-8 sem BOM, LF, sem mojibake"; fi
exit "$fail"
