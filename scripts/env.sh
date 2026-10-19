#!/usr/bin/env bash
# Variáveis comuns dos scripts (para fazer `source`). Tudo pode ser sobrescrito pelo ambiente.
RAIZ="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Pasta de compilação fora do repositório (o target/ não é versionado).
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$RAIZ/target}"
# Binário gerado por scripts/build.sh.
TC_BIN="${TC_BIN:-$CARGO_TARGET_DIR/release/thumbcraft}"
