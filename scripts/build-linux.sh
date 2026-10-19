#!/usr/bin/env bash
# Compila o binário Linux x64 (glibc) dentro do Docker, a partir de uma cópia limpa do código.
#
# O repositório é montado só para leitura; a cópia de trabalho, o cache do cargo e o `target` ficam em volumes
# nomeados do Docker (rápido no Windows, sem misturar arquivos). Os motores são baixados de novo dentro do
# container por scripts/fetch-upstream.sh, então a conferência dos hashes roda num ambiente limpo.
# Saída: outputs/linux/thumbcraft
#
# Uso: scripts/build-linux.sh [--test]     (--test roda também cargo test --workspace --locked)
set -euo pipefail

RAIZ="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGEM="${THUMBCRAFT_LINUX_IMAGE:-rust:1.97-bookworm}"   # glibc 2.36: roda em Debian 12, Ubuntu 22.04 e mais novos
TESTAR=0
[[ "${1:-}" == "--test" ]] && TESTAR=1

command -v docker >/dev/null || { echo "docker não encontrado no PATH" >&2; exit 1; }
docker info >/dev/null 2>&1 || { echo "o Docker não está rodando (abra o Docker Desktop)" >&2; exit 1; }

DEST="$RAIZ/outputs/linux"
mkdir -p "$DEST"

# No Git Bash do Windows o caminho do volume não pode ser convertido pelo MSYS.
export MSYS_NO_PATHCONV=1
if command -v cygpath >/dev/null; then
  RAIZ_DOCKER="$(cygpath -m "$RAIZ")"
  DEST_DOCKER="$(cygpath -m "$DEST")"
else
  RAIZ_DOCKER="$RAIZ"
  DEST_DOCKER="$DEST"
fi

echo "== build Linux x64 (glibc) em $IMAGEM"
docker run --rm \
  -e TESTAR="$TESTAR" \
  -v "$RAIZ_DOCKER":/src:ro \
  -v thumbcraft-linux-work:/work \
  -v thumbcraft-linux-cargo:/usr/local/cargo/registry \
  -v thumbcraft-linux-target:/work/outputs \
  -v "$DEST_DOCKER":/out \
  "$IMAGEM" bash -euo pipefail -c '
    # Cópia limpa do código (sem os gerados do Windows); o target fica no volume e sobrevive entre execuções.
    cd /src
    tar --exclude=./third_party --exclude=./outputs --exclude=./.git --exclude=node_modules --exclude=./benchmarks -cf - . | tar -xf - -C /work
    cd /work
    export CARGO_TARGET_DIR=/work/outputs
    scripts/fetch-upstream.sh
    if [ "$TESTAR" = 1 ]; then cargo test --workspace --locked; fi
    cargo build --release --locked -p thumbcraft
    cp outputs/release/thumbcraft /out/thumbcraft
    ls -l /out/thumbcraft
  '
echo "pronto: $DEST/thumbcraft"
