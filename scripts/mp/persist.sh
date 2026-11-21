#!/bin/sh
# Phase persist: what the server keeps of a player and of the world, checked with bots. No graphics.
# Needs the helpers, OUT, PORT, ADDR, BOT and DIR. Takes about 2 minutes.

# Writes the place and the ship the bot sends: a sound place, and a place whose height is not a number.
persist_files() {
  P="$OUT/persist"
  mkdir -p "$P"
  printf 'dir = [0.6, 0.0, -0.8]\naltitude = 2.0\n\n[body]\nindex = 0\nmoon = 0\n\n' >"$P/place.txt"
  printf '[body.star]\ngalaxy = [0, 0, 0]\nsector = [35, 0, -44]\nindex = 0\n' >>"$P/place.txt"
  sed 's/altitude = 2.0/altitude = nan/' "$P/place.txt" >"$P/place-nan.txt"
  printf 'seated = false\nlanded = true\n' >"$P/ship.txt"
}

# Runs one bot of key number KEY for 5 seconds into the log NAME, sending the record of the files PLACE and SHIP.
persist_bot() {
  env BOT_KEY="$1" BOT_PLACE_FILE="$P/$3" BOT_SHIP_FILE="$P/ship.txt" \
    "$BOT" "$ADDR" --seconds 5 >"$P/$2.log" 2>&1
}

# Prints the record line a bot log says it was welcomed with.
kept_line() {
  grep -a "welcome:" "$P/$1.log" | head -1
}

# The server keeps the record of a player who left, and gives it back on the next join.
persist_rejoin() {
  persist_bot 41 first place.txt
  persist_bot 41 second place.txt
  if kept_line first | grep -aq "no record kept" && kept_line second | grep -aq "altitude = 2.0"; then
    verdict PASS "leave and rejoin gives the player's record back" "$(kept_line second) [$P/second.log]"
  else
    verdict FAIL "leave and rejoin gives the player's record back" "$(kept_line first) / $(kept_line second)"
  fi
}

# A restart keeps the world (seed), the server key and the records.
persist_restart() {
  KEY1=$(banner_key "$SLOG")
  SEED1=$(banner_seed "$SLOG")
  CERT1=$(sha256sum "$SDIR/cert.der" | cut -c1-16)
  server_stop || verdict FAIL "the server saves when it is stopped" "no 'Saved, bye' in $SLOG"
  server_start persist-server2 "$SDIR" --max-players 4 || return 1
  persist_bot 41 third place.txt
  KEY2=$(banner_key "$SLOG")
  SEED2=$(banner_seed "$SLOG")
  CERT2=$(sha256sum "$SDIR/cert.der" | cut -c1-16)
  if [ -n "$KEY1" ] && [ "$KEY1" = "$KEY2" ] && [ "$SEED1" = "$SEED2" ] && [ "$CERT1" = "$CERT2" ] &&
    [ "$(kept_line second)" = "$(kept_line third)" ]; then
    verdict PASS "a restart keeps the world, the server key and the player's record" \
      "key $KEY1 seed $SEED1 cert $CERT1 unchanged; $(kept_line third) [$SLOG]"
  else
    verdict FAIL "a restart keeps the world, the server key and the player's record" \
      "key $KEY1/$KEY2 seed $SEED1/$SEED2 cert $CERT1/$CERT2; $(kept_line third)"
  fi
}

# A stored place with a height that is not a number is repaired when the file is loaded.
persist_damaged() {
  server_stop
  python3 -I "$DIR/mp/world_edit.py" "$SDIR/world.toml" --name "Bot 1" place nan || return 1
  server_start persist-server3 "$SDIR" --max-players 4 || return 1
  persist_bot 41 fourth place.txt
  if grep -aq "The stored place of Bot 1 is not valid" "$SLOG" && kept_line fourth | grep -aq -- "-20000"; then
    verdict PASS "a damaged stored place is repaired when the world loads" \
      "$(grep -a 'stored place' "$SLOG" | head -1 | cut -c1-120); $(kept_line fourth) [$SLOG]"
  else
    verdict FAIL "a damaged stored place is repaired when the world loads" "see $SLOG and $P/fourth.log"
  fi
}

# A damaged place sent by a game is repaired when the server keeps it.
persist_sent() {
  persist_bot 42 fifth place-nan.txt
  persist_bot 42 sixth place.txt
  if kept_line sixth | grep -aq -- "-20000"; then
    verdict PASS "a damaged place sent by a game is repaired when the server keeps it" \
      "$(kept_line sixth) [$P/sixth.log]"
  else
    verdict FAIL "a damaged place sent by a game is repaired when the server keeps it" "$(kept_line sixth)"
  fi
}

# Runs the whole phase on its own server and its own world.
phase_persist() {
  persist_files
  rm -rf "$OUT/persist-data"
  server_start persist-server "$OUT/persist-data" --max-players 4 || return 1
  persist_rejoin
  persist_restart || return 1
  persist_damaged || return 1
  persist_sent
  server_stop
}
