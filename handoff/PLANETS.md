# Real-size planets and realistic moons (bundled into 0.11.0)

## Goal
Planets and moons get their real sizes (Earth 6,371 km). Terrain is generated at every scale, from plates and continents down to today's local detail. Atmospheres, star systems, orbits and the ship follow physics. The frame budget stays where it is today.

## Why
- Planets are 3 to 40 km in radius today (`src/universe/planet.rs:13` `PLANET_RADIUS`, home 6 km in `src/universe/home.rs:16`). The owner wants real sizes, bigger lands and more realism.
- A code sweep (2026-11-19) found about 30 size-dependent places. These break outright at 6,371 km:
  - The far view never draws the planet. `src/world/globe/far.rs:38,71`: `FAR` is 1e6 m and the hand-over goes negative.
  - Positions sent between players are clamped. `crates/net/src/pack/position.rs:7`: i32 millimetres, which stops at 2,147 km.
  - The air model is wrong. `src/universe/planet.rs:35` `air()` scales home heights by the square root of the radius, which gives clouds at 29 to 49 km. Drag in `src/ship/drag.rs:13` and the sky falloff in `src/render/atmosphere.rs:24` are not exponential.
  - Days are physically impossible. 1,200 s days put the equator above orbital speed.
  - The star system is too small. In `src/universe/` the stars (`HOME_STAR_RADIUS` 185 km) and orbits (`ORBIT` from 5e6 m) are smaller than an Earth. `GIANT_RADIUS` and `MOON_RADIUS` are below it too.
  - The ship's ranges are in radii: `PLANET_LOCK` (3 radii), `CARRY` (50 to 100), `BODY_SWITCH` (25). At Earth size these lock warp for about two hours and make bodies overlap.
- Precision:
  - Face chart coordinates reach 2^22 m. `Terrain::height(x: f32, z: f32)` then has 0.25 to 0.5 m steps.
  - Shader altitude `length(world - centre) - radius` errs by ±0.5 m. The sea waves are 0.05 m tall.
- Terrain has no structure between 680 km continents and 650 m hills.
  - Mountains top out near 126 m (`globe/datum.rs`, `relief.rs`).
  - Rivers are at most 5 km long (`atlas/trace.rs`).
  - Spawn searches cover under 10 km, so a start point is often in open sea.

## Owner decisions (2026-11-19)
- Real size: rocky planets 1,500 to 12,000 km in radius, gas giants 25,000 to 70,000 km, moons at real sizes.
- Real rules for gravity and days. Surface gravity is real (Earth 9.81 m/s²), and the robot's movement is retuned to it. No day length is set anywhere. Each body has a physical spin, and its day (noon to noon) is computed from that spin and its orbit, as `src/render/clock.rs` already does.
- Kodya is Earth-like: radius 6,371 km, gravity 9.81 m/s², one turn every 86,164 s (23 h 56 min 4 s). Its day of about 24 h follows from that spin and its orbit.
- Other planets get physical spins: at least about twice the break-up period `2π·√(R/g)` (84 min for an Earth), log-spread up to days, and tidally locked when tides from their star or planet are strong enough.
- Bundled into 0.11.0 with the ecosystem update.
- A new universe. Every world records the game version that made it. Older worlds open in their own version, which the game downloads.
- Realistic moons. Kodya, the home planet, gets a moon.

## Assumptions
- With real gravity, `mu = g * R^2` gives Earth's real values on Kodya: orbital speed 7.9 km/s, escape speed 11.2 km/s.
- Today's local generator (islands, villages, scatter, everything finer than about 1 km) stays as the finest layer. Continents and regions are new layers above it.
- The anchor rebase (`src/space/rebase.rs`) and f64 lattices (`src/world/mesh/lattice.rs`) already keep mesh vertices precise. The precision work covers chart coordinates and shaders only.
- The work starts after 0.10.1 lands, because it touches `crates/net` and `src/saves`, which the other session is changing now.

## Approach
Keep the engine: cube-sphere quadtree, chunked levels of detail, moving origin. Outerra, Elite Dangerous and Proland use the same shape. Grow it in five places:

1. **Units.** A real-scale universe: stars at real radii, orbits in AU, Hill spheres, moons by how they form, spins in hours.
2. **Precision.** The terrain chart gets an f64 origin that moves with the anchor, so chart values stay small. Shaders get altitude from a stable form computed relative to the camera.
3. **Terrain on several scales,** each a pure function of direction:
   - Plates: a Voronoi split of the sphere into 8 to 20 plates, after Cortial et al., "Procedural Tectonic Planets", and Frozen Fractal's simplified plates.
   - Crust, continental or oceanic, by the planet's ocean share.
   - Boundary uplift: mountain ranges where plates converge, ridges and rifts where they part, island arcs.
   - Shelves and coasts with a domain warp.
   - A regional noise band of about 100 km.
   - Today's generator for detail under 1 km.
   - Planet-scale drainage for real rivers: a coarse priority-flood over a cube grid, computed once per planet off the main thread.
