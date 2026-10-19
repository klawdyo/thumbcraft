#!/usr/bin/env bash
# Build e publicação local dos pacotes npm. A versão vem sempre do Cargo.toml ([workspace.package]);
# quem a altera é o versioner, sob pedido do dono. Este script nunca muda a versão.
#
# Uso:
#   scripts/release.sh build [--fast]          compila o binário desta máquina e monta outputs/packages
#   scripts/release.sh publish [--otp 123456] [--dry-run]
#   scripts/release.sh status                  mostra versão local, login no npm e o que já está publicado
#
# Binários de outra plataforma (ex.: o do Linux, compilado no WSL ou no CI) entram por variável:
#   LINUX_BIN=/caminho/thumbcraft  WIN_BIN=/caminho/thumbcraft.exe  scripts/release.sh build
set -euo pipefail

RAIZ="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$RAIZ"

PKG_DIR="$RAIZ/packages/thumbcraft"
OUT="$RAIZ/outputs/packages"
# Ordem de publicação: as plataformas primeiro, o principal por último (ele depende delas).
PUBLISH_ORDER=("@thumbcraft/win32-x64-msvc:win32-x64-msvc" "@thumbcraft/linux-x64-gnu:linux-x64-gnu" "thumbcraft:thumbcraft")

version() {
  awk '/^\[workspace.package\]/{f=1;next} /^\[/{f=0} f&&/^version *=/{gsub(/[" ]/,"",$3);print $3;exit}' Cargo.toml
}

target_dir() {
  echo "${CARGO_TARGET_DIR:-$RAIZ/outputs}"
}

cmd_build() {
  local fast=0
  [[ "${1:-}" == "--fast" ]] && fast=1
  local v; v="$(version)"
  echo "== versão do Cargo.toml: $v"

  echo "== motores upstream"
  scripts/fetch-upstream.sh

  if [[ $fast -eq 0 ]]; then
    echo "== testes do núcleo"
    cargo test --locked
  fi
  echo "== compilando o binário (release)"
  cargo build --release --locked -p thumbcraft

  local native_id native_bin
  case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) native_id=win32-x64-msvc; native_bin="$(target_dir)/release/thumbcraft.exe" ;;
    Linux) native_id=linux-x64-gnu; native_bin="$(target_dir)/release/thumbcraft" ;;
    *) echo "sistema sem suporte: $(uname -s)" >&2; exit 1 ;;
  esac

  local args=(--version "$v")
  local win="${WIN_BIN:-}" linux="${LINUX_BIN:-}"
  [[ $native_id == win32-x64-msvc && -z "$win" ]] && win="$native_bin"
  [[ $native_id == linux-x64-gnu && -z "$linux" ]] && linux="$native_bin"
  [[ -n "$win" ]] && args+=(--binary "win32-x64-msvc=$win")
  [[ -n "$linux" ]] && args+=(--binary "linux-x64-gnu=$linux")

  echo "== wrapper TypeScript"
  (
    cd "$PKG_DIR"
    npm ci
    npm run typecheck
    [[ $fast -eq 0 ]] && npm test
    npm run build
    echo "== montando os pacotes em outputs/packages"
    npm run build:packages -- "${args[@]}"
  )
  echo
  echo "pronto: thumbcraft $v em $OUT"
  [[ -z "$win" || -z "$linux" ]] && echo "aviso: falta o binário de uma plataforma (use WIN_BIN ou LINUX_BIN); sem os dois não dá para publicar."
  return 0
}

npm_user() { npm whoami 2>/dev/null || true; }

is_published() { # $1 = nome, $2 = versão
  npm view "$1@$2" version >/dev/null 2>&1
}

cmd_status() {
  local v; v="$(version)"
  echo "versão local (Cargo.toml): $v"
  local user; user="$(npm_user)"
  echo "login no npm: ${user:-NÃO autenticado (rode: npm login)}"
  local entry name
  for entry in "${PUBLISH_ORDER[@]}"; do
    name="${entry%%:*}"
    if is_published "$name" "$v"; then echo "  $name@$v: já publicado"; else echo "  $name@$v: ainda não publicado"; fi
  done
}

cmd_publish() {
  local otp="" dry=0
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --otp) otp="${2:?--otp pede o código}"; shift 2 ;;
      --dry-run) dry=1; shift ;;
      *) echo "opção desconhecida: $1" >&2; exit 1 ;;
    esac
  done
  local v; v="$(version)"

  local entry dir
  for entry in "${PUBLISH_ORDER[@]}"; do
    dir="$OUT/${entry##*:}"
    [[ -f "$dir/package.json" ]] || { echo "falta $dir; rode 'scripts/release.sh build' (com os dois binários)" >&2; exit 1; }
    grep -q "\"version\": \"$v\"" "$dir/package.json" || { echo "$dir não está na versão $v; rode o build de novo" >&2; exit 1; }
  done
  for entry in "${PUBLISH_ORDER[@]}"; do
    [[ "${entry##*:}" == "thumbcraft" ]] && continue
    [[ -n "$(ls -A "$OUT/${entry##*:}/bin" 2>/dev/null)" ]] || { echo "pacote ${entry##*:} sem binário em bin/" >&2; exit 1; }
  done

  if [[ $dry -eq 0 ]]; then
    local user; user="$(npm_user)"
    if [[ -z "$user" ]]; then
      echo "não autenticado no npm; abrindo o login"
      npm login
      user="$(npm_user)"
      [[ -n "$user" ]] || { echo "login falhou" >&2; exit 1; }
    fi
    echo "autenticado como: $user"
  fi

  local name flags
  for entry in "${PUBLISH_ORDER[@]}"; do
    name="${entry%%:*}"; dir="$OUT/${entry##*:}"
    if [[ $dry -eq 0 ]] && is_published "$name" "$v"; then
      echo "== $name@$v já publicado, pulando"
      continue
    fi
    echo "== publicando $name@$v"
    flags=(--access public)
    [[ $dry -eq 1 ]] && flags+=(--dry-run)
    [[ -n "$otp" ]] && flags+=(--otp "$otp")
    if ! npm publish "$dir" "${flags[@]}"; then
      echo "falhou em $name. Se foi OTP expirado, rode de novo com um código novo: já publicados são pulados." >&2
      exit 1
    fi
  done
  echo "concluído"
}

case "${1:-}" in
  build) shift; cmd_build "$@" ;;
  publish) shift; cmd_publish "$@" ;;
  status) cmd_status ;;
  *) sed -n '2,13p' "$0"; exit 1 ;;
esac
