#!/bin/sh
# The dedicated server and the bots of the multiplayer check. Needs OUT, PORT, SRV and BOT.

# Stops the run unless the UDP port is free and is not the port of the public server.
free_port() {
  if [ "$PORT" = "24650" ]; then
    echo "port 24650 is the public server: pick another with MP_PORT" >&2
    return 1
  fi
  if [ -n "$(ss -H -uln "sport = :$PORT" 2>/dev/null)" ]; then
    echo "UDP port $PORT is already in use: pick another with MP_PORT" >&2
    return 1
  fi
}

# Starts the server on a data folder and waits for its banner: server_start LOGNAME DIR [options].
server_start() {
  SLOG="$OUT/$1.log"
  SDIR="$2"
  shift 2
  mkdir -p "$SDIR"
  "$SRV" --port "$PORT" --data "$SDIR" --name check "$@" >"$SLOG" 2>&1 &
  SPID=$!
  track "$SPID"
  wait_for "$SLOG" "^Listening|Listening " 30 && return 0
  verdict FAIL "server $1 starts" "no banner in $SLOG"
  return 1
}

# Stops the server with the signal a service manager sends and waits for it to save.
server_stop() {
  kill -TERM "$SPID" 2>/dev/null
  wait_for "$SLOG" "Saved, bye" 40
  SAVED=$?
  wait_exit "$SPID" 10
  untrack "$SPID"
  return "$SAVED"
}

# Prints the key line of the banner of a server log, without the label and the explanation.
banner_key() {
  plain "$1" | grep -a "^Key" | tail -1 | awk '{print $2}'
}

# Prints the seed of the world named in the banner of a server log.
banner_seed() {
  plain "$1" | sed -n 's/^World .*(seed \([0-9]*\)).*/\1/p' | tail -1
}

# Prints the identity key of a game, as 64 hexadecimal digits: key_hex ROLE.
key_hex() {
  od -An -tx1 -v "$OUT/$1/config/abstractions/identity.key" | tr -d ' \n'
}
