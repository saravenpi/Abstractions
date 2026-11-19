# Ecosystem update: contract for every agent

Read this whole file first. The full plan with research sources is `/home/yann/Code/Saravenpi/eco/PLAN.md`. When the plan and this file disagree, this file wins: scope grew after the plan was written. The field guide, tracks and signs, and plant growth shapes are now in scope.

## The goal
Living planets get seeded species: plants shaped by climate and tinted by their star, and animals built from rigid parametric parts. Animals live in herds that are pure functions of the universe clock, and move with procedural gaits, IK feet, spring bodies and follow-the-leader chains. It ships as v0.11.0.

## Repo and branches
- Main checkout: `/home/yann/Code/Saravenpi/Abstractions`, branch `ecosystems`. Only the merger (the lead session) touches it.
- You work in your own worktree, `/home/yann/Code/Saravenpi/eco-<name>`, on branch `eco-<name>`, made from `ecosystems`. Commit there, in small commits. Never push. Never merge. Never touch another worktree.
- Another Claude session ships 0.10.1 in parallel. Never edit: `src/map`, `src/saves`, `src/world/mod.rs`, `src/menus`, `src/avatar`, `src/companion`, `src/gear/fx`, `src/player`, `src/net`, `crates/`, `src/lib.rs`, `CHANGELOG.md`, `README.md`, `ROADMAP.md`. Reading them is fine.

## Building and testing
- Use the shared target, **but with your own crate version**, or cargo runs another agent's test binary (wiki note: copies share one artifact hash). Once, in your worktree (this sets the workspace version, line 16 of the root `Cargo.toml`; cargo updates `Cargo.lock` itself on the next build):
  ```
  sed -i 's/^version = "0.10.0"/version = "0.10.0-eco-<name>"/' Cargo.toml
  git update-index --skip-worktree Cargo.toml Cargo.lock
  export CARGO_TARGET_DIR=/home/yann/Code/Saravenpi/Abstractions/target
  ```
  Never commit `Cargo.toml` or `Cargo.lock`. Before you finish, run `git update-index --no-skip-worktree Cargo.toml Cargo.lock && git checkout Cargo.toml Cargo.lock`.
- Always pass `--features dev`. Without it cargo rebuilds all of Bevy.
- Check that the output has a `Compiling abstractions v0.10.0-eco-<name> (<your worktree>)` line and your own test names. Do not trust an "ok" without them.
- Cargo holds one lock on the shared target, so a build can wait minutes. Never wrap cargo in `timeout`. For fast loops on pure modules, use a shim crate in your own scratch directory (wiki: `mycelium memory search "shim crate harness"`). A `bevy` crate that re-exports `glam` as `bevy::math` and `bevy::prelude`, plus symlinks to your files, gives a test run of about 10 s.
- Do not run the game, and do not run `examples/`. The merger takes the screenshots.
- Your gate before your final commit: `cargo fmt`, `cargo clippy --features dev --all-targets -- -D warnings`, `cargo test --features dev <your module paths>`, and `filet check .` with no new finding in your files.

## Code rules (filet enforces most of them)
- Files of at most 250 lines, at most 8 functions per file, at most 5 parameters, nesting at most 4, cognitive complexity at most 10, functions of at most 30 lines, lines of at most 120 characters, at most 14 struct fields.
- A doc comment on every item (`///`, and `//!` at the top of each file). **No inline comments and no comments inside function bodies.** No TODO.
- Plain, factual doc comments that say what the item is or returns, in the style of the existing code.
- Remove dead code. No `unwrap()` on anything that can fail at runtime.
- Tests go in `tests*.rs` files or `#[cfg(test)] mod tests`. Every generator has a same-seed test. Non-trivial logic gets one small test that fails if the logic breaks.
- Seeded randomness only: `crate::universe::hash::{child, mix, Rng}`. No thread RNG, no wall clock, no frame count in `src/life`.
- `src/life` is pure: plain data and `bevy::math` only. No ECS, no `Res`, no assets.
- On a sphere: render `y` is not up. Use `crate::space::{up, altitude}` and `Terrain::mesh_height` for the visible ground (wiki: projects/abstractions, "rebase contract" and "mesh_height").
- New modules: you own the files listed for you. You may create child modules under a file you own (`foo.rs` declares `mod bar;` in `foo/bar.rs`). Never edit a `mod.rs` you do not own. If you truly need a change in a shared file or type, write it in your final report and do not make it.