4. **Physical air:** exponential density with scale height `H = R_gas * T / (M * g)`, about 8.5 km on Earth, and a top near 12 H. Clouds from 1 to 12 km. Drag and the sky shader read the same density.
5. **Scale-aware ranges** for the far view, fog, weather, quadtree caps, the ship and spawn searches.

There is one clock. The physical spin and the orbit drive the surface frame, the sun's direction, day and night, the climate and the ship alike. Nothing in the code sets a day length.

Bevy 0.20's built-in atmosphere (`bevy_pbr::atmosphere`, Hillaire LUTs, built for Earth-scale radii) is the alternative to extending our own sky shader. Our shader stays the default: it already draws every planet from orbit and is lit by the real star.

## Steps

### Phase A: units and moons (pure, `src/universe/`)
1. `src/universe/{planet,home,giant,hill,moon,star,system,zone,orbit}.rs`: real radii, masses and orbits.
   - Rocky planets 1.5e6 to 1.2e7 m (log-uniform, weighted toward 3e6 to 8e6).
   - Giants 2.5e7 to 7e7 m.
   - Stars at real radii (the home star 6.96e8 m) with masses from class.
   - Orbits 0.3 to 40 AU, with the habitable zone from luminosity.
   - Spin periods log-spread from twice the break-up period up to a few days. Tidally locked when the tidal locking time from the star or the host planet is shorter than the system's age.
   - Kodya: radius 6.371e6 m, gravity 9.81 m/s², spin 86,164 s.
   - The real gravity scale replaces `HOME_GRAVITY = 22`.

   Verify: `cargo test --features dev universe::` passes after the tests are updated to the new ranges. A new test asserts no orbit lies inside its star or overlaps a neighbour's Hill sphere, and that the equator speed stays under 0.5 × orbital speed.
2. `src/universe/moon.rs`, `hill.rs`: moons by how they form.
   - Rocky planets: about 25 percent get one large impact moon (0.1 to 0.3 of the planet's radius, at 20 to 80 radii), about 15 percent get 1 or 2 small captured moons (10 to 50 km). The rest have none.
   - Giants: 2 to 6 regular moons (1,000 to 2,600 km) on near-circular orbits from 5 to 30 radii, plus small irregular ones.
   - Every moon inside the Hill sphere and outside the Roche limit, tidally locked.
   - Kodya gets one impact moon.

   Verify: a test over 5,000 systems checks the shares within ±5 percent, Kodya has exactly one moon, and every moon has `ROCHE < a < 0.5 * hill`.
3. `src/render/clock.rs`: the solar day comes from the physical spin and the orbit, with no set length. `HOME_DAY_SECONDS` and `PLANET_DAY` are removed. Verify: a test asserts Kodya's noon-to-noon period is 86,400 s within 60 s, and a tidally locked moon's day equals its orbit around its planet.
3b. `src/player/`, `src/avatar/tune.rs`, `src/gear/`, `src/view/`, `src/ship/`: retune (after 0.10.1, read from the merged tree, not ac1720c: step-up in `climb.rs` with 0.29 m risers, and the new glider model with angle mode, about a 55° bank limit, auto-level, Q/E yaw, stall protection and re-spread. The owner asked for this glider model explicitly: keep the model and retune only its constants) the robot's walking, jumps, wingsuit and ship thrust for 9.81 m/s², so that jump heights and how they feel stay as they are today. Verify: the existing player, avatar, gear and ship tests pass with their expectations rescaled, and a new test checks that a standing jump on Kodya rises within 5 percent of today's height.

### Phase B: precision
4. `src/world/globe/chart.rs`, `src/world/terrain.rs`, `src/world/terrain/view.rs`, `src/world/chunks/face.rs`: the shared terrain chart gets an f64 origin that moves with each anchor rebase. `locate` returns f32 offsets from that origin. Scatter tiles and structure memos are keyed by integer tile ids, not by f32 positions. Verify: a test on an Earth-size globe samples `Terrain::height` at 1 cm steps 3,000 km from the face centre and asserts the result is smooth (no 0.25 m steps). The existing scatter and village tests still pass.
5. `src/world/sea/water.wesl`, `src/render/daylight.wesl`, `src/world/sea/follow.rs`: altitude in shaders from camera-relative values, in the stable form `(|p|^2 + 2(R + a) p·u + a^2 + 2 R a) / (|q| + R)`, where `a` is the camera's altitude, computed in f64 on the CPU. Verify: a naga validation of the shaders (the wiki's throwaway-crate method), plus a CPU test that the same f32 formula errs under 1 mm at 6.4e6 m.
6. `crates/net/src/pack/position.rs`, `crates/net/src/wire/bytes.rs`: positions are sent relative to the sender's anchor cell (an i64 cell plus i32 mm). Protocol version 3. Verify: `cargo test -p abstractions-net` with a round trip at 1.2e7 m kept within 1 mm.

