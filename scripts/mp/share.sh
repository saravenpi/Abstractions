#!/bin/sh
# Phase game, the checks of the shared stone: a puts it down, b takes it, a sees it gone (probe stage mp-share).

# Checks the shared stone: a put it down, b saw it and took it into the pack, and a saw it gone.
game_share() {
  A="$OUT/a/game.log"
  B="$OUT/b/game.log"
  share_step "a puts a stone on the ground" "$A" "MP share a sees id [0-9]* lying" \
    "$OUT/a/shots-game/40-mp-share-a-lying.png"
  share_step "b sees the stone a put down" "$B" "MP share b sees id [0-9]* lying" \
    "$OUT/b/shots-game/40-mp-share-b-sees.png"
  share_step "b is offered the take of the stone" "$B" "MP share b is offered the take" \
    "$OUT/b/shots-game/40-mp-share-b-sees.png"
  share_step "b takes the stone with the interact key" "$B" "MP share b took the stone into the pack" \
    "$OUT/b/shots-game/40-mp-share-b.png"
  share_step "a sees the stone gone" "$A" "MP share a sees the stone gone" "$OUT/a/shots-game/40-mp-share-a.png"
}

# Says PASS when the log LOG holds a line matching PATTERN: share_step NAME LOG PATTERN SHOT.
share_step() {
  LINE=$(plain "$2" | grep -a "$3" | tail -1)
  if [ -n "$LINE" ]; then
    verdict PASS "$1" "$(echo "$LINE" | cut -c1-110); shot $4"
  else
    verdict FAIL "$1" "no line '$3' in $2; $(plain "$2" | grep -a "MP share" | tail -2 | tr '\n' ';')"
  fi
}
