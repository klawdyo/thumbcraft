#!/usr/bin/env bash
# Compila o thumbcraft em release (perfil de tamanho). Busca os upstream antes, se preciso.
# Uso: scripts/build.sh            (CARGO_TARGET_DIR opcional)
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/env.sh"
cd "$RAIZ"
"$RAIZ/scripts/fetch-upstream.sh"
t0=$(date +%s)
cargo build --release --locked -p thumbcraft
t1=$(date +%s)
bytes=$(stat -c %s "$TC_BIN")
echo "binário: $TC_BIN"
echo "tamanho: $bytes bytes ($(awk -v b="$bytes" 'BEGIN{printf "%.2f MB decimais, %.2f MiB", b/1e6, b/1048576}'))"
echo "tempo de build: $((t1 - t0)) s"
