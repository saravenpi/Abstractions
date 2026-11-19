# Ecosystem update plan

## Goal
Every living planet gets its own seeded species: plants tinted by its star, animals built from parametric rigid parts, living in herds that follow a pure function of the universe clock, and moving with procedural gaits, IK feet, spring-driven bodies and follow-the-leader spines.

## Why
- `ROADMAP.md` section "0.10: ecosystems" is unbuilt. v0.10.0 shipped multiplayer instead (`CHANGELOG.md`).
- Today's life is four hard-coded ambient kinds (`src/effects/creatures/habitat.rs`: Firefly, Butterfly, Crow, Gull). They are picked by the six home biomes and never by the planet's own record, beyond `lifeless()` in `src/effects/planet.rs`.
- Nothing walks on the ground. The robot has a good gait (`src/avatar/gait.rs`) but no animal shares it.

## Assumptions
- It ships as **0.11.0**, with the roadmap's 0.10 ecosystems content. The roadmap's 0.11 (hunting, crafting) becomes 0.12. Roadmap headers get renumbered in the last step.
- Work starts from branch `next` (91d8816) once the other agent merges its fixes, or from `physical-planets` if `next` is not ready. Branch name: `ecosystems`.
- The robot is not prey. No combat, no kills, no harvest. So the roadmap's cascades (point 12) wait for 0.12.
- No network protocol change. Creatures agree between players because herds are pure functions of (seed, herd id, universe clock). Only small local reactions differ, and they react to every robot they see, local or remote (`src/net/state` `Remote`), so the difference stays small.
- Home planet Kodya gets rung 4 (skeletal animals, herds, flyers) and a grazer herd within 300 m of the spawn, so the first landing shows life.
- Existing fireflies, butterflies and birds stay as the ambient layer. They only appear where the planet's life rung allows.

## Approach
Two layers, kept apart so the filet limits (250 lines, 8 functions per file) and the shim-crate harness both hold.

1. **`src/life/` (new): pure generators.** Plain data and `bevy::math` only, like `src/universe/` and `src/climate/`. Seeds come from `universe::hash::child(planet_seed, index)`. Every generator has a same-seed test. Chain: life odds and rung, then biochemistry, tissues, species list, niche-model food web, guilds, body plans, life history, names, population field, herd paths.
2. **`src/fauna/` (new): Bevy systems.** Spawning from the population field, the utility brain, steering, the animation rig, meshes and glow. It copies the shape of `src/companion/` (brain, sense, motion and spring split into small files) and of `src/effects/creatures/` (pooled entities marked `Rebased`, run after `Sight`, gated by `playing` and `grounded`).

**Animation model (rigid parts, no skinning, as the roadmap says).** Each species is a tree of parts with sockets. Each part is its own entity with a mesh shared by every individual of the species, so Bevy's automatic instancing batches them. One frame of an individual:

1. **Root motion.** Steering (seek, flee, arrive, wander, separation, cohesion and alignment, after Reynolds) gives a velocity. The root follows it through a second-order system with frequency `f`, damping `zeta` and response `r`. Heavy species get low `f` and `zeta` near 1 (they lumber). Small ones get high `f` and lower `zeta` (twitchy). `r` below 0 gives anticipation.
2. **Gait choice by Froude number** `Fr = v^2 / (g * leg_length)`, with the planet's real `g`: walk under about 0.5, trot to about 2.5, then gallop or bound. The same species walks more slowly on a heavy world without any rule written for it. The duty factor falls with speed: above 0.5 at a walk, 0.3 to 0.5 at a trot, under 0.4 at a gallop.
3. **Phase offsets by leg count.** Biped 0 and 0.5. Quadruped walk: each leg lags the one before by a quarter stride. Trot: diagonal pairs together, 0.5 apart. Hexapod: alternating tripod. Myriapod: a metachronal wave down the body. The offsets are data in a gait table, not code.
4. **Foot planner.** Each foot has a rest point in the body frame, projected on `Terrain::mesh_height` (the visible ground, see wiki `projects/abstractions`). A planted foot never moves. A foot lifts when its phase window opens and it is past its step threshold. It aims at the rest point predicted ahead by `velocity * swing_time` and swings on an eased arc whose lift scales with leg length. Planted feet never slide: same rule and same test as `src/avatar/gait.rs`.
5. **Body over feet.** Height, pitch and roll come from the planted feet (the pelvis adjust), smoothed by a spring.
6. **Leg IK.** An analytic two-bone solver in 3D with a pole vector. Knees point forward for mammal-like legs, backward for bird-like legs, and up and out for arthropods. Three-segment legs fix the last segment's angle and solve the rest.
7. **Spine, neck and tail.** Follow-the-leader distance-constraint chains with an angle limit per joint. The tail lags through springs. Long necks and tentacles use FABRIK, 2 to 4 iterations.
8. **Look and idle.** The head turns toward the attention target with a clamp. A seeded idle layer adds breathing scale, weight shifts and flicks of ears and antennae.
9. **Other body plans.** Slimes squash and stretch along the velocity at constant volume (`sx * sy * sz = 1`). Jellies pulse: a fast contraction gives thrust, then a slow relax, and the tentacles are chains. Worms and swimmers carry a travelling body wave whose frequency scales with speed over length. Swarms are boids. Crystal walkers hop and tumble as rigid bodies. Flyers reuse `src/effects/creatures/anim.rs` (`wing_angle`, `glide`) and bank with their turn rate.

