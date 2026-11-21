#!/bin/sh
# End-to-end multiplayer check on real processes: the dedicated server, bots and two real game clients.
# Usage (from the worktree): scripts/mp-e2e.sh [phase ...]    Phases, in the order "all" runs them:
#   proto    refusals, same key twice, full server, kill -STOP; bots only; about 3 minutes
#   persist  records, restart, damaged place; bots only; about 2 minutes
#   game     two clients meet, act and share a stone, 10 minutes of script after the loading; 25 to 60 minutes
#   rejoin   the world of game restarted, player b joins again; 10 to 25 minutes (needs game)
#   edit     a ship saved inside the planet and an airborne place; 15 to 30 minutes (needs game)
#   soak     8 bots for 10 minutes, then a save and a reload; about 12 minutes
#   quick    proto and persist, about 5 minutes and no graphics
# Build first, with CARGO_TARGET_DIR set:
#   cargo build --features dev --example probe && cargo build -p abstractions-server &&
#   cargo build -p abstractions-net --example bot
# Environment: MP_OUT (folder of logs and shots), MP_PORT (UDP, default 24733, never 24650), MP_DISPLAY_A and
# MP_DISPLAY_B (virtual displays 106 and 107), MP_LP_THREADS (renderer threads per client, default 3, so two clients
# and the bots stay near 8 cores), MP_CPUS_A and MP_CPUS_B (taskset core lists), MP_READY_WAIT (seconds to load, 2400).
# It starts only processes of its own, stops only their pids, and never touches docker or the public server.
set -u
: "${CARGO_TARGET_DIR:?set CARGO_TARGET_DIR}"
DIR=$(cd "$(dirname "$0")" && pwd)
OUT="${MP_OUT:-$HOME/.cache/abstractions-targets/mp-logs/e2e}"
PORT="${MP_PORT:-24733}"
DISPLAY_A="${MP_DISPLAY_A:-106}"
DISPLAY_B="${MP_DISPLAY_B:-107}"
T="$CARGO_TARGET_DIR/debug"
SRV="$T/abstractions-server"
BOT="$T/examples/bot"
PROBE="$T/examples/probe"
ADDR="127.0.0.1:$PORT"
PIDS=""
BOTS=""
SLOG=""
SDIR=""
SPID=""
XVPID=""
CPID=""
GOT=""
mkdir -p "$OUT"

for lib in lib server client proto persist game share gamerun restart soak; do
  . "$DIR/mp/$lib.sh"
done
trap stop_all EXIT
trap 'stop_all; exit 1' INT TERM

# Stops unless the three programs exist and the port is free.
preflight() {
  for bin in "$SRV" "$BOT" "$PROBE"; do
    [ -x "$bin" ] || { echo "missing $bin: build first, see the top of this script" >&2; return 1; }
  done
  free_port
}

# Runs one phase by name and says how long it took.
run_phase() {
  START=$(date +%s)
  echo "=== phase $1"
  "phase_$1"
  stop_all
  echo "=== phase $1 took $(( $(date +%s) - START )) s"
}

# Prints every verdict again, then the count, and returns 1 when one failed.
summary() {
  echo
  echo "=== summary ($OUT/results.txt)"
  awk -F'|' '{ printf "%-5s %s\n", $1, $2 }' "$OUT/results.txt"
  FAILS=$(grep -c '^FAIL|' "$OUT/results.txt")
  PASSES=$(grep -c '^PASS|' "$OUT/results.txt")
  echo "$PASSES passed, $FAILS failed, $(grep -c '^SKIP|' "$OUT/results.txt") skipped"
  [ "$FAILS" = 0 ]
}

# Writes the verdict that waits for the headlight bit of the ship record to be merged.
placeholder() {
  verdict SKIP "the second client sees the first one's headlight" \
    "placeholder: merge look-ship, write the act in examples/probe/mpact.rs, run PROBE with SMOKE_ONLY=mp-lights"
}

preflight || exit 2
: >"$OUT/results.txt"
PHASES="${*:-all}"
[ "$PHASES" != all ] || PHASES="proto persist game rejoin edit soak"
[ "$PHASES" != quick ] || PHASES="proto persist"
for phase in $PHASES; do
  run_phase "$phase"
done
placeholder
summary