### Phase C: terrain at planet scale
7. `src/world/globe/plates.rs` (new) and children: Voronoi plates on the sphere from Fibonacci seeds with a warp, crust type, motion vectors, and boundary types with uplift profiles. A pure function of direction, under 2 µs per sample. Verify: tests that the same seed gives the same plates, that the land share matches `record.ocean` within ±3 percent over 20,000 samples, that mountains sit within 300 km of convergent boundaries, and a cost test.
8. `src/world/islands/`, `src/world/globe/{relief,datum,elevation,landform}.rs`: height becomes the sum of the plate base, shelf and coast warp, the regional band, and today's local detail.
   - Mountains reach 3 to 9 km, scaled by the inverse of gravity.
   - Ocean depth 3 to 6 km.
   - Datum re-measured by sampling.

   Verify: tests on Kodya at 6,000 km and on a dry, a wet and an ice planet check land share, peak height and that the local look tests pass. The `climate` and `scatter` cost tests still hold.
9. `src/world/globe/drainage.rs` (new), `atlas/trace.rs`: a 6 × 256 × 256 drainage grid, about 40 km cells on Earth.
   - It runs a priority-flood and flow accumulation once per planet on `AsyncComputeTaskPool`, cached in the globe.
   - Local rivers follow its trunks, so rivers run hundreds of km to the sea.

   Verify: a test checks every river path reaches the sea or a closed basin. A timing test keeps the build under 400 ms off the main thread.
10. `src/world/terrain/spawn.rs`, `src/home/{tune,works,found}.rs`: the spawn and home base search starts from the drainage grid's land cells near temperate latitudes, then refines locally. Verify: a test over 50 seeds finds a land spawn every time within 200 ms, and Kodya's base lands on a coast.

### Phase D: air, sky and weather
11. `src/universe/planet.rs` `air()`, `src/ship/drag.rs`, `src/render/atmosphere.rs`: exponential density with the real scale height, a top at 12 H, and cloud base and top from temperature and pressure. Verify: tests that Kodya has H between 7 and 10 km, that drag at 0 m over drag at 8.5 km is about e, and that the sky colour tests still pass.
12. `src/render/mod.rs` (cloud shell), `src/render/clouds/`, `src/weather/grid.rs`, `src/weather/field.rs`:
    - The cloud shell becomes a cube-sphere mesh following the quadtree's coarse levels, with no 13 km sag.
    - The weather grid goes from 64 to 256 per face for cover; the local puffs stay as they are.

    Verify: a test that the shell's error from the true sphere stays under 50 m at every level used, and the cloud tests pass.
13. `src/render/fog.rs`, `src/world/chunks/sight.rs`, `src/world/globe/far.rs`, `src/render/bodies.rs`, `src/render/dome.rs`, `src/world/chunks/dither.rs`: far distances in planet radii (hand-over 1.5 radii), the dome and body placement on reversed-Z depth with no fixed `FAR`, and a dither span relative to the camera's altitude. Verify: `examples/smoke.rs` stops at 2 m, 10 km, 100 km, 1,000 km and 20,000 km over Kodya; the screenshots show the ground, then the far ball, with no gap and no pop.
14. `src/world/globe/quadtree.rs`, `src/world/chunks/reach.rs`: node caps and splits per frame scaled by depth (17 levels on Earth), within today's frame budgets (`chunks/budget.rs`). Verify: `tests_reach.rs` and `tests_fly.rs` pass at Earth size, with settling under 300 s.

