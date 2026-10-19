#!/usr/bin/env bash
# Build e publicação local dos pacotes npm. A versão vem sempre do Cargo.toml ([workspace.package]);
# quem a altera é o versioner. Só o comando `all` o chama (rodar `all` é o pedido do dono); os demais nunca mudam a versão.
#
# Uso:
#   scripts/release.sh build [--fast]          compila o binário desta máquina e monta outputs/packages
#   scripts/release.sh publish [--otp 123456] [--dry-run]
#   scripts/release.sh status                  mostra versão local, login no npm e o que já está publicado
#   scripts/release.sh all [--from ETAPA] [--yes] [--otp 123456]
#                                              release inteiro, uma etapa por vez, pedindo confirmação entre elas:
#                                              versao, linux, build, dry-run, publish (--from retoma de uma etapa)
#
# Binários de outra plataforma (ex.: o do Linux, feito por scripts/build-linux.sh no Docker, ou no CI) entram por variável:
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

confirm() { # $1 = pergunta; --yes responde sim a tudo
  [[ $ASSUME_YES -eq 1 ]] && return 0
  local r
  read -r -p "$1 [s/N] " r
  [[ "$r" =~ ^[sSyY]$ ]]
}

step_versao() {
  [[ -z "$(git status --porcelain)" ]] || { echo "a árvore tem mudanças não commitadas; commite antes de lançar (o binário sairia com elas)" >&2; return 1; }
  command -v versioner >/dev/null || { echo "versioner não encontrado no PATH" >&2; return 1; }
  local calc; calc="$(versioner calc)"
  echo "$calc"
  if grep -q "Nenhum incremento" <<<"$calc"; then
    echo "nada a versionar; a versão segue $(version)"
    confirm "Continuar o release com a versão atual $(version)?" || return 1
    return 0
  fi
  confirm "Criar o commit de release com o versioner?" || return 1
  versioner commit
  cargo update --workspace
  if ! git diff --quiet -- Cargo.lock; then
    git add Cargo.lock
    git commit -m "chore(deps): sincroniza o Cargo.lock com a versão $(version)"
  fi
  echo "versão agora: $(version)"
}

step_linux() { scripts/build-linux.sh; }

step_build() {
  [[ -x "$RAIZ/outputs/linux/thumbcraft" ]] || { echo "falta outputs/linux/thumbcraft (etapa 'linux')" >&2; return 1; }
  LINUX_BIN="$RAIZ/outputs/linux/thumbcraft" cmd_build
}

step_dry_run() { cmd_publish --dry-run; }

step_publish() {
  local flags=()
  [[ -n "$OTP" ]] && flags+=(--otp "$OTP")
  cmd_publish "${flags[@]}"
}

cmd_all() {
  local from="versao"
  OTP=""; ASSUME_YES=0
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --from) from="${2:?--from pede a etapa}"; shift 2 ;;
      --yes) ASSUME_YES=1; shift ;;
      --otp) OTP="${2:?--otp pede o código}"; shift 2 ;;
      *) echo "opção desconhecida: $1" >&2; exit 1 ;;
    esac
  done
  local etapas=(versao linux build dry-run publish) n idx=-1 i
  for n in "${!etapas[@]}"; do [[ "${etapas[$n]}" == "$from" ]] && idx=$n; done
  [[ $idx -ge 0 ]] || { echo "etapa desconhecida: $from (${etapas[*]})" >&2; exit 1; }
  for ((n = idx; n < ${#etapas[@]}; n++)); do
    i="${etapas[$n]}"
    echo
    echo "######## etapa: $i"
    # Antes de publicar de verdade, mostra o estado e pede confirmação.
    if [[ "$i" == "publish" ]]; then
      cmd_status
      confirm "Publicar $(version) no npm agora?" || { echo "parado antes de publicar; retome com: scripts/release.sh all --from publish"; return 1; }
    fi
    if ! "step_${i//-/_}"; then
      echo "etapa '$i' não concluída; corrija e retome com: scripts/release.sh all --from $i" >&2
      return 1
    fi
    if [[ $n -lt $((${#etapas[@]} - 1)) ]]; then
      confirm "Etapa '$i' ok. Seguir para '${etapas[$((n + 1))]}'?" || { echo "parado; retome com: scripts/release.sh all --from ${etapas[$((n + 1))]}"; return 0; }
    fi
  done
  echo
  echo "release $(version) concluído. Tag e push continuam manuais (o versioner não cria tag)."
}

case "${1:-}" in
  build) shift; cmd_build "$@" ;;
  publish) shift; cmd_publish "$@" ;;
  status) cmd_status ;;
  all) shift; cmd_all "$@" ;;
  *) sed -n '2,17p' "$0"; exit 1 ;;
esac
