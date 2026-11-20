# Owner requests, session of 2026-11-20 (ruche)

Every request the owner made in this session, with its state. Nothing here is done until its line says so. The lead updates this file as work lands. Read it with `handoff/CLOUD.md`.

Branch `integrate` (worktree `~/Code/Saravenpi/wt-int`) holds everything merged so far. Nothing is pushed.

## How the owner wants the work run

- Parallel agents, Sonnet 5.5, each in its own worktree with its own `CARGO_TARGET_DIR`.
- Agents code with `cargo check` only. Tests run once at the end, then failures go back to agents.
- When all agents are done: review everything, fix the last problems, then `/ship` one big new version with every branch merged.
- The bar for all of it: beautiful, realistic, well made, and well optimised. No optimisation may replace a real object with decoration or lower the picture.

## 1. Finish `handoff/CLOUD.md`

| Item | State |
| --- | --- |
| Warp alignment (`warp`): snap onto the course, then spool, never turn in warp | Merged. Tests still failing, `fix-ship` is on it |
| `hunt`: merge `ecosystems` and the crouch commits, fix the 22 failing tests, spear leaves the hotbar in flight | Merged |
| `leaves`: filet clean | Merged |
| `wake`: shader read through for errors | Merged. Never loaded by a GPU |
| `planets`: the four `p-*` branches together, entry glow from `record.mix.main_gas()`, oxygen through `life::chem::air`, tests follow the generated home | Merged |
| `probe`: one tracked `examples/probe/`, stages by `PROBE_STAGES` | Merged, with five altitude stops added (2 m to 20,000 km). Never run |
| Hunting wave H3, creatures: attacks from body parts, part breaking, predators hunt the robot, carcasses, carving, the stuck spear | Merged (`h3-creature`). Never run in the game |
| Hunting wave H3, robot: health by part, visible damage, effects, repair, spare body | Merged (`h3-robot`). Never run in the game |
| Planets wave P2: drainage and rivers, spawn on a coast | Merged (`p2-drainage`) |
| Planets wave P2: protocol 3, `SAVE_VERSION` 3, "Open with 0.10", map at planet scale | Merged (`p2-format`), see `handoff/FORMAT3.md` |
| Planets wave P2: ship caps, climb to orbit, retune for 9.81 m/s² | Merged (`p2-ship`) |
| Planets wave P2: clouds, weather, far view, quadtree, galaxy light | Merged (`p2-sky`) |
| Hunting wave H4 (multiplayer: hits, carcasses, robot damage on the wire) | Not started |

## 2. Requests made during the session

