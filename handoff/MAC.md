# Running 0.11 on the Mac

Written 2026-11-19 on the owner's MacBook (Apple M2, 8 cores, 16 GB). It replaces the ruche-specific parts of `CONTRACT.md` ("Repo and branches", "Building and testing"). Everything else in `CONTRACT.md` still holds. The repo-wide rules are in `AGENTS.md` on `next`, which this branch gets when 0.10.1 merges in.

## Owner decision, 2026-11-19 (overrides CONTRACT.md and HANDOFF.md)

Kodya is not special-cased. Its life comes from the same odds and rung rules as every planet: no forced `Rung::Skeletal`, and no herd guaranteed near the wake point. If the normal rules do not make an Earth-like home planet live, fix the physics they read and report the numbers to the owner; never add a home-planet override.

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
