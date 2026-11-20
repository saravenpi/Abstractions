# Plan: finish 0.11 with parallel agents that do not compile

Written 2026-11-20 by the lead. Every agent reads this file, `AGENTS.md`, and `handoff/REQUESTS.md`, then its own package below.

## Goal

Land every open owner request of `handoff/REQUESTS.md` on `integrate`, so the owner can play 0.11 as soon as possible.

## Why

The owner wants to play now. Builds are the slow part: six agents testing at once took over an hour each. So agents only write code, and the lead compiles, tests and looks at the game once, at the end.

## Assumptions

- Base: branch `integrate` at the commit that holds this file. It includes the theme tokens and the icon enum of `ui-c` phase 1 (`aa71dea`).
- `fix-ship`, `fix-world`, `fix-sky`, `ui-c` (phase 2) and `combat` are still running on their own branches. Their files are listed under "Taken" below. The lead resolves the conflicts.
- Protocol 3 and `SAVE_VERSION` 3 are not released yet, so their formats may still change without a new version number.

## Approach

One agent per package, one worktree and one branch per agent, disjoint files. Each package is specified so that nothing is left to guess: the spec file, the files it owns, what to build, what to write as tests. Agents read the code they call instead of running the compiler.

## Rules for every agent

1. **No cargo build of any kind.** Not `cargo check`, `cargo clippy`, `cargo test`, `cargo run`. Allowed: `cargo fmt --all` and `filet check .` (both fast, neither compiles).
2. **Read instead of compiling.** Before you call a function, read its signature. Before you use a Bevy 0.20 API, find a use of it in this repo and copy its shape; if there is none, read it in `~/.cargo/registry/src/*/bevy_*-0.20.1/`. Never write an API from memory. In this Bevy: events are `Message`, read with `MessageReader`, written with `MessageWriter`, registered with `app.add_message::<T>()`; a type cannot derive both `Resource` and `Component`; `ChildOf` and `.parent()`; two queries reaching one component mutably need `Without<T>` filters.
3. **Stay in your files.** The "Owns" list is yours. Anything else: make the smallest change and list it in your report. Never touch a file under "Taken".
4. **Code rules of `AGENTS.md`:** a file has at most 250 lines and 8 functions, a function at most 30 lines and 5 parameters, lines at most 120 characters; `///` on every item, `//!` at the top of every file; no comment inside a function body; no TODO; no dead code; no new crate; tests in their own files; tuning constants in a `tune.rs` with units; every system has a run condition; no allocation per frame in a hot path.
5. **Shaders are `.wesl`.** Nothing compiles them before the lead runs the game. Reserved words (`patch`, `filter`, `sample`, `meta`, `module`, `texture`, `target`...) are not names. Check every type, every vec size, every literal (`1.0` not `1` for f32). A uniform struct in a shader and its Rust twin keep the same fields in the same order.
6. **Tests:** write the tests your package lists, in their own files, without running them. When a change replaces a behaviour on purpose, rewrite its test to the new behaviour and list it. Never weaken a test.
7. **Owner rules:** what the player sees is real, one source of truth per thing, nothing is decoration. No borders, no rounded corners, the Undefined font never bold. Physics over tuned constants.
8. **Git:** your worktree, your branch, no push. Conventional Commits (`type(scope): summary`, lowercase, under 72 characters), no co-author line, no "Generated with" line, one logical change per commit. Run `cargo fmt --all` before each commit.
9. **Report (short):** branch, commits, what you built against your package point by point, what you left out and why, tests written or rewritten, "Unverified": every line you are not sure compiles or behaves (file:line and why), and what the owner should look at in the game.

## Taken (other agents are editing these)

| Agent | Files |
| --- | --- |
| `fix-ship` | `src/ship/**` tests and `src/ship/route/align.rs`, `src/computer/tests_*`, `src/companion/**`, `src/render/warp/**` |
| `fix-world` | `src/world/globe/drainage*`, `src/world/globe/shore.rs`, `src/world/atlas/**`, `src/home/**` except `lights.rs`, `src/world/terrain/**`, `src/world/scatter/**`, `src/world/structures/**`, `src/world/chunks/` tests |
| `fix-sky` | `src/render/**` tests, `src/weather/**`, `src/render/starfield*`, `src/render/sky.rs`, `src/render/sky.wesl` |
| `ui-c` | `src/ui/**`, `src/menus/hud/**`, `src/menus/options/**`, `src/settings/model.rs`, `AGENTS.md` design rules |
| `combat` | `src/avatar/strike/**`, `src/avatar/hurt/**`, `src/fauna/attack/**`, `src/fauna/senses/blood.rs`, `src/effects/pool.rs`, `src/life/` blood functions |

