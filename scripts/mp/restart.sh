#!/bin/sh
# Phases rejoin and edit: the world the game phase left in OUT/server, started again and joined by real clients.
# Needs the helpers, OUT, PORT, ADDR, BOT, DIR, DISPLAY_A, DISPLAY_B and what phase game left.
# rejoin takes 10 to 25 minutes, edit 15 to 30: the loading of the games on the CPU renderer.

ACT_WAIT="${MP_ACT_WAIT:-1500}"

# Prints the three numbers after "pos" in a spot line.
spot_pos() {
  echo "$1" | sed -n 's/.*pos \(-\{0,1\}[0-9.]*\) \(-\{0,1\}[0-9.]*\) \(-\{0,1\}[0-9.]*\) altitude.*/\1 \2 \3/p'
}

# Runs one client to its end: run_one ROLE DISPLAY TAG STAGES.
run_one() {
  client "$1" "$2" "$3" "$4" || return 1
  wait_exit "$CPID" "$ACT_WAIT"
  kill -0 "$CPID" 2>/dev/null && kill "$CPID"
  end_client "$CPID" "$XVPID"
}

# The server started again keeps the key, the world and the record of the player who left.
rejoin_restart() {
  server_start rejoin "$OUT/server" --max-players 4 || return 1
  KEY1=$(banner_key "$OUT/game.log")
  SEED1=$(banner_seed "$OUT/game.log")
  env BOT_KEY_FILE="$OUT/b/config/abstractions/identity.key" "$BOT" "$ADDR" --seconds 3 >"$OUT/b-record2.log" 2>&1
  LINE2=$(grep -a "welcome:" "$OUT/b-record2.log" | head -1)
  if [ -n "$KEY1" ] && [ "$KEY1" = "$(banner_key "$SLOG")" ] && [ "$SEED1" = "$(banner_seed "$SLOG")" ] &&
    [ "$LINE2" = "$(cat "$OUT/b-record.line")" ]; then
    verdict PASS "a restart keeps the world, the server key and the players' places" \
      "banner key $KEY1 seed $SEED1 unchanged; $(echo "$LINE2" | cut -c1-120) [$SLOG]"
  else
    verdict FAIL "a restart keeps the world, the server key and the players' places" \
      "key $KEY1/$(banner_key "$SLOG") seed $SEED1/$(banner_seed "$SLOG"); $LINE2"
  fi
}

# The real game of player b joins again and stands where it left.
rejoin_client() {
  run_one b "$DISPLAY_B" rejoin mp-spot,mp-leave
  CODE=$?
  BEFORE=$(spot_pos "$(plain "$OUT/b/game.log" | grep -a 'MP spot before leaving' | tail -1)")
  AFTER=$(spot_pos "$(plain "$OUT/b/rejoin.log" | grep -a 'MP spot after joining' | tail -1)")
  if [ -n "$BEFORE" ] && [ -n "$AFTER" ] && echo "$BEFORE $AFTER" |
    awk '{ d = sqrt(($1-$4)^2 + ($2-$5)^2 + ($3-$6)^2); exit !(d < 3) }'; then
    verdict PASS "leave, restart and rejoin put the player back at the same place" \
      "left at $BEFORE, back at $AFTER (within 3 m); shot $OUT/b/shots-rejoin/41-mp-spot-b.png; exit $CODE"
  else
    verdict FAIL "leave, restart and rejoin put the player back at the same place" \
      "left at '$BEFORE', back at '$AFTER'; see $OUT/b/rejoin.log"
  fi
}

# Runs the phase rejoin.
phase_rejoin() {
  [ -s "$OUT/b-record.line" ] || { verdict SKIP "rejoin" "run the game phase first"; return 0; }
  rejoin_restart || return 1
  rejoin_client
  server_stop || verdict FAIL "the server saves when it is stopped" "no 'Saved, bye' in $SLOG"
}

# Edits the stored records while the server is stopped: a ship inside the planet and a place in the air.
edit_world() {
  KEYA=$(key_hex a)
  KEYB=$(key_hex b)
  python3 -I "$DIR/mp/world_edit.py" "$OUT/server/world.toml" --key "$KEYB" ship inside || return 1
  python3 -I "$DIR/mp/world_edit.py" "$OUT/server/world.toml" --key "$KEYA" place air:3000 || return 1
}

# Says what became of the edited records, from the spot lines of both games.
edit_verdicts() {
  LB=$(plain "$OUT/b/edit.log" | grep -a "MP spot after joining" | tail -1)
  LA=$(plain "$OUT/a/edit.log" | grep -a "MP spot after joining" | tail -1)
  if echo "$LB" | grep -aq "travel Some(InShip).*frame System"; then
    verdict PASS "a ship saved inside the planet comes back in orbit" \
      "$(echo "$LB" | cut -c1-150); shot $OUT/b/shots-edit/41-mp-spot-b.png shows the message about the orbit"
  else
    verdict FAIL "a ship saved inside the planet comes back in orbit" "'$LB', see $OUT/b/edit.log"
  fi
  HIGH=$(echo "$LA" | sed -n 's/.*altitude \(-\{0,1\}[0-9.]*\).*/\1/p')
  if awk -v h="${HIGH:--1}" 'BEGIN { exit !(h > 500) }'; then
    verdict PASS "an airborne on-foot place loads instead of hanging" \
      "$(echo "$LA" | cut -c1-150); shot $OUT/a/shots-edit/41-mp-spot-a.png"
  else
    verdict FAIL "an airborne on-foot place loads instead of hanging" "'$LA', see $OUT/a/edit.log"
  fi
}

# Runs the phase edit: both games join the edited world together.
phase_edit() {
  [ -s "$OUT/server/world.toml" ] || { verdict SKIP "edit" "run the game phase first"; return 0; }
  edit_world || { verdict FAIL "the stored records can be edited" "see $DIR/mp/world_edit.py"; return 1; }
  server_start edit "$OUT/server" --max-players 4 || return 1
  client a "$DISPLAY_A" edit mp-spot,mp-leave || return 1
  PA=$CPID
  XA=$XVPID
  client b "$DISPLAY_B" edit mp-spot,mp-leave || return 1
  wait_exit "$PA" "$ACT_WAIT"
  wait_exit "$CPID" "$ACT_WAIT"
  end_client "$CPID" "$XVPID"
  end_client "$PA" "$XA"
  edit_verdicts
  server_stop
}