**Level of detail.** Under 40 m: full IK, ground samples per foot, 30 Hz. From 40 to 150 m: gait phases on root height only, no per-foot ground samples, every other frame. Past 150 m: herds only as points in the population field, nothing drawn. Individuals fade in and out with the existing `VisibilityRange` dither (wiki `tools/bevy`: add it before the first draw).

**Meshes** are built once per planet off the main thread, the way `src/render/clouds/maps.rs` uses `AsyncComputeTaskPool`. They are flat-shaded `Soup`s from `src/avatar/mesh/soup.rs`, which moves to a shared spot (step 9).

The other way to do this, skinned meshes with Bevy animation graphs, is rejected: the roadmap rules out skinning, the art is low-poly rigid parts, and per-part entities batch for free.

## Steps

### Phase 0: the contract (alone, before any fan-out)
1. `ecosystem-contract.md` (new, in the scratchpad): type names, file owners, the gait table format, the part and socket format, the filet limits, "use `--features dev`", "do not run the game", one `CARGO_TARGET_DIR` per worktree. Verify: every agent prompt in later phases links to it.
2. `src/life/mod.rs` (new), `src/fauna/mod.rs` (new), and one `mod life; mod fauna;` line each in `src/lib.rs`: empty plugins plus the shared types (`SpeciesId`, `Rung`, `Tissue`, `Guild`, `Plan`, `Species`, `Part`, `Socket`) as stubs. Commit before fan-out, so no two agents ever own a `mod.rs`. Verify: `cargo check --features dev` passes and `filet check .` reports nothing new.

### Phase 1: prove the animation first (one agent)
The look and feel is the riskiest part, so it fails or passes first, on one hand-built quadruped.