## Packages (all parallel)

### `ui-a`: the Go flow and the planet card

- Spec: `handoff/specs/UX.md` sections 5.1, 5.2, 5.8, 5.9 (WP-A), 6.6, 6.7 (the sentences that live in `src/computer/`), 6.9, 6.10.
- Owns: `src/computer/{route,course,keys,home,detail,hold,facts,giant,words,odds,readout,sift}.rs`, `src/ship/route.rs`, `src/ship/route/**` except `align.rs`, `src/ship/pilot.rs`, the cancel line of `src/ship/helm.rs`, `src/map/{deeds,action}.rs`, and the tests of those files.
- Build: one "Go" action that sets the course and engages (key G, Enter in the map), the same key is "Stop" while a course runs (`Cancel`); `Block::OnFoot`; delete `Block::Surface` and let the climb of `src/ship/route/ascent.rs` serve every target; the exact sentences of 5.2 through one function `computer::card_line(...) -> (bool, String)` (true when Go can start); an H key and a "Home" action that flies home in one command, chaining legs if `goal::resolve` cannot route a far body; the planet card of 6.9: four groups of two rows, each row icon (`crate::ui::Icon::{Radius,Gravity,Temperature,Pressure,Air,Day,Year,Water,Life}`), label, value with unit, comparison ("1.2 x home"), a plain meaning per group, rarer facts behind "More".
- Tests: 5.9 WP-A list and 6.10.

### `ui-b`: pins and the map

- Spec: `handoff/specs/UX.md` sections 5.3, 5.4, 5.8, 5.9 (WP-B), 6.4 (pin glyph, `src/map/pins_draw.rs`), 6.6, 6.7 (sentences in `src/map/`), 6.10.
- Owns: `src/map/**` except `deeds.rs`, `action.rs` and `raster/{tiles,tile_paint,paint}.rs`; `src/computer/{view,layout,chrome,section,tab,typing,orbits}.rs` and `src/computer/pointer/**`; the pin model in `src/saves/model.rs` (pins only) and its migration.
- Build: one pin system `Places` with `drop`, `remove`, `rename`, `active`, `set_active`, at most 8, replacing `Waypoints` (`src/map/waypoint.rs`) and `Favourites` (`src/map/favourite.rs`); B drops a pin at the feet or removes one within 10 m; right-click on the ground in the map drops one; the card of a pin has Go, Rename (R), Remove (Delete); delete the context menu (`src/map/menu/`), the Back button, the Left/Right level switch, F2 and Delete-as-clear; level changes by the breadcrumb only; a footer legend line with the keys; the empty and loading sentences of 6.7; pins drawn as the `Icon::MapPin` glyph in `theme::PIN` with no filled square; remove the border and radius at `src/computer/orbits.rs:51-52`; old saves load their waypoint and favourites as pins. Keep `MarkerKind::{Waypoint,Favourite}` filled from `Places` (the HUD belongs to `ui-c`), and expose `Places::active()` for it.
- Tests: 5.9 WP-B list and 6.10.

### `light`: dark zones

- Spec: `handoff/specs/light.md`, "Bug 1".
- Owns: `src/render/{lighting,camera,shade}.rs`, the `relit` function and its helpers in `src/render/daylight.wesl`, `src/world/chunks/{fade,dither}.rs`.
- Build, in this order: (1) no shadow casting from a chunk that is fading out or coarser than the shadow range: insert `NotShadowCaster` while a mesh carries the leaving or hiding range, remove it after; (2) one source for the sun at the camera: the scene `DirectionalLight` illuminance and the `eye` term of `relit` come from the same function, so the correction is exactly zero at the camera, and the correction is scaled by the shadow term so it never removes light a shadow already removed nor re-lights a shadow; (3) shadows fade out over the last fifth of the last cascade instead of stopping at a line, and add a third cascade if the ground still shows a line on a flat plain (state the bounds in metres in `tune`); (4) SSAO: raise `constant_object_thickness` so chunk skirts and LOD seams do not cast halos, and fade the effect out with clearance above the ground; if it cannot be made clean, turn the default off in the settings default and say so; (5) the moon casts shadows only when its light is above a stated lux; (6) the scene lights and the place light update in the same schedule, after the anchor rebase.
- Tests: pure functions for the shadow fade and the correction (zero at the camera; never negative under full shadow), and that a fading chunk has `NotShadowCaster`.

