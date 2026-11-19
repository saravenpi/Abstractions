# Handoff: finishing 0.11.0 on another machine

Written 2026-11-19 on ruche, when the owner moved the work to a Mac M2. Read this first, then the files in `handoff/`. Delete `HANDOFF.md` and `handoff/` in the commit that releases 0.11.0.

## What 0.11.0 is
One release with three parts:

1. **Ecosystems:** species, food webs, herds, hunts and kills, procedural animation, plants by growth shape, tracks and signs, and a field guide.
2. **Real-size planets:** Kodya is Earth-like, moons form realistically, terrain comes from tectonic plates, air follows physics, and the galaxy light comes from real stars.
3. **A clean roadmap** on `main`. The website page that reads it is already live at https://abstractions.facile.studio/roadmap/.

## The plans
| File | What it is |
| --- | --- |
| `handoff/CONTRACT.md` | The rules every agent follows: file ownership, filet limits, build rules, shared types, owner decisions, and the kill rules. Read it before writing code. |
| `handoff/PLAN.md` | The ecosystem plan, with research sources. CONTRACT.md wins where they differ. |
| `handoff/PLANETS.md` | The real-size planets and moons plan, steps 1 to 18 with checks, including 14b (galaxy light) and 17b to 17d (the map at planet scale). |
| `handoff/SPACE.md` | The 0.12 real-space plan. Not part of 0.11. |
| `handoff/ROADMAP.md` | The cleaned roadmap, with a status per version and 0.11 to 0.19. It goes to `main` (see "Steps to finish"). |
| `handoff/smoke.rs.txt` | `examples/smoke.rs`, which is untracked on purpose (listed in `.git/info/exclude`). Copy it to `examples/smoke.rs` and add that path to `.git/info/exclude` on the Mac. It tours the world and takes screenshots. |

## Branches (all pushed to origin)
| Branch | Head | State |
| --- | --- | --- |
| `ecosystems` | this commit | Base from `physical-planets` at v0.10.0. It holds the shared seams (`src/life` types, `src/fauna` plugin and `Creature`, `Globe::with_life`, `life()` and `lives()`, the `life::of` call in `world/enter.rs`) and the merged **bodies** wave. Its `life::` tests pass (17). |
| `eco-core` | `c3f68f2` | Life odds and rung, chemistry and pigment, tissues, guilds, the niche-model food web assembled by `web::assemble`, names, relations. Committed by its agent; the final report never arrived, so test and clippy results are unknown. |
| `eco-pop` | `048dcd6` | Life history, census density, herds as pure functions of the universe clock, and hunts with kills and carcasses (`life::hunt`). Its agent was waiting on the real-crate test run when it was stopped, so results are unknown. |
| `eco-flora` | `70b7ff1` | Plant dials, meshes for the eight growth architectures, and planting from the life of a planet (dead planets bare, Kodya unchanged). Its agent reported filet clean and was on its final test run. |
| `eco-motion` | `7e14a0a` | All procedural motion in `src/fauna/motion/`. filet is clean for those files. Clippy and tests never finished: run them first. |
| `landing` | `9aefbee` | Live website with the roadmap page and the hero image. Already deployed. |

Each `eco-*` branch was made from `ecosystems` at `dc4097f`. They do not include each other's work.

### Known conflicts when merging
- `src/universe/mod.rs`, the `hash` module: `ecosystems` has `pub mod hash;` with `#[allow(clippy::should_implement_trait)]` on `Rng::next` in `src/universe/hash.rs`. `eco-core` ends with `pub(crate) mod hash;`. Keep `pub mod hash;` and the allow.
- `src/fauna/mod.rs`: `ecosystems` declares `pub mod mesh;` and `pub mod motion;`. Keep both.
- `src/life/mod.rs`: keep every `pub mod` line from all branches (core adds none new; pop adds `hunt`).
- `src/effects/mod.rs`: `eco-motion` makes `creatures` `pub(crate)` so motion can reuse `effects::creatures::anim`. Keep it.

## Owner decisions (2026-11-19)
- **Version:** 0.11.0 bundles the ecosystems, real-size planets, moons and galaxy light. Real space is 0.12.
- **Real rules, not tuning:**
  - Gravity 9.81 m/s²: retune the robot, and keep the 0.10.1 glider model; only its constants change.
  - Days computed from spin and orbit. No day length is set anywhere.
  - Kodya: 6,371 km radius, spin 86,164 s, one impact moon.
  - Nothing on screen is decoration.
