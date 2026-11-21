#!/bin/sh
# End-to-end multiplayer check: a dedicated server, bots, two real game clients (probe, headless).
# Usage (from the worktree): scripts/mp-e2e.sh [out-dir] [lead-seconds]
# Build first, with CARGO_TARGET_DIR set:
#   cargo build --features dev --example probe
#   cargo build -p abstractions-server
#   cargo build -p abstractions-net --example bot
# Displays 106 and 107. Stops only the processes it started, by pid.
set -u
: "${CARGO_TARGET_DIR:?set CARGO_TARGET_DIR}"
OUT="${1:-$HOME/.cache/abstractions-targets/mp-logs/e2e}"
LEAD="${2:-1500}"
PORT=24733
L="$HOME/.cache/abstractions-targets/lvp"
T="$CARGO_TARGET_DIR/debug"
mkdir -p "$OUT"
PIDS=""
stop() { for p in $PIDS; do kill "$p" 2>/dev/null; done; }
trap stop EXIT

rm -rf "$OUT/server" "$OUT/a" "$OUT/b"
mkdir -p "$OUT/server" "$OUT/a/shots" "$OUT/a/data" "$OUT/b/shots" "$OUT/b/data"

"$T/abstractions-server" --port $PORT --data "$OUT/server" --name check >"$OUT/server.log" 2>&1 &
PIDS="$PIDS $!"
sleep 3
head -3 "$OUT/server.log"

"$T/examples/bot" "127.0.0.1:$PORT" --count 2 --seconds 36000 >"$OUT/bots.log" 2>&1 &
PIDS="$PIDS $!"

BOT_PROTOCOL=2 "$T/examples/bot" "127.0.0.1:$PORT" --count 1 --seconds 5 >"$OUT/old-protocol.log" 2>&1
echo "old protocol bot said:"
head -5 "$OUT/old-protocol.log"

T0=$(( $(date +%s) + LEAD ))
echo "$T0" >"$OUT/t0"
run() {
  N=$1; ROLE=$2; D="$OUT/$ROLE"
  Xvfb ":$N" -screen 0 1280x720x24 >/dev/null 2>&1 &
  XV=$!
  sleep 10
  DISPLAY=":$N" \
  VK_ICD_FILENAMES="$L/libs/usr/share/vulkan/icd.d/lvp_icd.json" \
  LD_LIBRARY_PATH="$L/libs/usr/lib/x86_64-linux-gnu:$T/deps:$HOME/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib" \
  ALSA_CONFIG_PATH="$L/asound.conf" XDG_DATA_HOME="$D/data" SMOKE_DIR="$D/shots" \
  PROBE_SOFT=1 SMOKE_SEED=7 SMOKE_ONLY=mp-meet,mp-act,mp-leave \
  PROBE_JOIN="127.0.0.1:$PORT" PROBE_NAME="Player-$ROLE" PROBE_ROLE="$ROLE" PROBE_T0="$T0" \
  "$T/examples/probe" >"$D/log.txt" 2>&1
  echo "client $ROLE exit $?"
  kill "$XV" 2>/dev/null
}
run 106 a &
A=$!
run 107 b &
B=$!
wait $A $B

sleep 2
echo "--- errors"
for f in "$OUT/server.log" "$OUT/bots.log" "$OUT/a/log.txt" "$OUT/b/log.txt"; do
  echo "== $f"
  sed 's/\x1b\[[0-9;]*m//g' "$f" | grep -aiE "panic|failed validation|decode|error|disconnect|refused" | grep -av rodio | head -20
done
echo "--- MP lines"
for r in a b; do
  sed 's/\x1b\[[0-9;]*m//g' "$OUT/$r/log.txt" | grep -a "MP " | cut -c1-260 | awk 'NR%4==1' | head -60
done
ls "$OUT/a/shots" "$OUT/b/shots"
echo "--- server files"
ls -l "$OUT/server"
