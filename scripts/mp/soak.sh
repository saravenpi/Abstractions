#!/bin/sh
# Phase soak: 8 bots for 10 minutes on a 16 seat server, then a save and a reload. No graphics.
# Needs the helpers, OUT, PORT, ADDR and BOT. Takes about 12 minutes. Eight bots is the most one IP address may open.

SOAK_SECONDS="${MP_SOAK_SECONDS:-600}"

# Prints the resident memory of a process in megabytes.
rss_mb() {
  awk '/VmRSS/ {print int($2 / 1024)}' "/proc/$1/status" 2>/dev/null
}

# Runs the bots and samples the server every minute.
soak_run() {
  "$BOT" "$ADDR" --count 8 --seconds "$SOAK_SECONDS" >"$OUT/soak-bots.log" 2>&1 &
  BOTS=$!
  track "$BOTS"
  RSS0=$(rss_mb "$SPID")
  while kill -0 "$BOTS" 2>/dev/null; do
    sleep 60
    printf '%s s: server %s MB, %s\n' "$(( $(date +%s) - SOAK0 ))" "$(rss_mb "$SPID")" \
      "$(plain "$OUT/soak-bots.log" | grep -a connected | tail -1)" >>"$OUT/soak-samples.txt"
  done
  RSS1=$(rss_mb "$SPID")
}

# Says whether the bots stayed in and nothing panicked.
soak_verdict() {
  LAST=$(grep -a "Done:" "$OUT/soak-bots.log")
  if grep -aqi "panick" "$SLOG" "$OUT/soak-bots.log"; then
    verdict FAIL "8 bots for $SOAK_SECONDS s with no panic" "a panic is in $SLOG or $OUT/soak-bots.log"
  elif [ "$LAST" = "Done: 8/8 connected at the end" ]; then
    verdict PASS "8 bots for $SOAK_SECONDS s with no panic" \
      "$LAST; server memory $RSS0 MB at the start, $RSS1 MB at the end; samples in $OUT/soak-samples.txt"
  else
    verdict FAIL "8 bots for $SOAK_SECONDS s with no panic" "${LAST:-no Done line}, see $OUT/soak-bots.log"
  fi
}

# The world is saved when the last player leaves, and loads again with the same seed and key.
soak_reload() {
  KEY1=$(banner_key "$SLOG")
  SEED1=$(banner_seed "$SLOG")
  wait_for "$SLOG" "left \(0/16\)" 30
  server_stop || verdict FAIL "the world is saved after the soak" "no 'Saved, bye' in $SLOG"
  SIZE=$(wc -c <"$OUT/soak-data/world.toml")
  server_start soak-reload "$OUT/soak-data" --max-players 16 || return 1
  if [ "$SIZE" -gt 0 ] && [ "$(banner_key "$SLOG")" = "$KEY1" ] && [ "$(banner_seed "$SLOG")" = "$SEED1" ]; then
    verdict PASS "the soaked world saves and reloads" "world.toml $SIZE bytes, seed $SEED1, key $KEY1 [$SLOG]"
  else
    verdict FAIL "the soaked world saves and reloads" "see $SLOG"
  fi
  server_stop
}

# Runs the whole phase.
phase_soak() {
  rm -rf "$OUT/soak-data" "$OUT/soak-samples.txt"
  server_start soak "$OUT/soak-data" --max-players 16 || return 1
  SOAK0=$(date +%s)
  soak_run
  soak_verdict
  soak_reload
}
