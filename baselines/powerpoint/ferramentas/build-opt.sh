#!/bin/bash
# usage: build-opt.sh ROOT min|cli
R=$1
export CARGO_PROFILE_RELEASE_OPT_LEVEL=z CARGO_PROFILE_RELEASE_LTO=fat CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_PROFILE_RELEASE_PANIC=abort CARGO_PROFILE_RELEASE_STRIP=symbols
s=$(date +%s)
if [ "$2" = min ]; then
  cd $R/dc-min; CARGO_TARGET_DIR=$R/target-min cargo build --release
else
  cd $R/repo/deckcraft; CARGO_TARGET_DIR=$R/target-opt-cli cargo build --release --locked -p deckcraft-cli
fi
echo "exit=$? seconds=$(( $(date +%s)-s ))"
