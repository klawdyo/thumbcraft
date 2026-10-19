#!/bin/bash
# usage: run.sh ROOT DCBIN   (timeouts 30 s per file; one fresh process per file)
ROOT=$1; BIN=$2
cd "$ROOT"; rm -rf out-rust out-lo; mkdir -p out-rust out-lo; TSV=results.tsv
printf "file\tformat\ttool\texit\tms\tpng_bytes\tstderr\n" > $TSV
PROF=file:///tmp/claude-0/lo-pptx-prof
now() { date +%s%N; }
for f in in/*; do
  b=$(basename "$f"); stem="${b%.*}"; ext="${b##*.}"
  s=$(now); timeout 30 "$BIN" "$f" "out-rust/$stem.png" 1 > /dev/null 2> /tmp/claude-0/err.txt; rc=$?; e=$(now)
  sz=$(stat -c %s "out-rust/$stem.png" 2>/dev/null || echo 0)
  printf "%s\t%s\trust\t%s\t%s\t%s\t%s\n" "$b" "$ext" $rc $(( (e-s)/1000000 )) $sz "$(head -c 160 /tmp/claude-0/err.txt | tr '\n\t' '  ')" >> $TSV
  s=$(now); timeout 30 soffice -env:UserInstallation=$PROF --headless --convert-to png --outdir out-lo "$f" > /tmp/claude-0/lo.txt 2>&1; rc=$?; e=$(now)
  sz=$(stat -c %s "out-lo/$stem.png" 2>/dev/null || echo 0)
  printf "%s\t%s\tlibreoffice\t%s\t%s\t%s\t%s\n" "$b" "$ext" $rc $(( (e-s)/1000000 )) $sz "$(head -c 160 /tmp/claude-0/lo.txt | tr '\n\t' '  ')" >> $TSV
  pkill -9 -f lo-pptx-prof 2>/dev/null
done
