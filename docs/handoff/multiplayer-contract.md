# Abstractions multiplayer: design contract

> The 0.10.0 design contract, kept for reference. Paths under /home/yann are the build machine "ruche".

Read this fully before writing code. It is the agreed design. If something here cannot work, stop and
report instead of inventing a different design.

## Goal

Up to 16 players explore one universe together. A tiny self-hosted server (domain or IP), an in-game
"Open to LAN" host, and a Minecraft-style Multiplayer screen. The game must stay small and fast.

## Repos and trees

- Integration tree: `/home/yann/Code/Saravenpi/abstractions-mp`, branch `multiplayer`.
- Never touch `/home/yann/Code/Saravenpi/Abstractions` (other agents work there) except to read.
- Game crate: `abstractions` at the workspace root (Bevy 0.20.1, avian3d 0.8). New crates:
  `crates/net` (package `abstractions-net`, NO Bevy dependency) and `crates/server` (package
  `abstractions-server`, binary `abstractions-server`).

## Architecture

- Each client is the authority on its own robot, ship and companion. It sends their state; nobody
  corrects the local player from the network.
- The server is a referee: it never simulates. It owns the seed, the universe clock, the `Changes`
  log, the shared atlas, and one record per player (place and ship). It relays state.
- The same `Room` (sans-IO state machine, in `crates/net`) runs in the dedicated server and inside the
  game when a player opens a world to LAN.
- Single player runs no network code at all: no thread, no socket, every system gated.

## Transport

- QUIC through `quinn` 0.11 with `default-features = false, features = ["rustls-ring", "runtime-smol"]`.
  No tokio anywhere. rustls with `default-features = false` and `ring`.
- Unreliable datagrams carry state. Reliable streams carry everything else (hello, welcome, log
  entries, joins, leaves, pings).
- Serialization: `postcard` + serde. Message enums are append-only: new variants go at the end.
- Transport settings: keep-alive 5 s, idle timeout 15 s, datagram send buffer about 2 ticks (stale
  poses are dropped, not queued), larger socket buffers, address validation by retry on.
- Never assume a 1200-byte datagram: split batches by the connection's `max_datagram_size()`.
- Default ports: UDP 24650 game, UDP 24651 LAN beacon. Protocol version: a `u16` constant.

## Trust

- One path only: trust on first use, like SSH. The server makes a self-signed certificate with
  `rcgen` on first start and SAVES it with its key in its data folder. The client remembers the
  SHA-256 fingerprint per address in `known_servers.toml` and refuses a changed one.
- A domain is only a name for the IP. No web PKI, no `webpki-roots` in the net code.
- Short key shown to humans: first 6 bytes of the fingerprint as `7F3A-91C2-0B5E`.

## State on the wire

- Frames a position is expressed in:
  - `Surface(BodyId)`: fixed to the body and turning with it. On foot, or a landed ship.
  - `Around(BodyId)`: centred on the body, NOT rotating (inertial). A ship inside a gravity well.
  - `System(StarId)`: inertial, origin at the star. Deep space only.
  Reason for `Around`: planets move at about 8.6 km/s, so a 100 ms interpolation delay in `System`
  would draw an orbiting ship 860 m off relative to its planet.
- A state datagram is a batch of records. One record = a small header (player id `u8`, kind, frame,
  universe time of the sample as `f64` or a tick) + an opaque body. THE SERVER NEVER DECODES THE
  BODY: it stores the latest bytes per player and kind and copies them into each recipient's batch.
- Kinds: robot, ship, companion.
- Quantization (`pack`): rotation = smallest-three in 32 bits; surface position = millimetre `i32`
  per axis; `Around`/`System` position = `f64`; velocity = `f32` x3; pose fields small ints.
- Robot body carries what `avatar::AvatarPose` needs: mode, speed, vertical speed, stride distance,
  bank, pitch, thrust, wings, airspeed, look, landed_hard, plus yaw.
- Send rate: 30 Hz while moving, 2 Hz at rest (an at-rest flag is set). Interest: players in the
  same star system get full rate; others get 1 Hz (for the map).
- Player names: at most 24 characters, no control characters.

## Time

- The server owns universe time. Clients estimate the offset with ping/pong (keep the minimum-RTT
  sample of the last 16), then slew their clock by at most 2 percent. Snap only on join or beyond 1 s.
- In a session the universe clock never pauses, and the pause menu does not freeze physics.

## Smoothness

- Remote entities are drawn from a jitter buffer: sample at `now - delay`, Hermite interpolation
  using the sent velocity, slerp for rotation. Delay = 2 send intervals + p95 jitter, clamped to
  50..250 ms. Extrapolate at most 100 ms, then hold.

## Server behaviour

- One thread, one loop at 30 Hz, asleep when the room is empty. No allocation per tick.
- Limits: 16 players (configurable), message rate per player, log append rate per player, new
  connections per IP.