- **Speed:** the game must run fast. Every feature lands with a cost test or a measured frame budget. If something does not fit, tell the owner with numbers before shipping.
- **Kills:** hunts are pure functions of seed, herd and universe clock. The robot is not prey in 0.11.
- **Kodya fauna:** rung Skeletal, with a grazer herd guaranteed within 300 m of the wake point.
- **Scope in 0.11:** the field guide (key K, with a conflict check in `src/menus/options/binds.rs`), tracks and signs, and plant growth shapes.
- **Old saves:** each world records its game version, and older worlds open with their own release, downloaded by `src/update`. No old generator code in the new binary.

## Coordination with 0.10.1
Another Claude session on ruche is shipping **0.10.1**: the map with explored ground, waypoints, compass, step-up, the glider rework, and protocol 2. When it lands on `physical-planets` and `main`:

1. Merge `physical-planets` into `ecosystems` first, before any planet work. 0.10.1 changes `crates/net`, `src/player`, `src/saves`, `src/map`, `src/menus` and `src/lib.rs`. Read those files from the merged tree, not from v0.10.0.
2. In `src/lib.rs`, the fauna plugin is one line after `modules(app)` in `AppPlugin::build`. 0.10.1 splits `modules` into `modules` and `later_modules`, which should merge cleanly.
3. Put `handoff/ROADMAP.md` on `main` as `ROADMAP.md`, on top of v0.10.1 (docs only). The website then reads it live; until then it shows its shipped copy. When releasing 0.11.0, mark 0.11 as released in it.
4. The planet work changes formats the 0.10.1 session left to us: protocol 3 for positions and the explored grid, and `SAVE_VERSION` 3 with `game_version` and the explored quadtree.

## Steps to finish
1. **Merge 0.10.1** into `ecosystems` (see above).
2. **Finish wave 1, one branch at a time.** For each of `eco-core`, `eco-pop`, `eco-flora`, `eco-motion`:
   - Check it out, run `cargo clippy --features dev --all-targets -- -D warnings`, `cargo test --features dev <its modules>` and `filet check .`.
   - Fix what fails, then merge it into `ecosystems`. Suggested order: core, pop, flora, motion.
   - After core merges, `life::of` returns real life and Kodya lives. Check that the old effects tests still pass.
3. **Look at the animation before going further.** Build a small `examples/zoo.rs`, untracked like `smoke.rs`, that lays Kodya's species in a line walking a circle, and take screenshots at walk, trot and gallop. Planted feet must not slide. This was the plan's phase 1 gate.
4. **Wave 2 (ecosystems).** Briefs are in CONTRACT.md, "Wave 2":
   - `live`: spawning from census and herds within 150 m, at most 48 creatures, `Rebased`, dither fade; the needs brain and steering; reactions to every robot, local and remote; drawing hunts; glow; and the `effects/creatures/habitat.rs` presence by rung.
   - `signs`: tracks, droppings, grazed patches, nests, bones from carcasses, and calls through `AudioCue`.
   - `guide`: the field guide panel on K.
5. **Planet waves:** `handoff/PLANETS.md`, "Parallel plan". P1 is units and moons, precision, plates, air. P2 is galaxy light, net, drainage, sky, ship, saves, map and tune.
6. **Ship 0.11.0:**
   - `mise run check` green.
   - Smoke screenshots at every altitude, with frame times reported to the owner.
   - CHANGELOG entries above 0.10.1's.
   - README.
   - ROADMAP: 0.11 released.
   - The release flow: the `release-preflight` flow, a `v0.11.0` tag, and the landing version line.
   - The server image tag in the Dokploy compose "Space server" (see the wiki, `projects/abstractions.md`). Protocol 3 means 0.10 clients cannot join.

## On the Mac
- **CPU:** ruche (12 cores) choked at a load average of 100 to 140 with ten agents building Bevy at once. On an M2, run one cargo build at a time. Give each worktree its own crate version (CONTRACT.md "Building and testing") if two copies ever share a target directory.
- **Builds:** always pass `--features dev`. The gate is `mise run check` (fmt, clippy, tests, filet).
- **Ray tracing:** bevy_solari needs a binding-array limit M1 and M2 do not have on `main` (wiki: "Ray tracing on M1 and M2"). Leave ray tracing off when testing there.
- **Wiki:** the mycelium wiki syncs across machines. `mycelium memory search "Abstractions"` has the project page with every gotcha from 0.4 to 0.11, including the shim-crate harness for testing pure modules fast.