14b. `src/universe/density.rs`, `src/render/sky.wesl` (`galaxy_band`), `src/render/sky.rs`, `src/render/starfield/specks.rs` (`galaxy_speck`), plus `src/render/galaxy_light.rs` (new): one light model for every galaxy, so the sky shows the real universe (owner rule: nothing on screen is decoration).
    - Add a dust density along the arms to `density.rs`.
    - Off the main thread, bake a sky cube map (6 x 128 x 128) of starlight integrated along each line of sight through the real density of the galaxy you are in, with the dust absorbing light. Every other galaxy within sight goes into the same map, integrated through its own density.
    - The map is rebaked when the camera moves more than 2 percent of the distance to the nearest galaxy centre, or 50 GLY inside a galaxy.
    - It replaces the noise band and the round galaxy specks. Stars close enough to draw one by one stay points; their light is left out of the integral so nothing counts twice.

    Verify:
    - A test that the baked band peaks within 2° of the real disc plane and is brightest toward the real core.
    - A test that a spiral seen from outside has the aspect ratio its tilt gives, within 10 percent.
    - A test that the brightness of a galaxy changes continuously along a warp from outside to inside it (no jump over 5 percent per step).
    - A timing test that the bake stays under 300 ms off the main thread.
    - Smoke screenshots from inside the home galaxy, from its edge, and from outside it.

### Phase E: ship and travel
15. `src/ship/{cruise,wells,warp,place,route}.rs`: drive and cruise caps from the local orbital speed, warp lock outside about 1.2 radii, carry ranges in Hill-sphere shares, and an autopilot that flies to orbit in about 2 minutes on Kodya. Verify: `cargo test --features dev ship::` with the voyage tests updated; a new test reaches a 200 km orbit around Kodya in under 180 s of simulated time.

### Phase F: saves and versions
16. `src/saves/model.rs`, `src/saves/`, `src/menus/worlds/`: each world stores `game_version`. `SAVE_VERSION` goes to 3. Worlds from an older generator are listed as "Made with 0.10". Verify: a test that a version 2 file loads as an old world and is never opened by the new generator.
17. `src/update/` (`download.rs`, `checksum.rs`, `place.rs`) plus a new `src/update/older.rs`: "Open with 0.10" downloads that release's build for this platform into the game's data directory, checks its SHA-256, and launches it with `--world <id>`. Releases from 0.11.0 on accept `--world` (older releases open at their title screen with the world listed). Verify: a test of the whole path with the fake host (`src/update/fake.rs`), plus a real run on Linux by the merger.

### Phase F2: the map at planet scale (from the 0.10.1 notes)
17b. `src/map/explored/grid.rs` and `src/map/` (wire `EXPLORED_COLUMNS/ROWS`, the save field `explored`): the explored grid becomes a sparse quadtree per body on the cube-sphere, with leaves of about 500 m. Today it is a fixed 512 x 256 bitset, about 78 km per cell at Earth size, while the reveal radius is 2 km. It is stored and sent as sparse leaf runs, in protocol 3 and `SAVE_VERSION` 3. Verify: a test that walking 2 km reveals the cells under the path and no others, a round trip through the wire and the save, and a 10,000 km walk staying under 64 KB.
17c. `src/map/raster`: the planet picture paints from the drainage and plate layers at whatever zoom is shown, tiled and cached. Today it is one fixed image, which at Earth size is 78 km per pixel. Verify: a test that a zoomed tile 100 km across shows coastlines within 1 pixel of `Globe::height` crossings.
17d. `src/menus/hud/minimap`: check the range caps that use a share of the radius (`REACH = 0.35` R) and keep the metre-scale ranges. Verify: the minimap tests pass at Kodya's new size.

### Phase G: ship 0.11.0
18. Smoke screenshots at every altitude, `mise run check`, then CHANGELOG, ROADMAP and README, and the release. Verify: the `release-preflight` flow is green and the release has every asset.

## Parallel plan (after 0.10.1 merges into `ecosystems`)
- Wave P1, in parallel: `units` (steps 1 to 3), `precision` (steps 4 and 5), `plates` (steps 7 and 8), `air` (step 11). Disjoint files.
- Wave P2, in parallel: `galaxy` (step 14b), `net` (step 6), `drainage` (steps 9 and 10), `sky` (steps 12 to 14), `ship` (step 15), `saves` (steps 16 and 17), `map` (steps 17b to 17d; shares the protocol 3 and `SAVE_VERSION` 3 formats with `net` and `saves`, so those three agree a format note first), `tune` (step 3b).
- The ecosystem waves keep running. Herd cells (256 m) and census sampling do not depend on radius. The runtime agent reads `space::up` and `Terrain::mesh_height`, never chart values, so step 4 does not touch it.

## Boundaries
- Ask first:
  - Protocol 3.
  - `SAVE_VERSION` 3.
  - Any new crate (none planned; `big_space` is not needed because the anchor rebase covers it).
