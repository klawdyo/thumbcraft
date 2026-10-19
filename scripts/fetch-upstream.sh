#!/usr/bin/env bash
# Baixa os três projetos *craft da organização storytold em SHAs FIXOS para third_party/,
# aplica os patches em patches/<projeto>/ e verifica a árvore resultante.
#
# Idempotente: se o diretório já está no SHA certo e com o patch aplicado, não faz nada.
# Falha alto (código 1) se um SHA não existe mais no remoto ou se a árvore patchada difere.
#
# Uso: scripts/fetch-upstream.sh [--force]
#   --force  apaga third_party/ e recria tudo do zero.
#
# Também copia crates do crates.io que precisam de patch (tabela CRATES, com o sha256 do .crate).
#
# Para atualizar uma versão: edite a tabela abaixo (SHA e data), rode o script e atualize
# o UPSTREAM.md. O hash esperado da árvore patchada do GridCraft muda junto com o SHA ou o patch.

set -euo pipefail

# Sem conversão de fim de linha, em qualquer configuração global do git (os patches e o hash da árvore esperam LF).
export GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.autocrlf GIT_CONFIG_VALUE_0=false

RAIZ="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="$RAIZ/third_party"

# nome|url|sha|data do commit (ISO 8601)
PROJETOS=(
  "wordcraft|https://github.com/storytold/wordcraft|117aea31a8e61bd3e306e9210800cdc42897e3c1|2026-10-19T03:08:47-04:00"
  "gridcraft|https://github.com/storytold/gridcraft|66ce6be4743f4227047f76f56815520f824e6d81|2026-10-18T16:14:33-06:00"
  "deckcraft|https://github.com/storytold/deckcraft|010941ec60f81f1eece31ec1ea83825c9a396217|2026-10-18T12:31:16-06:00"
  "craft-libs|https://github.com/storytold/craft-libs|ae2f796b673f29cf3161f36eb6a8933f0a3fe028|2026-10-18T15:11:21-04:00"
)

# Hash da árvore do GridCraft depois de aplicar patches/gridcraft/*.patch (git write-tree
# de um índice temporário com `git add -A -f`). Prova que o patch reconstrói a cópia do PR.
ARVORE_GRIDCRAFT="0b8e31e398afda27d6c1015cbbb79275edb20d7d"

erro() { echo "erro: $*" >&2; exit 1; }
info() { echo "[fetch-upstream] $*"; }

if [[ "${1:-}" == "--force" ]]; then
  info "removendo $DEST"
  rm -rf "$DEST"
fi
mkdir -p "$DEST"

