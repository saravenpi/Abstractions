# Handoff to the cloud session (2026-11-20, evening)

Read `AGENTS.md`, then this file, then `handoff/MAC.md` (0.11 decisions), `handoff/HUNT.md` (hunting plan and corrections) and `handoff/PLANETS.md` (real-size planets; `MAC.md` overrides it: no hand-made home planet).

## How the work ran today

The owner asked for speed: agents coded with `cargo check` only, and the lead builds, tests and looks at the game at the end. So most of what is below type-checks but has never run. Shader errors only show in the running game (a reserved word, `patch`, already broke the ground once). Treat every "not run" item as unverified.

## Branches

| Branch | Holds | State |
| --- | --- | --- |
| `ecosystems` | The 0.11 line. Merged today: big game and slope walking and herd marks after seen and limb joints (`fauna2`), seated plant parts (`fix-flora`), fixed pixel look (`pixels`), water view and camera fixes (`waterview`), turf carpet and blades and leaves from real plants (`grass`), test fixes (`fixgate`), clear glass and align-before-warp (`shipfix`), smooth coloured minimap and map sea colour and one home mark (`minimap`), boot water jet and spray and wake (`watergear`), call poses and new cries (`voice`), compass home rule and atmosphere entry (`entry`) | Type-checks. Last full test run (before `entry`): 1,644 passed, 13 failed, see below |
| `fixgate2` | Fixes for those 13 failures, warp first | Agent was still working when this was written |
| `leaves` | Leaf fall as a function of wind, season, weather | Agent was still testing; uncommitted work may exist only on the Mac |
| `wake` | V-shaped Kelvin wake and a real wave trail in the sea shader | Just started |
| `hunt` | Hunting integration: H1 craft model, senses, swing, plus `ecosystems` and `voice` merged, hits wired to health | Integration agent was still working; then merge `h-craftui` and the crouch commits of `h-senses` |
| `h-senses` | Animal senses, health, ledger, lookouts, blood; then crouch (213662f, 065a779, 8139228) | Not run |
| `h-swing` | Swings, guard, parry, spear throw, hitstop, starter tools on F8 | Not run |
| `h-craftui` | Pack, hotbar, workshop screen on I, harvest, save | Not run |
| `p-air` | Real gas mix and air column per planet | Not run. Do NOT merge into `ecosystems` alone: at today's small radii it squashes the air |
| `p-precision` | f64 chart origin, shader altitude from the camera | Not run |
| `p-plates` | Plates, continents, relief | Agent still working |
| `p-units` | Real radii, gravity, spins, moons, generated home | Agent still working; uncommitted on the Mac |

Planet branches go together onto a `planets` branch first, and into `ecosystems` only once `p-units` lands.

## Known failures on `ecosystems`

- Eight `ship::tests` (reroute, retarget, steady, moon_route, voyage): the autopilot never reaches the warp. Cause: commit 1ae8704 demands 1e-3 rad and 1e-3 rad/s before spooling. The owner's rule stays: align fully before the warp, never turn during it. The ship probably cannot warp in the game until this is fixed.
- `menus::hud::minimap::tests::ground::a_world_of_seas_is_coloured` and `modes::the_chart_follows_the_destination_and_the_frame`.
- `gear::tests::jet::the_jet_settles_on_its_top_speed`, `life::signs::tests_voice::herds_sing_at_dawn_and_young_answer`.
- Timing tests that fail under load and pass alone: `sphere_height_is_not_slow`, `fauna::motion::tests_cost::grown_bodies_step_and_pose_within_budget` (rerun on a quiet machine).
- Clippy: `type_complexity` at `src/menus/hud/minimap/follow.rs:49`.
- A double despawn warning at boot, probably the warm-up copies (`src/render/warm.rs`).
- `src/ship/tests/fx.rs` numbers were rewritten by the entry agent from estimates; they may fail.

## Never seen in the game

Grass blades and their cost (the owner saw 20 fps, but the machine was at load 94), the waterline pass, the foam wake, the atmosphere-entry sheath (`src/render/heat/sheath.wesl`), call poses, crouch, every hunting system, leaf fall rates.

## Owner requests still open

- Hunting wave H3: creature attacks from body parts, carcasses that stay and are harvested by part, the stuck spear, robot health by part with visible damage and its effects (see "Additions from the owner" in `HUNT.md`). Wave H4: multiplayer.
- Planets wave P2 (`PLANETS.md`, "Parallel plan"): drainage and rivers, spawn, sky and weather, ship, saves, map, net, the 3b retune. Switch the entry colours from `Gas::of(class)` to `record.mix.main_gas()`; call `life::chem::air` so living worlds get oxygen.
- Remote players do not show crouch or the entry sheath.
- Carry limit: 25 kg at 9.81 m/s², about 11 kg on a 22 m/s² world; the owner has not judged it yet.
- Later: persist the field guide's seen species, the site hero alt text.

## Screenshots

`examples/probe.rs` drives the real game through a tour and saves PNGs (`SMOKE_SEED=1 SMOKE_DIR=<dir> cargo run --features dev --example probe`). It is not tracked. Three diverged copies are in `handoff/probe/` (`probe-water`, `probe-grass`, `probe-fauna`, each with its own stages) with `zoo.rs` and `handoff/smoke.rs.txt`. Copy one to `examples/probe.rs` to use it; merging them into one tracked example is worth doing. A cloud machine without a GPU cannot run it.