- Never:
  - Raise a frame budget in `chunks/budget.rs`.
  - Store f32 chart positions in saves or on the wire.
  - Keep old generator code in the new binary.
  - Edit `crates/net` or `src/saves` before 0.10.1 is merged.

## Done when
- A new world starts on Kodya, 6,000 km in radius, with a moon in its sky.
- You can see continents from orbit, mountain ranges along plate edges, and rivers that reach the sea.
- The robot can walk, fly up through a real atmosphere, reach orbit in about two minutes, and warp to a moon.
- Frame time stays within today's budgets.
- An old 0.10 world opens in 0.10 from the world list.
- `mise run check` is green.

## Risks
- Scope: this roughly doubles the release. The ecosystem and this work share one branch, so a slip in one delays both.
- The look of today's islands at the fine scale must survive the new layers. Phase C keeps the local look tests as the guard.
- Real days on Kodya last about 24 h of play, so night lasts about 12 h of real time. Night needs to stay playable: moonlight from Kodya's moon, starlight, glowing life and the robot's light.
- Streaming at 17 levels: the first load can take longer. The loading-screen budgets (14 ms per frame) cover it.
- Multiplayer: protocol 3 means 0.10.x clients cannot join 0.11 servers. The server image and the Dokploy compose need the new tag at release.

## Performance guard (owner rule: the game must run fast)
- Every new feature lands with a cost test or a measured frame budget, and the merger rejects a branch that raises a budget in `chunks/budget.rs` or the 2 ms fauna budget.
- The merger runs the smoke scene with frame-time numbers at each wave merge and reports them to the owner. When a feature does not fit, the owner hears it before it ships, with the option to cut or defer it.

## Skip
- Caves and overhangs: the ground stays a height field, as in Elite Dangerous.
- Real interstellar distances: the warp stays as it is.
- Erosion simulation: drainage and shape functions only.

## Sources
- Steven Wittens, "Making Worlds 1: Of Spheres and Cubes": https://acko.net/blog/making-worlds-1-of-spheres-and-cubes/
- "Making Worlds 2: Scaling Heights": https://acko.net/blog/making-worlds-2-scaling-heights/
- "Making Worlds 4: The Devil's in the Details": https://acko.net/blog/making-worlds-4-the-devils-in-the-details/
- Ocean rendering on a planetary scale (quadtree over CDLOD and clipmaps): https://playerunknownproductions.net/news/ocean-rendering-on-a-planetary-scale
- Planetary terrain (Rust): https://ralith.com/blog/planetary-terrain/
- Cortial et al., "Procedural Tectonic Planets": https://diglib.eg.org/items/d7b74043-dcb2-4b18-b128-5ec43b73548d/full
- Frozen Fractal, "Around The World, Part 1: Continents": https://frozenfractal.com/blog/2023/11/2/around-the-world-1-continents/
- Frozen Fractal, "Part 2: Plate tectonics": https://frozenfractal.com/blog/2023/11/13/around-the-world-2-plate-tectonics/
- Frozen Fractal, "Part 28: Scaling up": https://frozenfractal.com/blog/2025/12/12/around-the-world-28-scaling-up/
- Red Blob Games, planet generation on a sphere: https://redblobgames.com/x/1843-planet-generation
- Bruneton and Neyret, "Precomputed Atmospheric Scattering": https://hal.archives-ouvertes.fr/inria-00288758
- Schneegans et al. 2024, atmospheres of other planets with Mie theory: https://diglib.eg.org/items/1fb6b85a-b3f8-4817-975f-f65634020f03
- Elite Dangerous planets, full size, surface from Stellar Forge values: https://www.pcgamer.com/the-mind-bending-science-behind-the-planets-of-elite-dangerous
- Elite Dangerous Odyssey planet rebuild: https://www.pcgamer.com/heres-how-frontier-rebuilt-a-galaxys-worth-of-planets-for-elite-dangerous-odyssey
- Star Citizen planet tech v4 (temperature and humidity driven): https://twinfinite.net/news/star-citizen-microtech-planet-tech-v4/
- KSP 1/10 scale and travel time: https://forum.kerbalspaceprogram.com/topic/51895-why-are-ksp-planets-undersized-in-comparision-to-our-solar-system
- big_space floating origin for Bevy (considered, not needed): https://docs.rs/crate/big_space/latest/source/src/lib.rs
- Dead end: no primary source found on Outerra's engine internals (https://gamedev.net/forums/topic/643870-what-terrain-rendering-technique-does-the-outerra-engine-use/5066992/); the Star Citizen v4 details come from press coverage only.
