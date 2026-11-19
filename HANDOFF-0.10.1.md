# Handoff: finishing Abstractions 0.10.1

Written 2026-11-19, when the work moved off the build machine "ruche". Read this first, then
`docs/handoff/0.10.1-brief.md` (the plan and the rules every step followed) and the
`## [0.10.1]` section of `CHANGELOG.md`.

## Where things are

| Branch on origin | State |
| --- | --- |
| `next` | The 0.10.1 integration branch. Everything here passed the full gate when merged. Release commit `chore: release v0.10.1` (version 0.10.1 in `Cargo.toml`) is already on it, as are the changelog and the README. Merge the three branches below into it, then release from it. |
| `w-polish` | Map polish, round 2. 11 finished commits plus one WIP commit (see below). |
| `w-trails` | Trails over the sea and patchy water. One finished commit plus one WIP commit. |
| `w-flight` | The glider rework. One WIP commit only. |
| `main`, `physical-planets` | v0.10.0, plus on `main` the new `ROADMAP.md` (`d6c43e7`, already merged into `next`). |
| `ecosystems`, `eco-*` | The 0.11 work (ecosystems and real-size planets), handed off separately with its own `HANDOFF.md`. Start it after 0.10.1 is out. |

A WIP commit is titled `wip(<name>): unfinished work, not gated`. It may not compile, and no
test or lint was run on it. Never merge one into `next` as it is.

## On `next` already (merged, gate green)

- Protocol 2: ship stick and boost, companion face, waypoints and explored ground on the wire,
  favourites in the player record, a server player cap up to 1000 (default 16).
- Smoother remote bodies: the jitter buffer measures the one-way latency, records are stamped
  on the universe clock, a frame switch converts the buffer instead of emptying it, and
  `Around`/`System` hysteresis.
- Remote pilots seated in their cockpit, glow and plume per ship, trails per robot, companion
  faces, friends on the compass.
- HUD: a markers list shared by the compass, minimap and map; the compass fades at its ends
  and hides what is behind; coordinates under the compass; keys M (map), N (minimap), B
  (waypoint).
- Step up stairs and ledges up to 0.35 m.
- Wing, winglet, fin and canopy colliders; leaving the ship stops the motors.
- The map (M, or Tab in the ship): one zoomable map from the ground to the universe, coloured
  planets with explored ground shared on servers, waypoints, favourites, and a course line
  that always says why it cannot start.
- The map fixes from the review round 1 (MAPNET): hosting shares explored ground and cuts
  sends under 64 KB, waypoints reset with the world, bad server data ignored, painting only
  while the map is open.

The last full run on `next` before the latest merges: 1354 game tests, all net and server
tests, clippy, fmt and filet clean (only the four `gen.config.inert` notes on `filet.yml`).

## What is left, in order

1. **`w-polish`** (map look). Merge its 11 finished commits, which passed clippy, 143 map
   tests and a headless tour. Its WIP commit holds round 2, not finished:
   - unexplored ground plain black, without the bright grid lines (`src/map/raster/paint.rs`);
   - chart labels kept out of the header and footer bands (system chart "Vereya" draws over
     the breadcrumb);
   - "Double-click a place or press Enter to set a course" hidden while the "Set course"
     button shows;
   - the home icon and the ship mark not drawn on top of each other on the charts.
   Finish these, run the gate, look at the map in the game at 1280x720 and 960x540.
2. **`w-trails`** (trails and water). Commit `23d1444` is the trail fix: the sea draws before
   every see-through thing and writes its depth, so a trail over water is never covered.
   The WIP commit touches `src/world/sea/water.wesl` and its shelf test: it is the start of
   the second bug, "some parts of the water are more transparent than others". Find the
   cause (candidates: the depth map's alpha, the near sea and the coarse chunks' sea blending
   twice in the hand-over zone, patch sorting), fix it, and check the sea from above at a
   few heights near the shore and far out.
3. **`w-flight`** (glider). The WIP commit is the whole rework, untested. Target design, from
   the owner's report ("upside down", "pitch weird", "Q and E should work", "re-spreading
   forces a direction"):
   - keys set an attitude, not a rate: bank about ±55°, pitch from -70° (dive) to +30°
     (climb); no loops, never inverted;
   - releasing the keys levels the glider in about 0.5 to 0.8 s;
   - W dives, S climbs, A/D bank into a coordinated turn, Q/E yaw gently (like the ship);
   - stall protection: below the minimum airspeed the nose lowers itself;
   - spreading again mid-air starts from the velocity direction and blends over 0.3 s;
   - the camera follows the heading and tilts only about 30 % of the bank;
   - left-click steer-by-view stays, inside the same limits.
   The WIP deletes `src/gear/stick.rs` and the loop tests and adds `src/gear/command.rs` and
   new tests (`glider.rs`, `respread.rs`, `sends.rs`). Get it compiling, finish the tests
   listed in the brief, then fly it yourself: this one needs a human feel check. Note that
   0.11 will retune its constants for 9.81 m/s²; keep the model.
4. **Release** (the repo's own flow, as for 0.10.0):
   - merge the three branches into `next`; full gate: `cargo fmt --all --check`,
     `cargo clippy --workspace --features dev --all-targets -- -D warnings`,
     `cargo test --workspace --features dev`, `filet check .`;
   - the headless shader tour if you have it (on ruche: `examples/smoke.rs`, untracked, and
     `~/.cache/abstractions-targets/lvp/smoke.sh <display> <outdir>`); otherwise play every
     area once: map levels, flight over sea, cockpit, warp;
   - dry run: push `next` to a throwaway branch and `gh workflow run release.yml --ref <it>`;
     all four jobs must pass;
   - fast-forward `main` and `physical-planets` to `next`, tag `v0.10.1`, push the tag, watch
     the run; check the release has `abstractions-v0.10.1-linux-x86_64`,
     `abstractions-v0.10.1-windows-x86_64.exe`, `Abstractions-v0.10.1-macos-universal.dmg`,
     the `SHA256SUMS-*-v0.10.1.txt` files, the server binaries and
     `ghcr.io/saravenpi/abstractions-server:0.10.1`.
5. **Redeploy Space** (needs the Dokploy CLI configured for ruche's Dokploy): the compose is
   "Space server", id `kVuA_xv-8KNXbLO_cmhYz`, a raw compose. The new file is
   `docs/handoff/space-compose.yml` (image `0.10.1`, 64 players):
   `dokploy compose update --composeId kVuA_xv-8KNXbLO_cmhYz --sourceType raw --composeFile "$(cat docs/handoff/space-compose.yml)"`
   then `dokploy compose deploy --composeId kVuA_xv-8KNXbLO_cmhYz`. Check from outside:
   `abstractions-server ping space.facile.studio` must answer `v0.10.1` with `0/64`. The DNS
   record must stay DNS only (Cloudflare grey cloud).
6. **Site**: on branch `landing`, set the version line in `site/index.html` (`#version`,
   `#released`) to v0.10.1 and push (a push to `landing` deploys).

## Known gaps to keep in mind

- The pause menu's player list cannot scroll; with about 100 players the names run off.
- Planet pictures and explored ground use a fixed 512x256 grid; 0.11's real-size planets
  replace it with a quadtree (planned in the 0.11 handoff).
- The owner's design rules: no borders anywhere, never bold text (the Undefined font renders
  badly bold), black and white UI chrome, colour only in the map picture, player dots and
  markers.
