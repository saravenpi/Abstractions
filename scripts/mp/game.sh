#!/bin/sh
# Phase game, the checks: what each of two real clients logged and shot of the other.
# The phases of the script are 0 and 1 crouch, 2 and 3 walk, 4 and 5 board, 6 and 7 throttle; a acts in the
# even ones and b in the odd ones. Takes 25 to 60 minutes with gamerun.sh: the loading on the CPU renderer,
# then 8 phases of 60 seconds.

# Prints the lines the viewer VIEWER logged in the phases PHASES while ACTS said who acts.
phase_lines() {
  plain "$OUT/$1/game.log" | grep -a "MP phase $2 " | grep -a "acts $3"
}

# Prints the highest value that follows the word FIELD: best VIEWER PHASES ACTS FIELD.
best() {
  phase_lines "$1" "$2" "$3" |
    awk -v f="$4" '{ for (i = 1; i < NF; i++) if ($i == f && $(i+1) + 0 > m) m = $(i+1) + 0 } END { print m + 0 }'
}

# Prints how many lines of the viewer show the word WORD.
seen() {
  phase_lines "$1" "$2" "$3" | grep -ac "$4"
}

# Checks that each game saw the other and took the shot of it.
game_meet() {
  for r in a b; do
    LINE=$(plain "$OUT/$r/game.log" | grep -a "MP meet frame.*remote [0-9.]* m away" | tail -1)
    METRES=$(echo "$LINE" | sed -n 's/.*remote \([0-9.]*\) m away.*/\1/p')
    SHOT="$OUT/$r/shots-game/20-mp-meet-$r.png"
    if [ -n "$METRES" ] && [ -s "$SHOT" ] && awk -v m="${METRES:-99}" 'BEGIN { exit !(m < 12) }'; then
      verdict PASS "client $r sees the other player" "$METRES m away; shot $SHOT; $(echo "$LINE" | cut -c1-110)"
    else
      verdict FAIL "client $r sees the other player" "no sighting within 12 m in $OUT/$r/game.log"
    fi
  done
}

# Checks what each viewer saw of the other's crouch, walk, boarding and throttle, and the canopy of the actor.
game_acts() {
  for v in a b; do
    o=$([ "$v" = a ] && echo b || echo a)
    C=$(best "$v" "[01]" false crouch)
    W=$(best "$v" "[23]" false speed)
    S=$(seen "$v" "[45]" false "mode Seated")
    IDLE=$(best "$v" "[45]" false exhaust)
    EX=$(best "$v" "[67]" false exhaust)
    OPEN=$(best "$o" "[45]" true canopy)
    SAT=$(seen "$o" "[45]" true "me InShip")
    check "$v sees $o crouch" "crouch $C" "$C" 0.5 "$OUT/$v/shots-game/3[01]-*-$v-sees.png"
    check "$v sees $o walk" "speed $W m/s" "$W" 1.0 "$OUT/$v/shots-game/3[23]-*-$v-sees.png"
    check "$v sees $o board the ship" "$S log lines with mode Seated" "$S" 0 "$OUT/$v/shots-game/3[45]-*-$v-sees.png"
    check "the canopy of $o's ship lifts at the hatch" "canopy $OPEN rad, $SAT lines seated" "$OPEN" 0.2 \
      "$OUT/$o/shots-game/3[45]-0-$o-acts.png"
    check "$v sees the exhaust of $o's ship with throttle" "exhaust $EX against $IDLE seated" "$EX" 0.15 \
      "$OUT/$v/shots-game/3[67]-*-$v-sees.png"
  done
}

# Says PASS when VALUE is above THRESHOLD and a shot exists: check NAME EVIDENCE VALUE THRESHOLD SHOTGLOB.
check() {
  SHOT=$(ls $5 2>/dev/null | head -1)
  if awk -v v="$3" -v t="$4" 'BEGIN { exit !(v > t) }' && [ -n "$SHOT" ]; then
    verdict PASS "$1" "$2; shot $SHOT"
  else
    verdict FAIL "$1" "$2 (needs more than $4); shots $5; logs $OUT/a/game.log $OUT/b/game.log"
  fi
}

# Checks that the two games did not share a key, that nobody was replaced or lost and that nothing panicked.
game_clean() {
  BAD="another game\|stopped hearing\|panicked"
  if [ "$(key_hex a)" != "$(key_hex b)" ] && ! grep -aqi "$BAD" "$OUT/a/game.log" "$OUT/b/game.log"; then
    verdict PASS "the two clients have their own keys and nobody was replaced" \
      "a $(key_hex a | cut -c1-8) b $(key_hex b | cut -c1-8) [$OUT/a/config]"
  else
    verdict FAIL "the two clients have their own keys and nobody was replaced" "see $OUT/a/game.log and $OUT/b/game.log"
  fi
  if grep -aqi "panicked" "$SLOG" "$OUT/bots.log"; then
    verdict FAIL "no panic on the server or in the bots" "see $SLOG"
  else
    verdict PASS "no panic on the server or in the bots" "$SLOG and $OUT/bots.log are clean"
  fi
}