| # | Request | State |
| --- | --- | --- |
| 1 | While swimming, the legs stop kicking when the water jet pushes | Done, `acca30b`, test `avatar::tests::jetting` passes |
| 2 | Real-size planets | Merged, see section 1 |
| 3 | Merge every branch into one big new version | Done on `integrate`. `w-trails`, `w-polish`, `w-flight` and `worktree-agent-a839842f52ca9bac8` hold nothing that is not already in. The website stays on `landing` |
| 4 | Galaxy travel must be realistic: no "weird star" you fly into like a portal. No decorative element in space | The round galaxy speck and the noise band are replaced by one light map integrated through the real star density of every galaxy in sight (`p2-sky`, step 14b). Not yet seen on screen. Prove with shots from inside, the edge and outside a galaxy, and no jump along a warp. Raise the 128 px cube map if a far galaxy still reads as a blob |
| 5 | Optimise the game so it runs surprisingly fast | Three audits done: `perf-world.md`, `perf-render.md`, `perf-actors.md` in `~/.cache/abstractions-targets/int-logs/`. Fixes not started. Biggest: grass blade tiles built on the main thread (the likely cause of the 20 fps), the blade shader, the drainage grid built up to three times and blocking the load, the cloud cover map budget, the cloud shell rebuild, animal senses every frame |
| 6 | Fix the lights: dark zones appear sometimes | Diagnosed, `light.md` in the same folder. Suspects: SSAO on by default over chunk skirts and LOD seams, coarse chunks shadowing fine ones during the LOD fade, the ground light correction ignoring shadows, shadows cut off at 110 m. Fixes not started |
| 7 | Seen from outside, ground colours change with altitude | Diagnosed, same file. Causes: the season repaints the whole planet's colour table from the spot under the camera (certain, under 600 m), rock and snow read from the facet slope so they vanish at coarse LOD, three colour functions (meshes, far ball, map), turf carpet only on near chunks, the far ball has its own light model. Fix: one colour function for every range. Not started |
| 8 | Look up good practices online for 6 and 7 | Done, sources listed in `light.md` |
| 9 | Review and rework the UI and UX of the game, the map and the whole HUD. Carte blanche. Friends were lost launching a warp and using waypoints | Review done: spec in `UX.md` in the same folder (488 lines). Three packages: C (theme, icons, HUD) running on `ui-c`; B (pins and map) and A (Go flow and planet card) start when C lands its tokens |
| 10 | Markers over the world, as in modern games: a pin shows on the screen at its real place; when it is to the side or behind, it slides to the border of the screen with an arrow pointing to it, higher or lower on the border by its real position | Sent to `ui-c` (HUD). Not started |
| 11 | Review the combat animations and effects so combat is dynamic and pleasant. Blood spurts for animals that have blood, in the colour of their blood, only when the hit made an opening that bleeds | On `combat`, written without compiling: blood (colour from the oxygen carrier, wounds only when the flesh is reached, pressure and flow from physics, real drops that land as stains, a "Blood" setting), swing overshoot, chest lead, recovery by weapon inertia, camera kick along the blow, chips off hard layers |
| 12 | The owner wants all of these in this version: the trail at the tip of the weapon, the sound of hits, the robot staggering under a blow, missed and deflected hits that read differently, the animal's whole body moving in the wind-up | Sent back to the `combat` agent. Also still open from its list: blood stains on the blade |
| 13 | H gives stone from nowhere (`src/craft/pack/pick.rs`): stone must come from a real rock under the crosshair, which then is gone. The owner wants it in this version | Agent running on `rocks`: stable prop ids, a saved set of taken rocks, a filter at chunk build, a props rebuild, a crosshair pick |
| 14 | In multiplayer the other players must see everything the same as we do | Sent to `net-h4` as its acceptance test: crouch, entry sheath, robot damage, strikes and trails, blows and stagger, wounds and blood, impacts and sounds, animal attacks, carcasses, stuck spears, taken rocks, pins, and a check that season, weather, clouds, herds and leaf fall come only from the shared seed and clock |
| 15 | Take pretty screenshots of the game and add them to the website in a carousel that shows different aspects of the game | Not started: needs the game running. Shots through `examples/probe` on lavapipe (ground at dawn, forest and grass, sea and shore, animals, the base, flight, clouds from above, orbit, a moon, a galaxy, the map and the new HUD). Site lives on branch `landing` (`site/`): carousel with no borders, no rounded corners, no bold, keyboard and touch, alt text per shot |
| 16 | Third person by default on foot, first person (cockpit) by default in the ship, and the game remembers the last view used | Done on `integrate`: `ViewPrefs` default changed; the last view was already saved by `src/settings/prefs.rs`. `fx-world` makes the tests state their view |
| 17 | Improve the inside of the cockpit, it is simplistic in first person | Agent `cockpit` running: seat, frame, consoles, controls that follow the real inputs, instruments that show the real state, panel light |
| 18 | Redesign the ship: it looks like a child's drawing; real engines and real engine fire; no floating parts | Agent `ship-look` running: one lofted hull, wings with roots, nacelles, nozzles, gear bays, hatch, a test that the mesh is one connected piece, exhaust from thrust and air pressure, colliders that follow |
| 19 | Look up good practices for the sensation of speed, for the ship and the glider, and polish lightly | Agent `speed` running: research to `handoff/specs/speed.md`, then small cues tied to real quantities, with a "Speed effects" setting |
| 20 | After that: a complete review, fix what it finds, improve and polish the animations and everything that was added | After the tests are green: review of the whole diff by area, probe shots, polish pass |
| 21 | Polish the light effects and make them more realistic, from good practices found online, still optimised, with control given to the user for a weak PC | Agent `lightfx` running: research to `handoff/specs/lightfx.md`; exposure that adapts, sky and ground bounce ambient, cloud shadows on the ground, low sun and aerial perspective from the real air, water Fresnel and glint, local lights in physical units; a "Lighting quality" preset plus one switch per costly effect. Low keeps every light and drops only shadows, shafts, occlusion and bloom |
| 22 | Show units and distances correctly in the UI so everyone understands them easily | Sent to `fx-ui`: one formatting module for the whole UI (metres, km, AU, light-years, each explained once on the screen), a trip time beside every far distance, gravity, air and radius as multiples of home with the SI value beside, plain labels, the same number on the marker and on the map |
| 23 | From the owner through the session `abstractions-16`: read the architecture review (`handoff/specs/architecture-review.md`, read at `main` e8f0dd4), make a plan, fix what it lists, fold it into this update; reduce the size of the codebase and optimise the game: less redundancy, one way of doing each thing | Planned as its own wave, after every branch in flight is merged and the tests are green, because it moves and deletes code that eleven branches are editing. See section 6c |

## 3. The UI rework: what the owner asked for

Rules that stay:

- No borders anywhere. No rounded corners anywhere.
- The Undefined font everywhere. NEVER bold: it renders badly.

Rules that change (2026-11-20; `AGENTS.md` still says black and white, update it with this work):

- No more simplistic black-and-white UI. Keep a bit of the art direction, but make a real interface of a modern video game.
- Translucent panels are welcome.
- More icons, real icons, not only pixel-art ones. Cool icons for the ship too.
- Remove the old black backgrounds behind icons.
- Generous margins. Very legible, very clean, very modern.

Things to fix:

- Launching a warp to go somewhere must be obvious. One flow: pick a thing on the map, read what it is and how far, press one "Go" action. If it cannot start, one plain sentence says why and what to do.
- Waypoints must be simple to place, follow and remove.
- In many places the player has to guess. Find each one and remove the guessing: label every action, show its key, explain every state.
- An empty map (no data) must show a text, not just black.
- Planet stats are hard to read. Make them readable at a glance.
- Remove the dots for monuments on the map.
- Markers over the world and on the screen border with an arrow, see request 10.
- The HUD is contextual: it shows only what means something where the player is. No altitude or weather in open space, no orbit readings on foot, no cardinal compass between the stars. One function decides what is shown for each context (on foot, swimming, gliding, ship landed, ship in air, orbit, open space, between stars, warp, map).

## 4. Decisions waiting for the owner

| Question | Default if no answer |
| --- | --- |
| Exclude tidally locked planets (one side in permanent night) from the home choice? One line in `landable` (`src/universe/home.rs`), but it is a special case | Leave the rule as is, say it in the changelog. One test depends on it: `computer::tests_climate::the_card_writes_the_span_of_temperatures` |
| Harvest tool: pick the best tool in the pack, or use the one in hand? | Keep the one in hand |
| Carry limit: 25 kg at 9.81 m/s² | Keep |
| Wake V floored at 4 degrees (physical wedge at flight speed: 1 to 2 degrees) | Keep |
| Field repair key U; the spare body wakes at the base only when the robot dies on the base's planet | Keep |
| Big game is sparse on the chosen homes (herds 2 to 3 km apart). Denser? | Keep |
| Push `integrate` to origin before it is green, to play on the Mac? | Do not push until green |
| Galaxy scale | Decided: real distances, see 6b |

## 5. Tests loosened or rewritten: the owner should judge

- `universe::giant_tests::gas_giants_are_outer`: threshold 95 % to 90 % (measured 94.6 %).
- `fauna::tests_meet::big_game_lives_within_a_walk`: now counts per home, half the homes with big game need a herd within 1.5 km.
- `gear::tests::feel::a_level_pull_bends_the_path_at_once`: 15 degrees to 10 (measured 11.5 at 9.81 m/s²).
- `fauna::rig::SEED`: 1 to 7, because the home of seed 1 has only flyers and swimmers.
- About ten flight tests run longer: same speeds, accelerations take about 1.5 times longer at 9.81 m/s².
- `src/saves/upgrade.rs` is deleted: a v0.5 world no longer starts in the new game, it opens with 0.10.

## 6. Never seen in the game

Everything in section 1 marked "never run", plus: grass blades and their cost, the waterline pass, the foam wake, the entry sheath, call poses, crouch, the galaxy light, the cloud shell, the far depth of the sky and stars, trunk rivers, the map tiles, "Open with 0.10" on Linux (macOS returns `Unsupported`). Shaders only fail in the running game.

## 6b. Real scale everywhere (owner decision, 2026-11-20)

The owner decided: planet sizes, distances and orbits must all be realistic, from size and mass. Realism and optimisation.

What was found: planets are real size and orbit at 0.03 to 40 AU, but the game light-year is 1e10 m and stars are 1e10 to 1e11 m apart, so star systems overlap. The real distance between neighbour stars is about 4e16 m.

State: an audit is running. It checks every scale of `src/universe/` against real astrophysics (galaxy size, star density and masses, Kepler periods, mass-radius, gravity from mass and radius, Hill and Roche limits), lists everything that depends on the galaxy scale (position precision, warp speeds and travel times, starfield, galaxy light, map, saves, wire), and writes a plan with work packages to `~/.cache/abstractions-targets/int-logs/scale.md`. Implementation starts when the four test-fix branches are merged, because one of them is unifying the reach of a star system.

This is part of 0.11: the release waits for it.

## 6c. Architecture wave (request 23)

Source: `handoff/specs/architecture-review.md`, 22 candidates in parts A (celestial and navigation), B (world and actors), C (interface and net), plus Bevy practices (D). It read `main` at e8f0dd4; line numbers have moved, so every finding is checked on `integrate` before it is acted on.

