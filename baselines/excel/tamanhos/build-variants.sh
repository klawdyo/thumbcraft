#!/usr/bin/env bash
cd "$(dirname "$0")/variants"
export CARGO_TARGET_DIR="$PWD/../target-var"
for v in orig phase1 pr; do
  ( cd $v && s=$(date +%s); cargo build --release --offline 2>&1 | tail -1; echo "$v build $(( $(date +%s) - s )) s" )
  stat -c "$v %s" "$CARGO_TARGET_DIR/release/gc-pdf-$v"
done
echo done
