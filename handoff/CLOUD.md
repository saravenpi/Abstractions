# Handoff to the cloud session (2026-11-20, evening)

Read `AGENTS.md`, then this file, then `handoff/MAC.md` (0.11 decisions), `handoff/HUNT.md` (hunting plan and corrections) and `handoff/PLANETS.md` (real-size planets; `MAC.md` overrides it: no hand-made home planet).

## How the work ran today

The owner asked for speed: agents coded with `cargo check` only, and the lead builds, tests and looks at the game at the end. So most of what is below type-checks but has never run. Shader errors only show in the running game (a reserved word, `patch`, already broke the ground once). Treat every "not run" item as unverified.

## Branches (all pushed; every agent has stopped, nothing is left only on the Mac)

| Branch | Holds | State |
| --- | --- | --- |
| `ecosystems` | The 0.11 line. Merged today: big game, slope walking, herd marks after seen, limb joints, seated plant parts, fixed pixel look, water view and camera fixes, turf carpet and blades, leaves from real plants, clear ship glass, smooth coloured minimap, map sea colour, one home mark, boot water jet, spray and wake, call poses and new cries, compass home rule, atmosphere entry, and the `fixgate2` fixes | Full gate on 1caf41a: clippy clean, filet at its four baseline warnings, net and server tests pass, game tests 1,659 passed and 1 failed (`render::heat::tests::thick_air_glows_earlier_than_thin`, a wrong test, fixed in the next commit and passing with its module). The full suite was not rerun after that fix |
| `warp-align` | `ecosystems` with commit 1ae8704 (align before the warp, never turn in warp) still in | Eight `ship::tests` fail there: the autopilot never spools. It was reverted on `ecosystems` (3fd4952) so the ship can warp. Diagnosis below |
| `leaves` | Leaf fall as a function of wind, season, rain and thirst (953bd04) | 26 tests passed before two last fixes; filet will flag `src/effects/shed.rs` (9 functions). Not merged |
| `wake` | V-shaped foam and a wave trail in the sea shader (7168ebf) | Never run; `field.wesl` never loaded by a GPU. Not merged |
| `hunt` | H1 craft model, senses, swing, craft UI, `ecosystems` (before `entry` and `fixgate2`) and `voice` merged; hits wired to health, kills to the ledger and to harvest; one held item; F8 starter tools into the pack; H picks up stone | fmt, clippy, filet clean. 22 of 296 hunting tests fail, listed below |
| `h-senses` | Senses, then crouch (213662f, 065a779, 8139228). `hunt` holds it only up to 79b24d3: merge the crouch commits | Not run |
| `h-swing`, `h-craftui` | Already merged into `hunt` | |
| `p-units` | Real radii, gravity, spins, moons, generated home, fewer wet worlds (0c7ea5a) | Not run. About 200 tests outside `universe` assume the old fixed home |
| `p-air` | Real gas mix and air column | Not run. Edits `planet.rs`, `giant.rs`, `moon.rs`, `home.rs`, `mod.rs`, `record.rs`: expect conflicts with `p-units` |
| `p-precision` | f64 chart origin, shader altitude from the camera | Not run. May conflict with `p-air` in `src/render/atmosphere.rs` |
| `p-plates` | Plates, continents, relief (1a329f2); off below a 50 km radius | Not run |

Merge the four `p-*` branches onto a new `planets` branch together, never one alone into `ecosystems`: `p-air` alone squashes the air on small planets.

## Warp alignment (branch `warp-align`)

Owner rule: the ship finishes aligning before the warp and never changes heading during it; a new course drops out, re-aligns, warps again.
Diagnosis from the fix agent, unconfirmed: `align::settled` (`src/ship/route/align.rs`) stays false; `tolerance()` shrinks toward `FLOOR` 1e-7 rad for a far planet while the led aim keeps moving, so the proportional stick in `steer::aim` trails it. `moon_route` shows a drop-out loop (3 spools in 34 s) from `missed`.
Suggested fix: snap the residual and zero the spin once within about 1e-3 rad, then spool; freeze the lead aim from the spool on (count the drift of the 4 s spool); hysteresis on `missed`.

## Failing tests on `hunt`

- Pack save: `craft::pack::the_pack_survives_the_world_file`, five `menus::worlds` and two `menus::tests::worlds` fail with `UnsupportedNone`: the hotbar's `Option<u32>` slots cannot be written to TOML. Write empty slots as a sentinel in `src/craft/pack/bar.rs`. Until then saving a world with a pack may fail.
- `craft::build::tests_recipe::recipes_round_trip_small`: 101 bytes against 72; the text form of `Stuff::Tissue.planet` in `src/craft/matter/stuff.rs` should apply to human-readable formats only.
- No cause found yet: `craft::harvest::tests::real_parts_give_tissue`, `fauna::senses::tests::a_ridge_blocks_sight`, `fauna::senses::tests_blood::blood_falls_every_two_metres_and_fades`, `fauna::tests_react::animals_flee_a_near_robot_and_come_back` (Threaten where Flee is expected), `fauna::tests_walk::animals_stand_on_the_ground` (0.29 m against 0.25 m), two `avatar::strike::tests_ik`.
- Already fixed on `ecosystems` by `fixgate` and `fixgate2` (merge `ecosystems` into `hunt` again): the signs tests, the dawn chorus test, `settings::tests_removed`.
- Open in `hunt`: a thrown spear stays in the pack while it flies (a hotbar change can duplicate it); the harvest uses the held weapon as the tool; no carcasses; stone pickup is not tied to rock props.

## Decisions waiting for the owner

- `p-units`: the chosen home is often a tidally locked red-dwarf planet (one side in permanent night). Excluding locked planets is one line in `landable` (`src/universe/home.rs`) but bends the "no special case" rule.
- Carry limit: 25 kg at 9.81 m/s².
- The wake's V is floored at 4 degrees; the physical wedge at flight speeds is 1 to 2 degrees.
- The entry glow guesses the gas from the planet class until `p-air` merges (`record.mix.main_gas()`).

## Never seen in the game

Grass blades and their cost (the owner saw 20 fps, but the machine was at load 94), the waterline pass, the foam wake, the atmosphere-entry sheath (`src/render/heat/sheath.wesl`), call poses, crouch, every hunting system, leaf fall rates.

## Owner requests still open

- Hunting wave H3: creature attacks from body parts, carcasses that stay and are harvested by part, the stuck spear, robot health by part with visible damage and its effects (see "Additions from the owner" in `HUNT.md`). Wave H4: multiplayer.
- Planets wave P2 (`PLANETS.md`, "Parallel plan"): drainage and rivers, spawn, sky and weather, ship, saves, map, net, the 3b retune. Switch the entry colours from `Gas::of(class)` to `record.mix.main_gas()`; call `life::chem::air` so living worlds get oxygen.
- Remote players do not show crouch or the entry sheath.
- Carry limit: 25 kg at 9.81 m/s², about 11 kg on a 22 m/s² world; the owner has not judged it yet.
- Later: persist the field guide's seen species, the site hero alt text.

## Screenshots

`examples/probe/` drives the real game through a tour and saves PNGs (`SMOKE_SEED=1 SMOKE_DIR=<dir> cargo run --features dev --example probe`). It is one tracked example with a module per area. `PROBE_STAGES` (comma list of `water`, `grass`, `fauna`, `tour`, `zoo`; default all but `zoo`) picks the stages, and `SMOKE_ONLY` and `SMOKE_SKIP` pick single stages by name. `PROBE_STAGES=zoo` shows every animal of the seed walking in a line instead of touring the game. A cloud machine without a GPU cannot run it.