Already done or in flight on `integrate`: the shore search throttle of B1 (`perf-misc`), `SYSTEM_REACH` moved to `universe::system_reach` (A8, `fx-sky` and the scale agents), one units formatter (C4, `fx-ui`), borders and the radius removed (the conflict of C3, `ui-b`), several ungated systems and per-frame writes (parts of C7 and D4, the perf agents), the sun light from the place block (part of A5, `light` and `lightfx`).

Order, when the tree is green:
1. Baseline: counts of systems run per frame and CPU time of the hot systems from the probe, since a GPU frame time on this machine means nothing. A `profiling` cargo profile (release, symbols kept) for the owner's machine with Tracy.
2. Layers: A8 and C8 (move `Screen`, `playing`, `in_control`, `Menu`, the marker types, `Link`, `spin` down), then A7 (one base `math` module: smoothstep, angle wrap, spring).
3. Performance: A1 (one `Here` resource for the current system, moons and planet spec, replacing the private caches), the rest of B1 (one ground answer per frame for the focus point), C7 (stop the 0.5 s chart rebuild, gate the `ui` systems), C6 (settings reactions only on the fields they read, ranges checked on load).
4. Deletions with no change in behaviour: B4 (one mesh builder), B5 (one `Trail<const N>`), B6 (one face module), C1 (one list screen), C2 (one focus stepper and one text field), C3 (screen scaffold, progress bar, show/hide), A2 (one frame and velocity module), A6 (one latitude and longitude module), B2 (one ground reader).
5. Changes the player can see. The lead's proposal, each open to the owner's veto, all following the rule "real, one source of truth": A3 one air density curve for gear, ship, sky and sound; A4 one gravity function, the same for the player and the ship below sea level; A5 retire the phase-based `daylight`, lights, creatures and sounds follow the real sun; B3 structures stand on one height source; B7 one footfall source so sound and dust coincide. Hash and RNG seeding (A7, the cube face in the per-chunk seed) and the wire seam (C5) may change now without cost, because 0.11 worlds, `SAVE_VERSION` 3 and protocol 3 are all new and unreleased.

6. C5, the replicated-kind seam in net. Owner decision relayed by `abstractions-16`: it lands before protocol 3 ships, and any change of bytes on the wire stays inside protocol 3. One kind table shared by the room and the sender (the mapping is written twice today), one trait for the body records' `encode` and `decode`, one generic `Remotes` instead of a map and an arm per kind, the four frame enums reduced, one `lan_ip`, the close reasons worded once. `net-h4` added kinds since the review (`Vitals`, the hunt acts and events), so check them against the same seam. It starts after `fx-net` merges, because that branch is editing the same files.

7. Notes from the session `abstractions-53` (read-only look at this tree, to judge in this wave): `src/world/shapes.rs` gives every terrain vertex a constant tangent (16 of about 50 bytes) and shares no vertex (6,144 stored for 1,089 lattice points per near chunk); `src/world/chunks/build.rs` builds a trimesh collider where `plan.rs` says heightfield; `chunk_heights` in `src/world/mesh.rs` returns `Vec<Vec<f32>>`; entries of `crates/net/src/log.rs` repeat an 8-byte time and a full `BodyId`, and the whole log rides in `Welcome` under a 4 MiB cap; the log and the quarried rocks are text rows in the TOML save. Roadmap 0.13 ("ground you can shape") plans a binary per-chunk terrain file and area-based sync: if protocol 3 leaves room for a `Change::Dug` variant and for not sending the full log on join, 0.13 will not need protocol 4 for that alone. `zstd` is approved by the owner for 0.13, not before.

`ROADMAP.md` on `main` moved again (9e62575, ede5233): 0.12 places worth the trip, 0.13 ground you can shape, 0.14 real space and probes, 0.15 together, 0.16 the thread, 0.17 hunting, mining, crafting and building, then 0.18 to 0.23. Merge `main` again before the release and edit only the 0.11 section.

Each step: tests rewritten where behaviour changes on purpose and listed, never weakened; the gate after each step; the line count of `src/` before and after.

## 7. Order of the remaining work

1. `fix-ship`, `fix-world`, `fix-sky` finish; merge; run the full suite; fix what is left.
2. Fix wave, disjoint files: lights and colours (requests 6, 7), performance (5), UI rework (9 and section 3).
3. Run the probe: shots at 2 m, 10 km, 100 km, 1,000 km and 20,000 km, galaxy shots, frame times before and after.
4. Review the whole diff against `ecosystems`. Update `AGENTS.md`, `CHANGELOG.md`, `README.md`, `ROADMAP.md`.
5. `/ship` version 0.11.0 (protocol 3 and save format 3 are in), then show the new version on `landing`.