## Shared types (already committed, commit 5688406)
- `src/life/kinds.rs`: `Rung`, `Solvent`, `Energy`, `Structure`, `Chemistry`, `Guild`, `Habitat`, `Activity`, `Plan`, `GlowUse`, `Glow`, `Build`, `Relation`.
- `src/life/types.rs`: `SpeciesId`, `TissueKind`, `Reaction`, `Pattern`, `Tissue`, `Role`, `Tolerance`, `History`, `Species`, `Life`.
- `src/life/body/mod.rs`: `PartKind`, `ShapeKind`, `Part`, `Knee`, `Leg`, `Chain`, `Wings`, `Locomotion`, `FeatureKind`, `Feature`, `Body`. Body frame: +Y up, -Z forward, +X right, origin on the ground under the torso, limbs hang along -Y of their own frame.
- `src/life/world.rs`: `World` (gravity, pressure, temperature, flux, radius, ocean, star kelvin), `World::of(planet, star)`.
- `src/life/mod.rs`: `life::of(planet, star) -> Option<Life>`, which calls `web::assemble`.
- `src/world/globe/life.rs`: `Globe::with_life`, `Globe::life() -> Option<&Life>`. `world/enter.rs` fills it when a planet is entered, so `terrain.globe().life()` is the one source of truth at runtime.
- `src/fauna/mod.rs`: `Creature { species, herd, slot }` component and an empty plugin.
- Fixed signatures other agents call. Keep them exactly:
  - `life::odds::rung(world: &World, seed: u64) -> Option<Rung>`
  - `life::chem::pigment(kelvin: f32) -> [f32; 3]`
  - `life::history::of(seed: u64, role: &Role, mass: f32, rung: Rung, world: &World) -> History`
  - `life::body::build(seed: u64, form: (Plan, u8), role: &Role, history: &History, world: &World) -> Body`
  - `life::web::assemble(planet: &PlanetSpec, star: &StarSpec) -> Option<Life>`

A type change you need goes in your report, not in the code.

## Owner decisions
- Version 0.11.0. The roadmap is renumbered by the merger.
- Kodya, the home planet (`universe::home`, seed from `HOME_*`), lives at rung `Skeletal`. One herd of the most common grazer always walks within 300 m of the robot's wake point.
- The robot is not prey. Animals notice it: curious ones approach, shy ones keep away, herds part, and a parent stands between it and its young.
- Multiplayer: no protocol change. Herds are pure functions of (seed, herd, universe clock `crate::space::Clock`), so every player sees the same herd. Local reactions consider every robot, local and remote (`crate::net` `Remote` entities: read only).
- UI is black and white, pixelarticons only, `crate::ui` widgets, no rounded corners.
- The field guide opens with K (the merger adds the bind after 0.10.1 lands).

## Wave 1 (now, in parallel)
| Agent | Worktree | Owns |
| --- | --- | --- |
| `core` | eco-core | `src/life/{odds,chem,tissue,guild,web,names,relate}.rs` and children |
| `pop` | eco-pop | `src/life/{history,census,herd}.rs` and children |
| `body` | eco-body | `src/life/body/` (all of it, `mod.rs` types only extendable, not changed), `src/fauna/mesh.rs` (new; the merger adds its `mod` line) |
| `motion` | eco-motion | `src/fauna/motion/` (all of it) |
| `flora` | eco-flora | `src/life/flora.rs` and children, `src/world/propmesh/`, `src/world/scatter/`, `src/world/lands/` |

## Wave 2 (after wave 1 merges)
| Agent | Owns |
| --- | --- |
| `live` | `src/fauna/` except `motion/`, `mesh.rs`, `signs/`, `guide/`; `src/effects/creatures/habitat.rs` |
| `signs` | `src/life/signs.rs`, `src/fauna/signs/`, `src/audio/` creature calls only |
| `guide` | `src/fauna/guide/` |

## Final report (your last message)
Commits (hash and subject), what works, test names and counts that passed, filet result for your files, anything you could not do, and any change you need in a file you do not own.

## Cross-agent agreements (commit dc4097f)
- **Home planet:** `planet.name == crate::universe::HOME_PLANET_NAME` and `planet.id.moon == 0`. Kodya is forced to `Rung::Skeletal` in `web::assemble` (owner: core).
- **Producers are species too.** Each producer species stands for one plant group, with an empty `Body` (no parts), `role.guild == Guild::Producer`, `role.level == 1.0`, and `plan == Plan::Colonial`. Its `archetype` is the growth architecture, in this fixed order: 0 trunk and branches, 1 rosette, 2 mat, 3 vine, 4 bulb, 5 stalk and cap, 6 kelp, 7 crystal lattice. Core picks which architectures exist from the climate and the rung. Flora draws them.
- **Globe life states:** `globe.lives()` is `None` when no life was given (test globes from `Globe::new`, far views, the minimap), and then the current home rules apply unchanged. `Some(false)` is a dead planet: no plants, no creatures. `Some(true)` with `globe.life()` drives everything. Kodya keeps its current plant tables and look.
- **Sites:** pop owns `life::census::Site`, the climate at a point that census and herds read: height above sea level, temperature in kelvin, moisture 0 to 1, `crate::world::Biome`, whether it is liquid (sea or pond), daylight 0 to 1. The runtime (wave 2) builds it from `Terrain` and `crate::climate`.
- **Herds:** pop owns `life::herd`. Herd centres are unit directions on the planet (`DVec3`), and members are offsets in metres in the tangent plane. Functions take the universe time in seconds (`f64`).
- **Motion:** the motion agent works in a local flat frame, +Y up, with a ground function `Fn(Vec2) -> f32` (height in local `y` at local `(x, z)`). It returns part transforms in that frame. The runtime converts frames.

## Kills (owner decision, added after wave 1 started)
- Hunts are pure functions of (seed, herd, universe time), owned by pop in `src/life/hunt.rs`: hunt times per hunter herd, target prey herd, stalk and chase windows, outcome by hash roll (kill a slot, or escape), carcasses that draw scavengers and decomposers, bones as signs, and herd counts reduced by recent kills and recovering over time. Every player sees the same hunt with no network.
- The runtime (wave 2, `live`) only draws the hunt timeline: stalk, sprint, catch, the carcass, and scavengers feeding. The robot cannot change an outcome in 0.11.