### `colour`: one ground colour at every altitude

- Spec: `handoff/specs/light.md`, "Bug 2".
- Owns: `src/render/{seasons,season}.rs`, `src/world/mesh/tris.rs`, `src/world/ground.wesl`, `src/world/lands/paint.rs`, `src/world/colormap.rs`, `src/map/raster/paint.rs`, `src/render/bodies.wesl`, `src/world/globe/far/ball.rs`, `src/world/globe/site/ball.rs`.
- Build: (1) the season no longer repaints the shared lookup from the camera: it is a tint per fragment in `ground.wesl` from a small uniform (the planet's season state) and the fragment's own latitude and height, the same at every camera height; delete the repainting; (2) surface class (rock, snow, sand, biome) comes from the height field at a fixed analytic scale at the cell centre, the way `Globe::glance` does, never from the facet of the current LOD; per-triangle `tone` only on the finest levels and with zero mean; (3) one colour function for the near mesh, the far mesh, the ball and the map: `src/map/raster/paint.rs` calls the mesh path and loses its own constants and its fake north-west light; (4) the mean turf carpet colour is baked into the vertex colour of every level, and the shader adds only detail around that mean; (5) the ball is lit by the same `sun_response` as the ground and reads the same seasoned colours.
- Tests: the same direction on the globe gives the same linear colour (within 2 percent) from the level-0 mesh path, the coarsest mesh path, the ball texel and the map pixel, for a meadow, a mountain top, a beach and a snow cap; the season tint of a point does not depend on the camera.

### `perf-grass`

- Spec: `handoff/specs/perf-world.md` items 1, 2, 6.
- Owns: `src/world/blades/**`, `src/world/chunks/place.rs`.
- Build: filter the footprints to those that can touch the tile before the blade loop; build tiles in `AsyncComputeTaskPool` tasks collected like `src/world/chunks/turf.rs` (read it first), at most 2 landing per frame; the blade colour computed once per blade (vertex stage or vertex colour); the fragment shades a blade with the sun term, the sky term and the shadow of the ground under it, not the full PBR loop per fragment, with the same look at a glance; `Meadow.carpets` evicted with its tiles; `chunks::place` runs only when the anchor rebased or the frame is the star frame.
- Tests: the filtered footprint set gives the same blades as the full set on a tile with props inside, on its border and far away.

### `perf-sky`

- Spec: `handoff/specs/perf-render.md` items 1 to 6, 9.
- Owns: `src/render/clouds/**`, `src/render/galaxy_light.rs`, `src/render/galaxy_light/**` (not their tests), `src/render/bodies/light.rs`, `src/render/atmosphere.rs`, `src/render/dome.rs`, `src/render/entry.rs`, `src/world/sea/water.wesl`, `src/world/sea/follow.rs`, `src/effects/water/foam.wesl`, `src/effects/water/foam.rs`.
- Build: the cover map stages only what a snapshot needs in its period (state the texels per second in `tune`), with the per-snapshot constants hoisted out of the texel loop and one budget shared by all maps; the first build split by face over the compute pool, and a time jump never stages on the main thread; the cloud shell tolerance grows with the distance to the nearest shell point so a high camera gets a mesh 5 to 10 times smaller with no visible facets, and the mesh is built in a task and swapped; the galaxy bake uses the compute pool with at most half the cores and waits a stated minimum between bakes; every per-frame material write builds the new uniform and compares before `get_mut`, and skips hidden bodies; in `water.wesl` gate shore foam, crest flecks and the wake loop by the terms that already make them zero, and compute `turned` first; foam vertices return early when `alive` is 0. No visible change.
- Tests: the staging budget finishes a snapshot within its period; the shell error stays under the tolerance at 2 m, 10 km and 1,000 km.

### `perf-fauna`

- Spec: `handoff/specs/perf-world.md` items 4, 5, 7.
- Owns: `src/fauna/senses/{feel,cues,judge,sense}.rs`, `src/fauna/{ground,place,herds}.rs`, `src/fauna/signs/lay.rs`, `src/fauna/motion/{walk,stance,pose}.rs`, `src/home/lights.rs`.
- Build: each animal senses every third frame, phased by its entity index, with `dt` scaled so meters fill at the same rate; no sight test when the meter is full or the stance is flee; the sight ray located once and stepped in chart coordinates; the height cache at 4096 slots; the herd watch and the trail watch rolled in a task and swapped in, or one derived from the other; scratch buffers reused in walk, stance and pose; `set_if_neq` on the home lights.
- Tests: the meter of an animal sensed every third frame matches the every-frame meter within 2 percent after 3 s.

### `perf-misc`

- Spec: `handoff/specs/perf-actors.md` items 1 to 4, 6, 7, 9 and `perf-world.md` item 7 (`atlas/map.rs:139` is taken: skip it).
- Owns: `src/map/raster/{tiles,tile_paint}.rs`, `src/map/tiles_view.rs`, `src/audio/{sense,shore}.rs`, `src/saves/{capture,snapshot}.rs`, `crates/net/src/leaves/convert.rs`, `src/ship/sky.rs`, `src/avatar/{animate,fx}.rs`, `src/companion/shine.rs`.
- Build: each item's "Fix" line, nothing more.
- Tests: the tile cache does no work on a frame where no tile is wanted; the shore probe is skipped above its ceiling.

### `net-h4`: the new systems on the wire

- Spec: `handoff/HUNT.md` wave H4 (`net`), `handoff/FORMAT3.md`, `handoff/CLOUD.md` ("Remote players do not show crouch or the entry sheath").
- Owns: `crates/net/**` except `src/leaves/convert.rs`, `crates/server/**`, `src/net/**`.
- Build, inside protocol 3 (update `handoff/FORMAT3.md`): crouch and entry heat in the robot and ship records so remote bodies crouch and glow; the robot's condition (10 parts, one byte each) so remote bodies show damage and limp; hits on animals, kills, the kill ledger and carcasses (place, time, taken parts, broken parts) so every player sees the same carcass; a stuck spear; items as recipes for a held weapon. The server relays and stores; it never trusts a client for another player's state. Also `perf-actors.md` item 4, server half: the snapshot clones only what changed.
- Tests: a round trip of each new record at its documented size; an old client byte stream is refused, not misread.

### `ends`: small open items

- Spec: `handoff/CLOUD.md` ("Open in `hunt`", "Later").
- Owns: `src/craft/pack/pick.rs`, `src/fauna/guide/seen.rs`, the field-guide field of `src/saves/model.rs`.
- Build: stone is picked up from a real rock prop under the crosshair, which then is gone, not from thin air; the field guide's seen species are saved with the world and loaded back.
- Tests: picking takes the prop; seen species survive a save round trip.

## Boundaries

- Ask first: any new crate; a protocol or save version beyond 3; excluding tidally locked planets from the home choice; anything that makes a thing on screen decoration.
- Never: raise a frame budget in `src/world/chunks/budget.rs`; weaken or delete a test to pass; push; commit on `ecosystems`, `next` or `main`.

## Done when

The lead has merged every branch into `integrate` and these pass: `cargo fmt --all --check`, `cargo clippy --workspace --features dev --all-targets -- -D warnings`, `cargo test --workspace --features dev`, `filet check .`; the probe shots at 2 m, 10 km, 100 km, 1,000 km and 20,000 km show one ground colour and no dark patch; the owner can go to a planet with one click and one key.

## Risks

- Code written without a compiler will not all compile. The lead's first check will list errors; each goes back to its agent.
- `perf-sky`, `light` and `colour` share ground with `fix-sky` and `fix-world`: expect merge conflicts in `src/render/` and `src/world/`.
- `ui-a` and `fix-ship` both touch `src/ship/route/`.
- Shaders fail only in the running game.

## Skip

- Hunting multiplayer polish beyond the records above, cross-planet respawn, our own ship icon art, macOS "Open with 0.10": after the release.