- A status request (name, players, cap, version, protocol) is answered BEFORE the version check, so
  an outdated server shows in the list as outdated instead of failing.
- World file in TOML: name, seed, clock, log, atlas, player records. Atomic writes every 30 s and on
  SIGTERM/Ctrl-C, last 3 copies kept. `--import <single-player world file>`.
- Zero configuration: `abstractions-server` in an empty folder creates `./data/world.toml` with a
  random seed, makes the certificate, and prints a banner (name, seed, port, LAN address, short key,
  "forward UDP 24650"). Flags or env: `ABSTRACTIONS_NAME`, `ABSTRACTIONS_PASSWORD`,
  `ABSTRACTIONS_PORT`, `ABSTRACTIONS_MAX_PLAYERS`, `ABSTRACTIONS_DATA`. Never print the password.
- Subcommands: `run` (default), `ping <address>`, `version`.

## Game behaviour

- Puppets: no avian body, no collider, shared mesh and material handles. Detail by distance: full
  under 150 m, pose only to 600 m, nothing but a map marker on another planet.
- While joined, nothing is written to local world files; the player's place and ship go to the
  server. Discoveries (atlas, `Changes`) are shared by the whole server.
- UI: black and white, no hue, no rounded corners, pixelarticons only, existing `crate::ui` widgets.

## Code rules (the gate fails otherwise)

- `filet check .` must pass (config in `filet.yml`): at most 250 lines per file, 8 functions per
  file (trait impl methods count), 30 lines per function (warning), 5 parameters, 14 struct fields,
  nesting 4, complexity 10, line length 120. A module that grows becomes a folder.
- NO comments inside function bodies, NO inline comments, no TODO. EVERY item (public or not) gets a
  `///` doc comment; every file starts with a `//!` module doc. Plain language, short sentences.
- Tests live in their own files (`foo_tests.rs` or `tests/`), as the repo already does.
- No dead code. No new dependency beyond the ones named in your task without reporting it.
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings` must be clean for what you touched.
- Commits: Conventional Commits, `type(scope): summary`, imperative, lowercase, under 72 chars. NO
  co-author line, NO "Generated with" line, no gitmoji. Commit on your branch. Never push.
- Never weaken or delete an existing test to make it pass.

## Building without fighting other agents

- Disk is at 92 percent and other agents hold the shared `target/`. NEVER build into
  `/home/yann/Code/Saravenpi/Abstractions/target`.
- Work on `crates/net` or `crates/server` only: use your own small target,
  `export CARGO_TARGET_DIR=/home/yann/Code/Saravenpi/abstractions-mp-target/<your-name>` and build
  with `-p abstractions-net` / `-p abstractions-server` only. Never build the game crate there.
- Work on the game crate: use the target dir your task names. Seeding a Bevy target without a cold
  build: `rsync -a --exclude 'abstractions-*' --exclude 'libabstractions-*'
  /home/yann/Code/Saravenpi/Abstractions/target/debug/{deps,build,.fingerprint} <target>/debug/`
  (about 14 GB). Game checks: `cargo check --features dev --all-targets`,
  `cargo test --features dev`, `cargo clippy --features dev --all-targets -- -D warnings`.
- If cargo prints "Blocking waiting for file lock", wait; do not kill other processes.

## Report format

End with: what you built (files), the exact verify commands you ran and their real output lines,
anything that fails or was skipped (say so plainly), any deviation from this contract and why, and
the commit hashes.

## Server distribution names (fixed, everything must match these)

- Release assets: `abstractions-server-linux-x86_64`, `abstractions-server-linux-aarch64` (static musl
  binaries, no archive), checksums in `SHA256SUMS-server-vX.Y.Z.txt` (lines `<sha256>  <file name>`),
  plus the same binaries under versioned names `abstractions-server-vX.Y.Z-linux-x86_64` and
  `abstractions-server-vX.Y.Z-linux-aarch64`. Stable names are reachable at
  `https://github.com/saravenpi/Abstractions/releases/latest/download/<name>`.
- Docker image: `ghcr.io/saravenpi/abstractions-server` with tags `latest` and `X.Y.Z`, linux/amd64 and
  linux/arm64, from `scratch`. Data volume at `/data`. UDP 24650.
- Install script: `https://abstractions.facile.studio/server.sh` (file `site/server.sh` on branch
  `landing`).
- CLI: `abstractions-server [run]` with flags `--name`, `--password`, `--port`, `--max-players`,
  `--data`, `--import <file>`; env `ABSTRACTIONS_NAME`, `ABSTRACTIONS_PASSWORD`, `ABSTRACTIONS_PORT`,
  `ABSTRACTIONS_MAX_PLAYERS`, `ABSTRACTIONS_DATA` (default `./data`, `/data` in the image);
  `abstractions-server ping <address>` (exit 0 when the server answers); `abstractions-server version`.
- Public server run by the project: `space.facile.studio` (UDP 24650).
