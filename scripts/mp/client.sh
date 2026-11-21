#!/bin/sh
# A real game client: the probe on the CPU Vulkan driver, under its own virtual display, with its own
# data, config and cache folders, so its identity key is its own. Needs OUT, PORT, T and PROBE.

LVP="$HOME/.cache/abstractions-targets/lvp"
TOOLCHAIN="$HOME/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu"
RUSTLIB="${MP_RUSTLIB:-$TOOLCHAIN/lib/rustlib/x86_64-unknown-linux-gnu/lib}"

# Starts a virtual display and waits for it. It refuses a display number somebody else uses.
display_start() {
  if [ -e "/tmp/.X11-unix/X$1" ] || [ -e "/tmp/.X$1-lock" ]; then
    echo "display :$1 is in use: pick another with MP_DISPLAY_A or MP_DISPLAY_B" >&2
    return 1
  fi
  Xvfb ":$1" -screen 0 1280x720x24 >/dev/null 2>&1 &
  XVPID=$!
  track "$XVPID"
  END=$(( $(date +%s) + 30 ))
  while [ ! -S "/tmp/.X11-unix/X$1" ]; do
    [ "$(date +%s)" -lt "$END" ] || return 1
    sleep 1
  done
}

# Prints the cores a client may use, from MP_CPUS_A or MP_CPUS_B, or nothing.
cores_of() {
  case "$1" in
    a) echo "${MP_CPUS_A:-}" ;;
    *) echo "${MP_CPUS_B:-}" ;;
  esac
}

# Starts a client: client ROLE DISPLAY TAG STAGES. Its log is OUT/ROLE/TAG.log and its shots OUT/ROLE/shots-TAG.
# It sets CPID to the pid of the game and XVPID to the pid of its display.
client() {
  D="$OUT/$1"
  mkdir -p "$D/shots-$3" "$D/data" "$D/config" "$D/cache" "$OUT/sync"
  display_start "$2" || return 1
  CORES=$(cores_of "$1")
  PIN=""
  [ -z "$CORES" ] || PIN="taskset -c $CORES"
  $PIN env DISPLAY=":$2" VK_ICD_FILENAMES="$LVP/libs/usr/share/vulkan/icd.d/lvp_icd.json" \
    LD_LIBRARY_PATH="$LVP/libs/usr/lib/x86_64-linux-gnu:$T/deps:$RUSTLIB" ALSA_CONFIG_PATH="$LVP/asound.conf" \
    XDG_DATA_HOME="$D/data" XDG_CONFIG_HOME="$D/config" XDG_CACHE_HOME="$D/cache" \
    LP_NUM_THREADS="${MP_LP_THREADS:-3}" PROBE_SOFT=1 SMOKE_SEED=7 SMOKE_DIR="$D/shots-$3" SMOKE_ONLY="$4" \
    PROBE_JOIN="127.0.0.1:$PORT" PROBE_NAME="Player-$1" PROBE_ROLE="$1" PROBE_SYNC="$OUT/sync" \
    "$PROBE" >"$D/$3.log" 2>&1 &
  CPID=$!
  track "$CPID"
}

# Waits until both games wrote their ready file: wait_ready PID_A PID_B SECONDS. Returns 2 when a game died.
wait_ready() {
  END=$(( $(date +%s) + $3 ))
  while [ "$(date +%s)" -lt "$END" ]; do
    [ -f "$OUT/sync/ready-a" ] && [ -f "$OUT/sync/ready-b" ] && return 0
    kill -0 "$1" 2>/dev/null && kill -0 "$2" 2>/dev/null || return 2
    sleep 5
  done
  return 1
}

# Starts the script of both games at the same moment, 20 seconds from now.
start_script() {
  echo $(( $(date +%s) + 20 )) >"$OUT/sync/t0.tmp"
  mv "$OUT/sync/t0.tmp" "$OUT/sync/t0"
}

# Stops a game and its display after it ended, and says how it ended: end_client PID XVPID.
end_client() {
  wait "$1" 2>/dev/null
  ENDED=$?
  kill "$2" 2>/dev/null
  untrack "$2"
  untrack "$1"
  return "$ENDED"
}
