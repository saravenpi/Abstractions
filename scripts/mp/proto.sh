#!/bin/sh
# Phase proto: what the server says to bots, no graphics. Needs the helpers, OUT, PORT, ADDR, BOT.
# One server with a password and two seats. Takes about 3 minutes.

PW="hunter2"

# Returns the first line of a bot log that says why it was sent away.
closed_line() {
  grep -a "closed" "$1" | head -1
}

# A wrong password, a protocol 2 game and a protocol 99 game are refused with plain words.
proto_refusals() {
  D="$OUT/proto"
  "$BOT" "$ADDR" --password wrong --seconds 3 >"$D/wrong.log" 2>&1
  if grep -aq "Wrong password" "$D/wrong.log"; then
    verdict PASS "a wrong password is refused with a plain message" "$(closed_line "$D/wrong.log") [$D/wrong.log]"
  else
    verdict FAIL "a wrong password is refused with a plain message" "see $D/wrong.log"
  fi
  env BOT_PROTOCOL=2 "$BOT" "$ADDR" --password "$PW" --seconds 3 >"$D/old.log" 2>&1
  if grep -aq "This server needs" "$D/old.log"; then
    verdict PASS "a protocol 2 game is refused and told to update" "$(closed_line "$D/old.log") [$D/old.log]"
  else
    verdict FAIL "a protocol 2 game is refused and told to update" "see $D/old.log"
  fi
  env BOT_PROTOCOL=99 "$BOT" "$ADDR" --password "$PW" --seconds 3 >"$D/new.log" 2>&1
  if grep -aq "You need" "$D/new.log"; then
    verdict PASS "a newer game is refused with a plain message" "$(closed_line "$D/new.log") [$D/new.log]"
  else
    verdict FAIL "a newer game is refused with a plain message" "see $D/new.log"
  fi
}

# The right password joins; the same key joining twice closes the older session and the server logs it.
proto_replaced() {
  D="$OUT/proto"
  env BOT_KEY=11 "$BOT" "$ADDR" --password "$PW" --seconds 30 >"$D/first.log" 2>&1 &
  FIRST=$!
  track "$FIRST"
  wait_for "$SLOG" "Bot 1 joined" 20
  env BOT_KEY=11 "$BOT" "$ADDR" --password "$PW" --seconds 6 >"$D/second.log" 2>&1
  sleep 1
  if grep -aq "Done: 1/1" "$D/second.log"; then
    verdict PASS "the right password joins" "$(grep -a 'Done:' "$D/second.log") [$D/second.log]"
  else
    verdict FAIL "the right password joins" "see $D/second.log"
  fi
  if grep -aq "joined this server from another game" "$D/first.log" &&
    grep -aq "A player joined again: the older session was closed" "$SLOG"; then
    verdict PASS "the same key joining twice replaces the old session and the server logs it" \
      "$(closed_line "$D/first.log"); $(grep -a 'joined again' "$SLOG" | head -1) [$SLOG]"
  else
    verdict FAIL "the same key joining twice replaces the old session and the server logs it" \
      "see $D/first.log and $SLOG"
  fi
  wait_exit "$FIRST" 40
}

# A third player is refused when both seats are taken.
proto_full() {
  D="$OUT/proto"
  env BOT_KEY=21 "$BOT" "$ADDR" --password "$PW" --seconds 25 >"$D/hold1.log" 2>&1 &
  track $!
  env BOT_KEY=22 "$BOT" "$ADDR" --password "$PW" --seconds 25 >"$D/hold2.log" 2>&1 &
  track $!
  sleep 5
  env BOT_KEY=23 "$BOT" "$ADDR" --password "$PW" --seconds 4 >"$D/third.log" 2>&1
  if grep -aq "The server is full" "$D/third.log"; then
    verdict PASS "a full server is refused with a plain message" "$(closed_line "$D/third.log") [$D/third.log]"
  else
    verdict FAIL "a full server is refused with a plain message" "see $D/third.log"
  fi
  sleep 25
}

# A bot stopped with kill -STOP loses its seat: a waiting bot gets in between 15 and 60 seconds later.
proto_silent() {
  D="$OUT/proto"
  env BOT_KEY=31 "$BOT" "$ADDR" --password "$PW" --seconds 300 >"$D/frozen.log" 2>&1 &
  FROZEN=$!
  track "$FROZEN"
  env BOT_KEY=32 "$BOT" "$ADDR" --password "$PW" --seconds 300 >"$D/steady.log" 2>&1 &
  track $!
  wait_for "$D/frozen.log" "connected 1/1" 20
  wait_for "$D/steady.log" "connected 1/1" 20
  kill -STOP "$FROZEN"
  START=$(date +%s)
  GOT=""
  while [ -z "$GOT" ] && [ $(( $(date +%s) - START )) -lt 90 ]; do
    env BOT_KEY=33 "$BOT" "$ADDR" --password "$PW" --seconds 3 >"$D/waiting.log" 2>&1 &&
      GOT=$(( $(date +%s) - START ))
    sleep 1
  done
  kill -CONT "$FROZEN"
  proto_silent_verdict
}

# Says whether the seat of the stopped bot was freed in time.
proto_silent_verdict() {
  if [ -n "$GOT" ] && [ "$GOT" -ge 10 ] && [ "$GOT" -le 65 ]; then
    verdict PASS "kill -STOP frees the seat after 15 to 60 s" \
      "a waiting player got in after $GOT s; server log: $(plain "$SLOG" | tail -3 | tr '\n' ';') [$SLOG]"
  else
    verdict FAIL "kill -STOP frees the seat after 15 to 60 s" \
      "seat freed after '${GOT:-never}' s, see $OUT/proto/waiting.log"
  fi
}

# Runs the whole phase on its own server.
phase_proto() {
  mkdir -p "$OUT/proto"
  rm -rf "$OUT/proto-server"
  server_start proto-server "$OUT/proto-server" --max-players 2 --password "$PW" || return 1
  proto_refusals
  proto_replaced
  proto_full
  proto_silent
  stop_all
  if grep -aqi "panick" "$SLOG" "$OUT"/proto/*.log; then
    verdict FAIL "no panic in the protocol phase" "see $SLOG"
  else
    verdict PASS "no panic in the protocol phase" "$SLOG and $OUT/proto/*.log are clean"
  fi
}