3. `src/fauna/motion/spring.rs` (new): a second-order system with `f`, `zeta`, `r`, stable at any `dt` (k2 clamped as in t3ssel8r's write-up). Verify: `cargo test --features dev fauna::motion::spring` shows a step response that settles within 2 percent by `4 / (zeta * 2 pi f)` s and no blow-up at `dt = 0.1`.
4. `src/fauna/motion/gait.rs` (new): the gait table (offsets, duty factor by Froude band) and the phase clock. Verify: a test shows a quadruped at `Fr = 0.2` lifts legs in the order LH, LF, RH, RF, a quarter apart, and at `Fr = 1.5` pairs the diagonals.
5. `src/fauna/motion/feet.rs` (new): the foot planner and swing arc. Verify: a test walks 50 m over a sloped height function and asserts no planted foot moves more than 1 mm, and no foot sinks below the ground by more than 2 cm.
6. `src/fauna/motion/ik.rs` (new): the 3D two-bone solver with a pole vector, plus FABRIK for chains. Verify: a test asserts every reachable target is met within 1 mm, unreachable targets give a straight limb toward them, and the knee stays on the pole side.
7. `src/fauna/motion/chain.rs` (new): follow-the-leader spine and tail with angle limits. Verify: a test turns the head 180 degrees and asserts segment lengths are kept to 1e-4 and no joint passes its limit.
8. `src/fauna/rig.rs` (new): turns (root, feet, chain states) into part transforms. `examples/zoo.rs` (new, untracked, added to `.git/info/exclude` like `smoke.rs`): one test quadruped walking a circle on Kodya, screenshots at walk, trot and gallop. Verify: run it with the lavapipe smoke script from wiki `tools/bevy`, read the screenshots, and check by eye that feet stay put and the body bobs. **Stop and show the owner the screenshots before phase 2.**

### Phase 2: pure generators (parallel, one file set per agent, all in `src/life/`)
9. (parallel A) `odds.rs`, `chem.rs` (new): life or not, the rung (star age derived from the star seed through `hash::child`, no change to the star record), and the biochemistry roll from `PlanetRecord`. Verify: a test over 10,000 seeded planets gives about one living system in three. Lava, barren and gas giant planets never live, and Kodya is rung 4.
10. (parallel B) `tissue.rs`, `web.rs`, `guild.rs` (new): tissue records, a niche-model food web (Williams and Martinez 2000) with 12 to 40 species scaled by productivity and rung, and guilds. Verify: tests check same seed gives the same web, connectance between 0.05 and 0.3, at least one producer and one consumer on every living planet, and no consumer without prey.
11. (parallel C) `body/plan.rs`, `body/rules.rs`, `body/parts.rs` (new): six body plans with 2 to 4 archetypes each, the rule table from the roadmap (square-cube with gravity, air density for flyers, Bergmann and Allen with cold, island dwarfism, eyes from light), features as data, symmetry always, at most two accent tissues. `src/avatar/mesh/soup.rs` moves to `src/render/soup.rs`, with a `pub use` left in `avatar/mesh` so avatar code is untouched. Verify: tests check every species is bilaterally or radially symmetric, under 600 triangles and 16 parts, and land animals on a 2 g world have thicker legs than the same archetype at 0.5 g.
12. (parallel D) `history.rs`, `census.rs`, `herd.rs`, `names.rs` (new): life history from mass, the population field (density in proportion to `mass^-0.75`, about a tenth per trophic level), and herd paths as pure functions of (seed, herd, clock). A herd walks a route between waypoints (water, grazing) picked by `hash(herd, floor(t / leg_duration))`. Names reuse the style of `src/universe/names.rs`. Verify: tests check the same (seed, clock) gives the same herd position on two calls built from scratch, a herd never stands in the sea or above the snow line it does not tolerate, and the apex is the rarest level.

### Phase 3: life in the world (parallel, `src/fauna/`)
13. (parallel E) `fauna/build.rs` (new): species meshes per planet on `AsyncComputeTaskPool`, one mesh per (species, part), shared materials. Verify: a headless test app builds Kodya's species in under 50 ms on the pool, and the cost test pattern of `src/climate/tests_cost.rs` holds.
14. (parallel F) `fauna/spawn.rs`, `fauna/pool.rs` (new): individuals appear from `census` and `herd` within 150 m, capped at 48, entities marked `Rebased`, dither fade. Verify: `src/effects/creatures/tests_rebase.rs` copied as `fauna/tests_rebase.rs` passes, and a test asserts no more than 48 individuals or about 700 part entities.
15. (parallel G) `fauna/brain.rs`, `fauna/sense.rs`, `fauna/steer.rs` (new): needs-based utility (hunger, thirst, fear, rest, company, curiosity, care) with response curves, weights from guild and life history. Steering, and reactions to every robot: curious ones approach, shy ones keep away, herds part around it, a parent stands between the robot and its young. Verify: tests check a grazer at full fear flees a robot at 10 m, the herd centre stays within 2 m of the pure herd path once no robot is near, and the young stay inside the herd's hull.
16. `fauna/motion/plans.rs` (new, after 13 to 15): slime, jelly, worm, swimmer, swarm and crystal motion. Verify: tests check slime volume stays 1 within 1e-3, a jelly moves only during contraction, and a worm's segments follow the head's path within 5 cm.

### Phase 4: tie-ins (after phase 3 merges)
17. `src/effects/creatures/habitat.rs`: `presence_on` reads the life rung (no butterflies under rung 3, no birds under rung 4). Verify: `cargo test --features dev effects::creatures` passes with the existing home results unchanged.
18. `fauna/glow.rs` (new): bioluminescence by rule (lure, warning, courtship, belly light), coloured by biochemistry, emissive only, no point lights (wiki: Solari has none). Verify: a test checks glow appears only on night-active species or dim-star planets.
19. `fauna/calls.rs` (new) plus `src/audio/cue.rs`: calls synthesised from body size and air density through `AudioCue`, and a dawn chorus. Verify: `cargo test --features dev audio` passes and the RMS and peak stay under the limiter.
20. Plants: pigment tint by star colour on non-home planets in `src/world/lands/thrive.rs`. Verify: a test checks a red dwarf planet's leaf colour is darker and redder than Kodya's.

### Phase 5: ship
21. `examples/smoke.rs`: creature stops on Kodya, one rung-2 planet and one ice planet. Verify: screenshots read by eye. Run `mise run check`, which must be green.
22. `CHANGELOG.md`, `README.md`, `ROADMAP.md` (renumber), release 0.11.0 through the release flow. Verify: `mycelium flow run release-preflight` passes, then the GitHub release has all assets.

## Boundaries
- Ask first:
  - The version number (0.11.0) and renumbering the roadmap.
  - Kodya's tuned fauna (rung 4, herd near spawn).
  - Moving `soup.rs` to `src/render/`.
  - Any change in `crates/net` or the protocol. The plan needs none.
  - Plant growth architectures (roadmap point 4). Only the pigment tint is in scope.
  - The field guide panel.
  - Adding any new crate. The plan needs none.
- Never:
  - Edit files owned by the other agent's live worktrees: `src/map`, `src/saves`, `src/world/mod.rs`, `src/menus`, `src/avatar`, `src/companion`, `src/gear/fx`, `src/player`, `crates/net`.
  - Use a thread RNG or the wall clock in `src/life`.
  - Store an `Entity` in anything shared.
  - Raise a filet limit or weaken an existing test.
  - Run the game in parallel agents. Only the merger runs `examples/zoo.rs` and `examples/smoke.rs`.

## Done when
- Landing on Kodya shows a herd of grazers walking, trotting and fleeing with planted feet. A different living planet shows different animals that fit its gravity, cold and light. A dead planet shows none.
- Two players side by side see the same herd in the same place.
- `mise run check` is green.
- The frame cost of `fauna` stays under 2 ms with 48 individuals (measured in the zoo example with Bevy's diagnostics).

## Risks
- **Looks.** Procedural animals can look like furniture on legs. Phase 1 exists to catch that early, with the owner looking at screenshots.
- **Cost.** About 700 part entities with transforms every frame. Watch the transform propagation cost. If it is too high, merge distant parts into one mesh per individual at the 40 to 150 m LOD.
- **Divergence in multiplayer.** Local reactions differ by up to the network delay. Acceptable while creatures are not prey. 0.12 combat will need authority.
- **Merge conflicts** with the other agent's branch on `src/lib.rs` (one line) and `src/effects/creatures/habitat.rs`. Keep those edits tiny and last.
- **Sphere frames.** Feet and herds must read `space::up` and `Terrain::mesh_height`, never render `y` (wiki `projects/abstractions`, rebase contract).

## Skip
- Cascades from kills, signs (tracks, droppings), the field guide, the robot as prey: these need 0.12 combat or the HUD panel the other agent is changing.
- Plant growth architectures: a rewrite of `src/world/propmesh` and scatter, big enough for its own release.
- Moons, under-ice and vent life: roadmap 0.12 "life odds".

## Sources
- Chris Hecker et al., "Real-time Motion Retargeting to Highly Varied User-Created Morphologies" (Spore, SIGGRAPH 2008): https://www.chrishecker.com/Real-time_Motion_Retargeting_to_Highly_Varied_User-Created_Morphologies
- Hecker, "How To Animate a Character You've Never Seen Before" (GDC 2007): https://chrishecker.com/How_To_Animate_a_Character_You%27ve_Never_Seen_Before
- IEEE Spectrum, "Engineering Spore" (legs, symmetry, footfall rhythm): https://spectrum.ieee.org/engineering-spore
- R. McN. Alexander, gait table (duty factors by gait), J. Exp. Biol.: https://cob.silverchair.com/jeb/article-pdf/115/1/263/2432986/jexbio_115_1_263.pdf
- Cartmill et al. 2002, support polygons and symmetrical gaits: https://www.originalwisdom.com/wp-content/uploads/bsk-pdf-manager/2019/10/Cartmill-et-al_2002_Support-polygons-and-symmetrical-gaits-in-mammals.pdf
- Second-order dynamics (t3ssel8r style f, zeta, r), reference port: https://gist.github.com/Andicraft/ab605dda924a0321e755abf8f573ebd7
- Allen Chou, numeric springing: https://allenchou.net/2015/04/game-math-precise-control-over-numeric-springing
- FABRIK walkthrough (Vulkan tutorial): https://docs.vulkan.org/tutorial/latest/Advanced_glTF/Procedural_Animation_IK/03_fabrik.html
- Composable constraint chains in Rust: https://battesonb-github-io.pages.dev/2024/11/16/procedural-animation
- Distance-constraint snake chain: https://observablehq.com/@real-john-cheung/procedural-animation-1
- Tartu course project, FABRIK worm and tentacle with sine offsets: https://courses.cs.ut.ee/2025/cg/fall/Main/Project-ProceduralAnimation
- Craig Reynolds steering behaviours, summary: https://slsdo.github.io/steering-behaviors
- Williams and Martinez 2000, "Simple rules yield complex food webs": https://www.sccs.swarthmore.edu/users/08/bblonder/phys120/docs/martinez.pdf
- Damuth 1981, density and body mass: https://pdodds.w3.uvm.edu/files/papers/others/1981/damuth1981a.pdf
- Dave Mark's Infinite Axis Utility System: https://gameai.com/iaus.php
- Needs-based critters example: https://github.com/NickMercer/UtilityCritters
- Bevy automatic instancing example: https://bevy.org/examples/shaders/automatic-instancing/
- No Man's Sky creature blueprints and size-dependent motion: https://killscreen.com/how-no-mans-sky-paints-18-quintillion-worlds-algorithmic-brush
- Dead end: no primary source found on Rain World's internals. The claim that its limb grip changes gravity response is from a comment thread only, so the plan does not rely on it.
