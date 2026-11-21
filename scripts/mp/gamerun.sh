#!/bin/sh
# Phase game, the running of it: the server, the bots, both games and the end of each. Needs game.sh.

READY_WAIT="${MP_READY_WAIT:-2400}"
ACT_WAIT="${MP_ACT_WAIT:-1500}"

# Starts the server with 4 seats, 2 bots and both games, runs the script and stops it all.
game_run() {
  rm -rf "$OUT/sync" "$OUT/a" "$OUT/b" "$OUT/server"
  mkdir -p "$OUT/sync"
  server_start game "$OUT/server" --max-players 4 || return 1
  "$BOT" "$ADDR" --count 2 --seconds 36000 >"$OUT/bots.log" 2>&1 &
  BOTS=$!
  track "$BOTS"
  wait_for "$SLOG" "Bot 2 joined" 30
  client a "$DISPLAY_A" game mp-meet,mp-act,mp-share,mp-leave || return 1
  PA=$CPID
  XA=$XVPID
  client b "$DISPLAY_B" game mp-meet,mp-act,mp-share,mp-leave || return 1
  PB=$CPID
  XB=$XVPID
  wait_ready "$PA" "$PB" "$READY_WAIT"
  READY=$?
  [ "$READY" = 0 ] || verdict FAIL "both clients reach the world and meet" \
    "status $READY (1 timeout, 2 a game died); server: $(plain "$SLOG" | tail -4 | tr '\n' ';'); $OUT/a/game.log"
  [ "$READY" = 0 ] || return 1
  start_script
  game_full
  wait_exit "$PA" "$ACT_WAIT" && wait_exit "$PB" "$ACT_WAIT"
  game_end "$PA" "$XA" a
  game_end "$PB" "$XB" b
}

# A fifth player is refused while the two games and the two bots hold the 4 seats.
game_full() {
  "$BOT" "$ADDR" --count 1 --seconds 4 >"$OUT/full.log" 2>&1
  if grep -aq "The server is full" "$OUT/full.log"; then
    verdict PASS "a full server is refused with a plain message" \
      "$(grep -a closed "$OUT/full.log" | head -1) [$OUT/full.log]"
  else
    verdict FAIL "a full server is refused with a plain message" "see $OUT/full.log"
  fi
}

# Ends one game: it must have exited by itself and cleanly.
game_end() {
  kill -0 "$1" 2>/dev/null && kill "$1"
  end_client "$1" "$2"
  CODE=$?
  if [ "$CODE" = 0 ]; then
    verdict PASS "client $3 ran its script and left" "exit 0, $OUT/$3/game.log"
  else
    verdict FAIL "client $3 ran its script and left" "exit $CODE, see $OUT/$3/game.log"
  fi
}

# Keeps what a later phase needs: the record of player b as the server holds it, read by a bot with b's key.
game_record() {
  KEYFILE="$OUT/b/config/abstractions/identity.key"
  [ -s "$KEYFILE" ] || return 0
  env BOT_KEY_FILE="$KEYFILE" "$BOT" "$ADDR" --seconds 3 >"$OUT/b-record.log" 2>&1
  grep -a "welcome:" "$OUT/b-record.log" | head -1 >"$OUT/b-record.line"
  if grep -aq "record kept place \[dir" "$OUT/b-record.line"; then
    verdict PASS "the server kept a record of the player who left" \
      "$(cut -c1-160 "$OUT/b-record.line") [$OUT/b-record.log]"
  else
    verdict FAIL "the server kept a record of the player who left" "$(cat "$OUT/b-record.line") [$OUT/b-record.log]"
  fi
}

# Runs the whole phase, then stops the bots and the server, which saves the world.
phase_game() {
  game_run
  game_meet
  game_acts
  game_share
  game_clean
  game_record
  kill "$BOTS" 2>/dev/null
  server_stop || verdict FAIL "the server saves when it is stopped" "no 'Saved, bye' in $SLOG"
  stop_all
}
