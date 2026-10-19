#!/bin/bash
cd /tmp/claude-0/-home-claude/37e359fe-23e0-5bd2-965b-70005e330799/scratchpad/combined-analysis/ws
export CARGO_TARGET_DIR=/tmp/claude-0/-home-claude/37e359fe-23e0-5bd2-965b-70005e330799/scratchpad/combined-analysis/target CARGO_PROFILE_RELEASE_OPT_LEVEL=z CARGO_PROFILE_RELEASE_LTO=fat CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_PROFILE_RELEASE_PANIC=abort CARGO_PROFILE_RELEASE_STRIP=symbols
for f in ppt xlsx word,ppt word,ppt,xlsx word; do
  t0=$(date +%s)
  cargo build --release -p all-min --no-default-features --features $f > /tmp/claude-0/-home-claude/37e359fe-23e0-5bd2-965b-70005e330799/scratchpad/combined-analysis/build-${f//,/+}.log 2>&1
  rc=$?
  cp /tmp/claude-0/-home-claude/37e359fe-23e0-5bd2-965b-70005e330799/scratchpad/combined-analysis/target/release/all-min /tmp/claude-0/-home-claude/37e359fe-23e0-5bd2-965b-70005e330799/scratchpad/combined-analysis/bin/all-min-${f//,/+} 2>/dev/null
  echo "$f rc=$rc secs=$(($(date +%s)-t0)) size=$(stat -c %s /tmp/claude-0/-home-claude/37e359fe-23e0-5bd2-965b-70005e330799/scratchpad/combined-analysis/bin/all-min-${f//,/+} 2>/dev/null)" >> /tmp/claude-0/-home-claude/37e359fe-23e0-5bd2-965b-70005e330799/scratchpad/combined-analysis/progress.txt
done
echo DONE >> /tmp/claude-0/-home-claude/37e359fe-23e0-5bd2-965b-70005e330799/scratchpad/combined-analysis/progress.txt