# Hash de todos os patches de um projeto (para detectar mudança de patch).
hash_patches() {
  local nome="$1"
  local dir="$RAIZ/patches/$nome"
  if compgen -G "$dir/*.patch" > /dev/null; then
    cat "$dir"/*.patch | sha256sum | cut -d' ' -f1
  else
    echo "sem-patches"
  fi
}

# Hash da árvore de trabalho atual, sem tocar no índice real.
hash_arvore() {
  local dir="$1" idx
  idx="$(mktemp)"
  rm -f "$idx"
  (cd "$dir" && GIT_INDEX_FILE="$idx" git add -A -f . && GIT_INDEX_FILE="$idx" git write-tree)
  rm -f "$idx"
}

baixar() {
  local nome="$1" url="$2" sha="$3" dir="$DEST/$1"
  if [[ -d "$dir/.git" ]]; then
    local atual
    atual="$(git -C "$dir" rev-parse HEAD 2>/dev/null || true)"
    if [[ "$atual" == "$sha" ]]; then
      return 0
    fi
    info "$nome: SHA diferente ($atual), recriando"
    rm -rf "$dir"
  fi
  rm -rf "$dir"
  mkdir -p "$dir"
  git -C "$dir" init -q
  git -C "$dir" remote add origin "$url"
  info "$nome: buscando $sha"
  if ! git -C "$dir" fetch -q --depth 1 origin "$sha"; then
    erro "$nome: o commit $sha não pôde ser buscado em $url (foi removido ou reescrito?)"
  fi
  git -C "$dir" checkout -q --detach FETCH_HEAD
  local obtido
  obtido="$(git -C "$dir" rev-parse HEAD)"
  [[ "$obtido" == "$sha" ]] || erro "$nome: HEAD $obtido difere do esperado $sha"
  rm -f "$dir/.git/thumbcraft-patched"
}

aplicar_patches() {
  local nome="$1" dir="$DEST/$1"
  local esperado marca
  esperado="$(hash_patches "$nome")"
  marca="$dir/.git/thumbcraft-patched"
  if [[ "$esperado" == "sem-patches" ]]; then
    return 0
  fi
  if [[ -f "$marca" && "$(cat "$marca")" == "$esperado" ]]; then
    return 0
  fi
  # Estado limpo antes de aplicar (descarta qualquer patch parcial anterior).
  git -C "$dir" reset -q --hard HEAD
  git -C "$dir" clean -q -fdx
  local p
  for p in "$RAIZ/patches/$nome"/*.patch; do
    info "$nome: aplicando $(basename "$p")"
    git -C "$dir" apply --whitespace=nowarn "$p" || erro "$nome: falha ao aplicar $p"
  done
  echo "$esperado" > "$marca"
}

for linha in "${PROJETOS[@]}"; do
  IFS='|' read -r nome url sha data <<< "$linha"
  baixar "$nome" "$url" "$sha"
  aplicar_patches "$nome"
  info "$nome: ok em $sha ($data)"
done

# Crates do crates.io copiados para third_party/ só para receber um patch (nome|versão|sha256 do .crate, o mesmo do
# Cargo.lock). O .crate é baixado, conferido e extraído; um git local guarda o estado limpo para o patch.
CRATES=(
  "glifo|0.3.0|282a26c1e23de04bdab3e34a21b6f877a479b96737792516b9b8b8f69b6661be"
)

baixar_crate() {
  local nome="$1" versao="$2" soma="$3" dir="$DEST/$1"
  if [[ -f "$dir/.git/thumbcraft-crate" && "$(cat "$dir/.git/thumbcraft-crate")" == "$versao $soma" ]]; then
    return 0
  fi
  rm -rf "$dir"
  mkdir -p "$dir"
  local arq
  arq="$(mktemp)"
  info "$nome: baixando $nome-$versao.crate"
  curl -fsSL "https://static.crates.io/crates/$nome/$nome-$versao.crate" -o "$arq" || { rm -f "$arq"; erro "$nome: falha ao baixar $nome-$versao"; }
  local obtida
  obtida="$(sha256sum "$arq" | cut -d' ' -f1)"
  [[ "$obtida" == "$soma" ]] || { rm -f "$arq"; erro "$nome: sha256 $obtida difere do esperado $soma"; }
  tar -xzf "$arq" -C "$dir" --strip-components=1
  rm -f "$arq"
  git -C "$dir" init -q
  git -C "$dir" add -A -f .
  git -C "$dir" -c user.name=thumbcraft -c user.email=thumbcraft@localhost commit -q -m "$nome $versao"
  echo "$versao $soma" > "$dir/.git/thumbcraft-crate"
  rm -f "$dir/.git/thumbcraft-patched"
}

for linha in "${CRATES[@]}"; do
  IFS='|' read -r nome versao soma <<< "$linha"
  baixar_crate "$nome" "$versao" "$soma"
  aplicar_patches "$nome"
  info "$nome: ok na versão $versao"
done

# Verificação da árvore patchada do GridCraft.
obtida="$(hash_arvore "$DEST/gridcraft")"
if [[ "$obtida" != "$ARVORE_GRIDCRAFT" ]]; then
  erro "árvore patchada do gridcraft difere: obtida $obtida, esperada $ARVORE_GRIDCRAFT"
fi
info "gridcraft: árvore patchada confere ($obtida)"
info "concluído"
