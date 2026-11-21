#!/bin/sh
# Helpers shared by every phase of the multiplayer check: verdicts, processes and waits.
# Sourced by scripts/mp-e2e.sh. Needs OUT. Never kills a process it did not start.

# Records and prints one verdict: verdict STATE NAME EVIDENCE.
verdict() {
  printf '%s|%s|%s\n' "$1" "$2" "$3" >>"$OUT/results.txt"
  printf '%-5s %s\n      %s\n' "$1" "$2" "$3"
}

# Remembers a process id this script started, so only it is stopped at the end.
track() {
  PIDS="$PIDS $1"
}

# Forgets a process id that ended.
untrack() {
  KEEP=""
  for p in $PIDS; do
    [ "$p" = "$1" ] || KEEP="$KEEP $p"
  done
  PIDS="$KEEP"
}

# Stops every tracked process, resuming the ones that were stopped first.
stop_all() {
  for p in $PIDS; do
    kill -CONT "$p" 2>/dev/null
    kill "$p" 2>/dev/null
  done
  PIDS=""
}

# Waits until a regular expression is in a file: wait_for FILE PATTERN SECONDS. Returns 1 on timeout.
wait_for() {
  END=$(( $(date +%s) + $3 ))
  while [ "$(date +%s)" -lt "$END" ]; do
    grep -aqE "$2" "$1" 2>/dev/null && return 0
    sleep 1
  done
  return 1
}

# Waits until a process ended: wait_exit PID SECONDS. Returns 1 on timeout.
wait_exit() {
  END=$(( $(date +%s) + $2 ))
  while kill -0 "$1" 2>/dev/null; do
    [ "$(date +%s)" -lt "$END" ] || return 1
    sleep 2
  done
  return 0
}

# Prints a log without its colour codes.
plain() {
  sed 's/\x1b\[[0-9;]*m//g' "$1" 2>/dev/null
}
