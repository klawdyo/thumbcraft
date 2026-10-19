#!/bin/bash
cd "$1/repo/deckcraft"
s=$(date +%s)
CARGO_TARGET_DIR="$1/target-default" cargo build --release --locked -p deckcraft-cli
echo "exit=$? seconds=$(( $(date +%s)-s ))"
