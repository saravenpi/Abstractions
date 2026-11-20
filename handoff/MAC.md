# Running 0.11 on the Mac

Written 2026-11-19 on the owner's MacBook (Apple M2, 8 cores, 16 GB). It replaces the ruche-specific parts of `CONTRACT.md` ("Repo and branches", "Building and testing"). Everything else in `CONTRACT.md` still holds. The repo-wide rules are in `AGENTS.md` on `next`, which this branch gets when 0.10.1 merges in.

## Owner decision, 2026-11-19 (overrides CONTRACT.md and HANDOFF.md)

Kodya is not special-cased. Its life comes from the same odds and rung rules as every planet: no forced `Rung::Skeletal`, and no herd guaranteed near the wake point. If the normal rules do not make an Earth-like home planet live, fix the physics they read and report the numbers to the owner; never add a home-planet override. The same holds for plants: on a living globe Kodya's plants come from its life through the normal planting, not the old hand-made tables, so its look changes in 0.11.

## Owner decision, 2026-11-19: home is chosen, not made, and not named by hand (overrides PLANETS.md and HANDOFF.md)

Kodya and Vereya are no longer hand-made. Remove `PlanetRecord::TEMPERATE` as the home record and every fixed home value (the 6,371 km radius, the 86,164 s spin and the single impact moon that PLANETS.md and HANDOFF.md set). The universe is generated with the normal rules for every planet; then the home is picked: the nearest planet to the universe's centre that is solid, landable and alive under the normal life odds. If no living planet lies within a reasonable search (say the nearest few hundred systems), take the most suitable solid planet and report it. The chosen planet and its star get generated names like every other place: there is no Kodya and no Vereya any more. Remove `HOME_PLANET_NAME`, `HOME_STAR_NAME` and every fixed home name, the home galaxy's included (it is already generated; keep it so); code that needs home asks for the chosen planet. Rewrite the texts that name them (README, ROADMAP, the website hero, the og image alt text, the boot and wake screens) to speak of "your home planet" or use the generated name at runtime. The same seed gives the same home, so multiplayer still agrees on it. The home base, the wake point and every `HOME_PLANET_NAME` / `is_home` check follow the chosen planet. Old saves open with their own release, as already planned. This belongs to the planet waves (P1, with units and moons) and needs the new life odds from `eco-core` first.

## Owner decisions, 2026-11-20: life odds (after the eco-core per-solvent model)

- **Too many wet worlds.** The universe generator makes a liquid-water world in 84% of systems, so 88% of systems near home hold life. Fix the cause in the planet generator (planet wave P1): water worlds rarer, for physical reasons. Target: life in about 20 to 40% of systems. Core adds a tidal-locking penalty for red-dwarf planets.
- **Lava life rare.** Life in molten silicate stays possible but rare and always at the lowest rung (core).
- **Evolution at Earth's pace.** One rung takes about 1.3 billion years, anchored on Earth's 4 billion years from first life to complex animals (core).
- **Data the planet wave adds for life** (missing today, reported by core): the star's age in `StarSpec` (so planets of one star share an age), liquid amounts per solvent (ammonia and hydrocarbon lakes), stored tidal heat (for vents), axial tilt in `World` (climate swings), and the planet's carbon supply.

## Paths

| What | Path |
| --- | --- |
| Lead checkout (merges only) | `/Users/fangafunk/Code/Sara/w-eco`, branch `ecosystems` |
| Agent worktrees | `/Users/fangafunk/Code/Sara/eco-<name>`, branch `eco-<name>`, made from `ecosystems` |
| Agent build cache | `/Users/fangafunk/Code/Sara/abs-target-<name>`, an APFS clone of the lead's warm cache (`cp -c -R`), deleted when the agent ends |

## Build rules on this machine

- **One target directory per agent.** The crate-version trick from `CONTRACT.md` is not needed: separate caches never share test binaries.
- **At most two cargo builds at once.** On 2026-11-19 four parallel Bevy builds pushed the load average past 120, put RAM into swap and made each build three to four times slower. Two agents building, plus the lead idle, is the limit.
- Export in every shell: `CARGO_TARGET_DIR=/Users/fangafunk/Code/Sara/abs-target-<name> CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`. Debug info off keeps a cache near 5 GB; full debug info doubles it.
- On macOS, `world::scatter::tests_frozen::flat_output_is_frozen` fails from float rounding. Skip it with `-- --skip flat_output_is_frozen`. Timing tests (`sphere_height_is_not_slow`, `fast_flight_loads_beyond_the_fog`) fail when the machine is loaded: rerun them alone before calling them broken.
- filet is 0.21 from `kit` (`~/.local/bin/filet`). The clean baseline is the four `gen.config.inert` warnings.

## Order

### 0. Base (lead)
1. After v0.10.1 is tagged, merge `physical-planets` (then equal to v0.10.1) into `ecosystems`. 0.10.1 changed `crates/net`, `src/player`, `src/saves`, `src/map`, `src/menus`, `src/gear`, `src/view`, `src/avatar` and `src/lib.rs`, and added `AGENTS.md` and `bacon.toml`.
2. Resolve the conflicts listed in `HANDOFF.md`, "Known conflicts when merging". Gate the merged base: clippy, the `life::` and `fauna::` tests, filet.
3. Copy `handoff/smoke.rs.txt` to `examples/smoke.rs` and add it to `.git/info/exclude`.

### 1. Wave 1: gate and merge the four branches (two agents at a time)
Each agent merges the new `ecosystems` into its branch, runs the gate, fixes what fails inside the files it owns (see `CONTRACT.md`, "Wave 1"), and reports. The lead merges in the order core, pop, flora, motion.

| Round | Agents | Why this pairing |
| --- | --- | --- |
| A | `core`, `flora` | Core decides which plants exist; flora draws them. Neither touches the other's files. |
| B | `pop`, `motion` | Pop needs core merged (its census reads guilds). Motion is independent. |

After round A: check that Kodya lives (`life::of` returns real life) and that the old effects tests still pass.

### 2. The animation gate (lead)
Build `examples/zoo.rs` (untracked, like `smoke.rs`): Kodya's species in a line, walking a circle. The owner looks at walk, trot and gallop. Planted feet must not slide. Nothing in wave 2 starts before the owner has seen it.

### 3. Wave 2: two rounds
| Round | Agents | Owns |
| --- | --- | --- |
| C | `live`, `guide` | `live`: `src/fauna/` except `motion/`, `mesh.rs`, `signs/`, `guide/`, and `src/effects/creatures/habitat.rs`. `guide`: `src/fauna/guide/`, key K. |
| D | `signs`, then the planet waves | `signs`: `src/life/signs.rs`, `src/fauna/signs/`, creature calls in `src/audio/`. The planet waves follow `PLANETS.md`, "Parallel plan", two agents at a time. |

### 4. Release 0.11.0
As in `HANDOFF.md`, "Steps to finish", step 6, with the release flow proven for 0.10.1: the full gate, a dry run of `release.yml` on a throwaway branch, the tag, the Space redeploy through the Dokploy CLI on `gare.facile.studio`, and the landing version line.

## Brief template for an agent

Each agent gets: its worktree and branch, its build env line, the files it owns (from `CONTRACT.md`), "read `AGENTS.md`, `handoff/CONTRACT.md` and this file first", its gate, the commit rules (Conventional Commits, no co-author line), and the report format from `CONTRACT.md`, "Final report". It never pushes, never merges and never touches another worktree.
