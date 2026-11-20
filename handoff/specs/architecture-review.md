---
title: "Abstractions architecture review, 2026-11-20"
machine: "ruche"
created: 2026-11-20T18:39:11Z
updated: 2026-11-20T18:39:11Z
type: artifact
---

# Abstractions architecture review, 2026-11-20

Read-only review of `saravenpi/Abstractions` at commit `e8f0dd4` (branch `main`). Three explorations read the code. Nobody ran cargo and nobody measured anything: every cost below comes from reading call chains. Measure before and after each performance change.

"Tested" means a test file mentions the item, not that the behaviour is asserted.

Goals set by the owner: less redundancy, one way of doing each thing, a smaller codebase, a faster game.

## Summary of candidates

| # | Candidate | Gain | Strength |
| --- | --- | --- | --- |
| A1 | Cache the universe: one resource for the current system and planet | Performance, deletes 4 to 10 private caches | Strong |
| A2 | One frame and velocity module for ship and net | Deletes a copy, one test surface | Strong |
| A3 | One atmosphere model (density with height, "has air") | Fixes drift between gear and ship | Strong |
| A4 | One gravity function | Fixes player and ship disagreeing below sea level | Worth exploring |
| A5 | Retire the phase-based `daylight`, read `SunDir` | Removes the fake sun model | Worth exploring |
| A6 | One latitude and longitude module in the base layer | Deletes 3 copies | Strong |
| A7 | A base `math` module: smoothstep, angle wrap, spring, hash | Deletes about 30 copies | Strong |
| A8 | Move `Screen`, `playing`, `Link`, `spin`, `SYSTEM_REACH` down | Clears most upward imports | Strong |
| B1 | One per-frame "ground under the focus" resource, throttle the shore search | Performance | Strong |
| B2 | One deep `Ground` reader in `world` | Deletes 4 readers and 10 copies of the floor rule | Strong |
| B3 | Structures use one height source | Ends the floating-props class of bug | Worth exploring |
| B4 | One mesh builder shared across areas | Deletes 3 to 4 builders | Strong |
| B5 | One `Trail<const N>` for gear and companion | About 250 lines | Strong |
| B6 | One face module for avatar and companion | About 450 lines | Strong |
| B7 | One footfall source for audio and avatar | Sound and dust coincide | Worth exploring |
| C1 | One list-screen module (worlds, multiplayer, computer board) | Deletes 2 of 3 copies, key hints from one table | Strong |
| C2 | One focus stepper, one text field with caret | Deletes 5 handlers, 3 carets | Strong |
| C3 | Screen scaffold, progress bar, show/hide helper in `crate::ui` | Deletes 8 + 3 + 16 copies | Strong |
| C4 | One formatting home: lat/lon, durations, units, truncation | Harmonises "deg" vs "°", "65 %" vs "40%" | Strong |
| C5 | One replicated-kind seam in net | A 4th kind touches 3 files, not 12 | Worth exploring |
| C6 | Settings declared once, split by concern | Performance, one option in fewer files | Worth exploring |
| C7 | Stop the 0.5 s computer chart rebuild, gate the ungated `ui` systems | Performance | Strong |

## Part A: celestial and navigation

### A1. Every universe lookup regenerates, and each area keeps its own cache

`Universe` holds only `{ seed }` (`src/universe/mod.rs:84`).

- `planet(id)` rebuilds the whole system: the sector of stars, every planet record, the moon orbit slots (`mod.rs:138-143`).
- `star(id)` rebuilds the whole sector, names included (`sector.rs:50-53`).
- `planet_position` looks like a getter but is `self.planet(id).position(time)` (`universe/orbit.rs:9-11`).

Caches of "a system plus its moons", built three different ways:

- `src/render/nearby.rs:25` `Nearby::read`: moons in a separate Vec, no `moons > 0` filter.
- `src/ship/sky.rs:55` `Skies::system` with `src/ship/moons.rs:20` `with_moons`: moons after planets, filtered.
- `src/net/state/bodies.rs:48` `Bodies::system`: HashMap of planets chained with moons, filtered.
- `src/menus/hud/coords.rs:103`: its own `(StarId, SystemSpec)`, no moons.

Caches of "the planet under me" (`world::Planet` at `world/mod.rs:75` holds no spec):

- `wind.rs:116`, `audio/sense.rs:57`, `weather/watch.rs:58`, `world/calendar.rs:48` key on `(Universe, BodyId)`.
- `ship/sky.rs:73` (`Land`) and `menus/hud/climate.rs:82` key on the body id only. `calendar.rs:13` says why that is wrong. The HUD one is a `Local` that is never reset: possible stale-spec bug.

Uncached callers on hot paths:

- Every frame, ship report: `ship/route.rs:166` calls `ship_position` before checking for a goal.
- Every frame with autopilot: `route.rs:115`, `ship/pilot.rs:59-62`.
- Every frame, map chart: `map/pins.rs:141` to `map/spots.rs:44-45`, once per marker, up to 32.
- Every frame, HUD compass: `menus/hud/compass/aim.rs:97,105-106`.
- Computer table twice a second (`computer/show.rs:28`): per row `rows.rs:95`, `rows.rs:67`, `rows.rs:104`.
- Minimap on its timer: `menus/hud/minimap/orbit.rs:67,72`.
- `ship/sky.rs:130-140` `refresh` builds a new `wells` Vec every frame (`wells.rs:58-62`).

Tests: `Nearby::read` in 8 test files, `with_moons` in 1. Nothing checks the three caches list bodies the same way.

```text
Before
render::Nearby   ship::Skies   net::Bodies   hud::coords   wind  audio  weather  calendar
      \              |              |             |          \     |      |       /
       +---------- Universe { seed }: regenerates sector + system on every call -------+

After
Universe { seed }  (pure, unchanged)
      |
Here (resource, base layer): current system, moons, current planet spec,
      keyed on (Universe, StarId, BodyId), rebuilt only when the key changes
      |
render  ship  net  hud  wind  audio  weather  calendar  map  computer   (all read Here)
```

### A2. Frame and velocity maths written twice (ship and net)

- `ship/frames.rs:18` `planet_velocity` and `net/state/frame.rs:93` `velocity` are identical: `position(time + 0.5) - position(time - 0.5)`.
- Spin rate `TAU / day_seconds` at `ship/frames.rs:14` and inline at `net/state/frame.rs:117`.
- `ship/frames.rs:24-44` (`to_system`, `to_surface`) and `Rigid::out`, `Rigid::back` (`net/state/frame.rs:123-140`) do the same pose transform.
- Both rest on `world::focus::spin` and `Link` (`world/focus.rs:17-28`), world-layer types that hold universe-level ideas.
- Orbit angle `phase + TAU * time / seconds` copied in `climate/season.rs:11` and `universe/spec.rs:108`.

Tests: `ship/tests/space.rs`, `ship/tests/crossing.rs`, `net/state/tests/frames.rs` each test their own copy.

### A3. Air thickness with height: four curves that drifted

- `ship/drag.rs:34` `density`: `1.2 * thickness * exp(-h / (0.125 H))`, with a deep layer under a gas giant's cloud deck.
- `gear/input.rs:84` `air_density`: same exponential in f32, no `thickness`, no giant layer. Its doc (`:82`) claims parity, which is no longer true.
- Copied constants: `SEA_DENSITY` (`gear/tune.rs:6`, `ship/drag.rs:10`), `SCALE_SHARE` (`gear/input.rs:8`, `ship/drag.rs:13`).
- Callers recompute the air height with `air(globe.radius())[0]` (`player/flight.rs:41`, `gear/fx/wakes.rs:121`) instead of reading `spec.atmosphere_height`.
- `render/atmosphere.rs:24` `air`: `1 - (h/H)^2`. `audio/scene.rs:103` `altitude_fade`: `(1 - h/H)^2`.

"Does this planet have air" has several rules: `wind/air.rs:17` (hard cut at `NO_AIR`, capped at 3), `render/air.rs:63` (quadratic ramp), plain `pressure < NO_AIR` in `effects/planet.rs:10`, `climate/bands.rs:45`, `render/season.rs:67`.

Tests: `gear/tests/vacuum.rs`, `ship/tests/giant.rs`. No parity test.

### A4. Gravity: four inverse-square formulas

- `space/gravity.rs:17`: `g * (R/d)^2`, stops increasing below sea level, f32. The player uses it.
- `ship/atmo.rs:26`: `pos * mu / r^3` plus centrifugal, not clamped below sea level.
- `ship/deck.rs:75`: `mu / r^2`. `ship/wells.rs:68-72`: softened at `r/2`.

They agree only because `mu = g R^2` (`universe/spec.rs:171`). Below sea level the player and the ship feel different gravity. `space/gravity.rs:22` holds an empty `plugin(_app)` registered at `space/mod.rs:65`.

### A5. Two models of the sun

- `render/sun.rs:18` `SunDir`: real geometry, shadow and eclipse (`observer.rs:97-108`, `sun.rs:150-151`).
- `render/celestial.rs:17` `sun_direction(phase)`: older fake circle tilted by `AXIS_TILT = 0.5`. `celestial::daylight(phase)` (`:27`) and `Palette::at` (`palette.rs:128`) use it.
- `celestial::daylight(phase)` feeds `companion/sense.rs:140`, `audio/sense.rs:69`, `home/lights.rs:57`, `world/structures/lights_sys.rs:91`, `effects/sim.rs:112`, `effects/creatures/env.rs:69`. These ignore latitude, eclipses and being in space.
- Two functions named `daylight`: `render/celestial.rs:27` and `render/daylight.rs:57`.

This one changes behaviour the player can see (lights, creatures, sounds follow the real sun). Rewrite the tests to assert the new behaviour and say so.

### A6. Latitude and longitude conversions written several times

- `map/geo.rs:18` `lat_lon`: (lat, lon), lon wrapped to 0..2pi.
- `map/explored/grid.rs:29` `lon_lat`: same maths, tuple order swapped, no test.
- `menus/hud/coords.rs:52` `latlon`: degrees, lon not wrapped.
- `render/season.rs:66`: inline `asin`.
- The inverse `direction` sits in `computer/season.rs:34` (a card-text module); `map/geo.rs:13` imports it; `map/explored/grid.rs:51` copies it inline.

### A7. Small helpers copied around (merged with B7 helpers)

Smoothstep, body `t * t * (3.0 - 2.0 * t)`:

- Shared one: `render/celestial.rs:11`, used in about 45 files, but it lives in the drawing layer so base areas cannot import it.
- Copies: `universe/density.rs:72` (f64), `climate/bands.rs:32`, `world/sea/uniform.rs:162`, `world/sea/blend.rs:24`, `world/islands.rs:37`, `companion/spring.rs:8`, `avatar/clock.rs:49`, `avatar/wing.rs:43`, `effects/creatures/anim.rs:6`, `render/warp/shader.rs:85`, `world/mesh/bands.rs:41`, inline in `weather/noise.rs:52,68`.

Angle wrap `(angle + PI).rem_euclid(TAU) - PI`: `player/steer.rs:25` (imported 7 times by avatar), copied in `view/smooth.rs:13`, `view/roll.rs:44`, `gear/aim.rs:52`, `companion/neck.rs:35`, `effects/creatures/steer.rs:9`.

Critically damped spring, written 5 times: `avatar/clock.rs:57` (f32), `companion/spring.rs:18` (Vec3), `gear/servo.rs:19` (Vec2), `view/smooth.rs:44` (`Spring` struct), inline in `player/steer.rs:117-125` with a rate clamp.

Hashes and generators:

- `universe/hash.rs:6` `mix` is the splitmix finalizer only; `climate/heat.rs:46` `mix` adds the golden-ratio constant first (drifted). Changing either changes generated worlds: do not unify outputs, only share code where outputs stay identical.
- `world/globe/roll.rs:4,16` and `world/terrain/ponds.rs:30`: same cell hash.
- `avatar/puffs.rs:97` and `gear/fx/streaks.rs:31` `noise`: byte-identical. `effects/rng.rs` uses the same mixer.
- `companion/rng.rs` is `universe::hash::Rng` (SplitMix64) with f32 output.
- `weather/noise.rs:7` imports `audio::Rand` (upward) although `universe::hash::Rng` exists below.

Other:

- `class == PlanetClass::GasGiant` at 11 sites; only `ship/deck.rs:29` wraps it.
- Angular size: `render/moons.rs:110` guards with `.max(radius)`, `render/bodies.rs:102` with `.max(1e-9)`.
- Ring of points on a circle: `home/ground.rs:15`, `home/site.rs:29`, `world/structures/rules.rs:124`, `village/place.rs:43`, `village/streets.rs:34`, `scatter/pass.rs:56`, `terrain/spawn.rs:53,130`, `audio/shore.rs:24`.
- Upright tilt: the f64 form explained in `player/upright.rs:14` is copied correctly in `gear/fx/scene.rs:151`, `gear/fx/wakes.rs:136`, `effects/eye.rs:45`, `net/state/send/kinds.rs:78`, but `avatar/kick.rs:136` and `world/structures/smoke.rs:94` still use the f32 `Quat::from_rotation_arc(Vec3::Y, ..)`.
- Distance from body centre to soles: derived in `player/climb.rs:26` `SOLES` and `world/structures/touch.rs:17` `HALF_BODY`, hard-coded 0.9 in `gear/fx/scene.rs:21` and `avatar/tune.rs:6`, inline 8 more times (`player/*`, `audio/steps.rs:96`).
- Per-chunk RNG seeding: `world/structures/site.rs:116` `cell_rng` and `world/scatter.rs:130` `stream` seed the same way, but only scatter mixes in the cube face. Fixing this changes generated worlds: ask the owner first.

### A8. Upward imports and shallow modules

Non-test upward imports by pair:

| From to | Count | Types |
| --- | --- | --- |
| render to world | 12 | `Link`, `Globe`, `Planet`, `globe::far::{handover, FAR, ball}`, `spin`, `SEA_LEVEL` |
| render to screens | 10 | `Screen`, `WarmUp` |
| render to computer | 2 | `SYSTEM_REACH` |
| map to menus | 9 | `Marker*` |
| map to screens | 5 | `Screen`, `in_control`, `playing` |
| map to net | 4 | `Link`, `FromServer`, `Receive` |
| space to world | 5 to 9 | `Globe`, `Terrain`, `Chart`, `SEA_LEVEL` |
| space to player | 1 | `Player` in `rebase.rs` |
| space to menus, net, screens | 3 | all in `clock.rs` |
| sim to computer | 2 | `Atlas` in `log.rs` |
| sim to net, screens | 2 | `Session`, `Screen` |
| weather to menus | 3 | `HudWeather` |
| climate to world | 1 | `Biome` |
| computer to menus, screens, ship | 2 each | `Menu`, `Markers`, `Screen`, `Ship`, `ShipTelemetry`, `ShipSet` |
| effects, gear, world to screens | 6 each | `Screen`, `playing` |
| player to screens | 5 | |
| audio to screens | 4 | |
| avatar to ship | 5 | `ship::SEAT`, `cockpit::Levers` |
| companion to ship | 4 | |
| saves to world | 7 | |
| settings to view | 3 | |
| world to player | 1 | `world/structures/touch.rs:11`, body constants |

- 35 of the `screens` imports are `Screen` or `playing`. Moving those two into the base layer clears most violations.
- `world` and `home` import each other: world to home 3 (`world/mod.rs:42`, `world/reserve.rs:9`, `world/calendar.rs:65`), home to world 9.
- `SYSTEM_REACH` is a universe concept defined in `computer/locate.rs:10`.

Shallow modules: `universe/orbit.rs` (12 lines, hides the cost of A1), `climate/season.rs` (13 lines), `space/gravity.rs` (one function plus an empty plugin), `universe/ids.rs` (5 lines).

### Part A, checked and clean

- Same-named files (`moons.rs`, `giant.rs`, `look.rs`, `clock.rs`) are different concerns per layer.
- Unit formatting has one source: `computer/format.rs` `distance` and `gravity`, plus `saves::play_time`.
- The season formula has one source, `climate::season`.
- Time: `space::Clock` is the single universe time; `render::DayClock` derives from it.
- Orbit positions all go through `PlanetSpec::position` or `Orbit::at`.

## Part B: world and actors

### B1. The terrain is asked about the same point many times per frame

`Terrain::surface` goes through `Globe::cover` (`world/globe/ground.rs:47`), which calls `ground()` five times per lookup.

- `audio/sense.rs:76-78`: `shore()` passes `terrain.height` to `audio/shore.rs:21` `nearest`, which reads 8 rings x 12 spokes + 1 = 97 heights and only stops early on a shoreline. Runs every frame in Gameplay (`audio/sense.rs:179-186`), no throttle.
- `audio/steps.rs:69-71` `footing`: one `height`, then a surface (5 samples) computed even when the feet are not touching.
- `avatar/kick.rs:96-99` (`soil`, called at `:131`): `terrain.surface` once per avatar per frame, plus `recolour` every frame.
- `player/spawn.rs:165` `rescue`: one `height` per player per frame.
- `companion/spot.rs:66-71` `gap`: two calls from `companion/wing.rs:88`, one from `companion/motion.rs:98`.
- `world/chunks/near.rs:78` `Height::of` (`world/chunks/sight.rs:36`): `Around::along` rebuilt twice per frame by `stream` and `collect` (`world/chunks/stream.rs:92,121`). `collect` also allocates `finished()` (`world/chunks.rs:158`) and a HashMap every frame.
- For contrast, `effects/sense.rs:64-77` already throttles itself to 0.5 s.

Systems without a run condition: `world/sea.rs:116` `ember::kindle`, `audio/feed.rs:56` `unity`.

### B2. Four separate ground readers, and the floor rule written 10 times

- `effects/ground.rs:15` `Ground` (`chart`, `altitude`, `height`, `sea_level`, `raised`)
- `companion/spot.rs:55-78` `Probe` (`up`, `gap`, `clear`)
- `avatar/kick.rs:80-106` `Ground` SystemParam (`soil`, `air`)
- `audio/steps.rs:63` `footing` and `player/spawn.rs:159` `rescue` inline the same chain

`terrain.locate(anchor.to_frame(pos))` at 6 sites: `companion/spot.rs:67`, `audio/steps.rs:98`, `audio/sense.rs:138`, `avatar/kick.rs:98`, `player/spawn.rs:164`, `effects/ground.rs:35`.

`height(..).max(sea_level())` 10 times: `companion/spot.rs:71`, `effects/creatures/wander.rs:71`, `effects/respawn.rs:70,88`, `world/chunks/sight.rs:38`, `render/murk.rs:51`, `ship/spawn.rs:54`, `ship/board.rs:71`, `map/explored/reveal.rs:66`, `saves/upgrade.rs:43`.

```text
Before
effects::Ground   companion::Probe   avatar::Ground   audio::footing   player::rescue
       \                |                  |                |              /
        each: anchor.to_frame -> terrain.locate -> height/surface -> .max(sea_level)

After
world::Ground (SystemParam): at(pos) -> { height, floor, surface, biome, wet }
       |  caches the answer for the focus point once per frame (B1)
effects  companion  avatar  audio  player  render  ship  map
```

### B3. Structures pick between two height sources per call site

- `Build::ground_at` (`world/structures/brush.rs:150`): analytic `terrain.height`. 11 callers, plus 41 callers of `dress::on_ground` (`world/structures/dress.rs:12`).
- `Build::mesh_y` (`world/structures/hold.rs:45`): `mesh_height`. 5 callers, all in `village/drape.rs` and `village/fields.rs`.
- Other analytic floors: `flat.rs:28`, `village/place.rs:89`, `village/floors.rs:62`, `parts.rs:62`, `parts.rs:72-76`.
- Crates and barrels sink 0.1 m (`dress.rs:20,30`), which hides the gap.

Tests: `tests_streets.rs:22`, `tests_village.rs:163`, `tests_headroom.rs:78` check contact against `mesh_height`. Nothing checks dress props, rocks or `settle`. Scatter is clean.

Switching sources moves props by up to about 1 m on coarse rings, and changes what the player sees. It also touches the frozen-output tests. Handle with care.

### B4. Mesh builders: six near-copies

| Builder | Location | Methods |
| --- | --- | --- |
| `Soup` | `avatar/mesh/soup.rs:9` | `tri`, `quad`, `into_mesh` |
| `Soup` | `companion/body.rs:30` | `tri`, `quad`, `into_mesh` |
| `Soup` | `home/soup.rs:27` | `tri`, `quad`, `beam`, `slab`, `body`, `boxes`, `merge`, `into_mesh` |
| `Solid` | `ship/solid.rs:9` | `tri`, `quad`, `plate`, `slab`, `merge`, `into_mesh` |
| `FlatMesh` | `world/shapes.rs:25` | `tri`, `into_mesh`, free `textured_mesh` |

- Avatar and companion `Soup` are identical line for line.
- `companion/orb.rs:10` `solid` is identical to `world/shapes.rs:49` `textured_mesh`. `avatar/mesh/mod.rs:42` `loose` is the same without tangents.
- `slab` written three times: `home/soup.rs:64`, `ship/solid.rs:41`, `world/solids/slab.rs:9`.
- Colour handling drifted: avatar, companion, home store vertex colours; world stores colormap UVs; ship stores none.

Use counts: avatar 9, ship 21, home 4, world about 8. Tests: only `home/tests_cover.rs`.

### B5. The companion wake is a copy of the gear trail and ribbon

- `companion/ribbon.rs:3-4` says so itself. 143 lines against 173 in `gear/fx/ribbon.rs`. Differences: `FADE_IN` (2 vs 3), `NEAR` ((0.5, 1.5) vs (0.75, 2.5)), a missing `spread`.
- `companion/wake.rs` (113 lines) and `gear/fx/trail.rs` (120 lines): same ring buffer, `POINTS` 24 vs 64.
- One `Trail<const N>` removes about 250 lines.

### B6. The avatar face is a copy of the companion face

`avatar/face/{machine,paint,sprites,mod}.rs` (449 lines) and `companion/face/*` (454 lines).

- `row`, `stamp`, `paint`, `rgba` are the same code (`avatar/face/paint.rs:38-77`, `companion/face/paint.rs:33-84`); only `frame.mien` vs `frame.expr` differs.
- Same state machine API: `shown`, `react`, `change`, `advance`, `lid`, `frame`. `BLINK_EVERY` differs.
- One module generic over the expression enum and the sprite table.

### B7. Footfalls are modelled twice and disagree

- Avatar: gait phase (`avatar/gait.rs:24`, `STRIDE (0.8, 0.24)` in `avatar/tune.rs:79`) drives dust through planted edges (`avatar/kick.rs:46,75`).
- Audio: counts distance (`audio/steps.rs:14-17`, `STRIDE 1.7`, `STRIDE_PER_SPEED 0.11`). Sounds and dust do not coincide.
- Audio derives contact from analytic height (`audio/steps.rs:69-70`, `CONTACT 0.6`) instead of `Motion.grounded` (`player/controller.rs:41`).
- Two surface mappings: `audio/steps.rs:39`, `avatar/dust.rs:75`.
- Avatar kick has no `Surroundings`, so walking on planks kicks up terrain dust.
- `WADE_DEPTH` is 0.5 in `audio/steps.rs:23` and 1.2 in `player/swim.rs:27`.

### Part B, checked and clean

- The 12 `ground.rs` files, and `spawn.rs`, `air.rs`, `steps.rs`, are unrelated content. Only the face `paint.rs` is a real duplicate.
- Up and gravity for actors: everyone uses `space::up`, `space::altitude`, `space::local_gravity`.
- Wind: single `Wind` and `Thickness` resources. `player::above_sea` is shared.
- Mesh building inside `world`: one builder.
- Chunk streaming leaves early on its `settled` key; light selection is throttled.

## Part C: interface, menus and net

`crate::ui` is used well for text, colour and font sizes: zero `Text::new` and zero numeric font sizes outside `src/ui`. The duplication sits one level up: list screens, text fields, input handling and screen layout. Of the 12.8k lines of `menus`, 4.0k are tests and 3.4k the HUD.

### C1. One list-screen pattern written three times

Each copy has a `Row`/`Slot(usize)` marker, an `enum Part`, a `Cell(usize, Part)`, a `cell_text`, a change-gated `show`, an `Act` enum with a `Ctx` and `run`, an `on_activate` observer, a `Mode` and a `Notice` resource.

- `focus_cursor`, 3 copies: `menus/worlds/keys.rs:89`, `menus/multiplayer/keys.rs:45`, `computer/keys.rs:87`.
- `follow_focus`, 2 copies: `worlds/keys.rs:39`, `multiplayer/keys.rs:23`.
- `cell_text`, 3 copies: `worlds/rows.rs:39`, `multiplayer/rows.rs:76`, `computer/show.rs:67`.
- `Cell`, 3 copies: `worlds/rows.rs:35`, `multiplayer/rows.rs:28`, `computer/view.rs:49`.
- `Act` and `Ctx`, 2 copies: `worlds/act.rs:12,32,42,138`, `multiplayer/act.rs:16,37,52,170`.
- Key table and `FAR = 1_000_000` copied: `worlds/keys.rs:14-26`, `computer/keys.rs:27-41`.
- Wheel conversion, 3 copies with 3 scales: `worlds/keys.rs:74` (1 row per line), `computer/keys.rs:76` (3 rows), `ui/scroll.rs:84` (pixels).
- Three cursor models: worlds keeps a `ui::Window`; `computer/board.rs:28-30` stores `cursor` and `top` and copies them through a temporary `Window` (`board.rs:96-113`); multiplayer keeps a bare `cursor` and despawns and respawns its rows (`multiplayer/view.rs:122-133`).
- Key hints are written twice per action: labels in `worlds/bars.rs` and `multiplayer/bars.rs`, real keys in the `ACTS` tables in `keys.rs`. They already differ: Refresh is bound to F5 and R but shows "F5"; Rename is bound to R and F2 but shows "R".

```text
Before
worlds:      Slot, Part, Cell, cell_text, show, Act, Ctx, keys, bars, ui::Window
multiplayer: Row,  Part, Cell, cell_text, show, Act, Ctx, keys, bars, bare cursor + respawn
computer:    Slot, Part, Cell, cell_text, show,            keys,       cursor + top

After
ui::list: Window, focus_cursor, follow_focus, wheel, key table -> hints, pooled rows
worlds / multiplayer / computer: only their columns, their actions and their data
```

### C2. Focus movement and text entry rebuilt per screen

- `menus/slots.rs:13 next_slot` and `:32 move_slots` repeat `ui/focus.rs:8 next_index` and `:19 move_focus`, tests included. `nav.rs:115-119` switches between them.
- Text entry, 5 handlers: `worlds/modal.rs:21`, `multiplayer/form.rs:53`, `new_game/input.rs:33`, `computer/typing.rs:44`, `map/favourite.rs:46`. Menus share `menus/name.rs:12 edit_to`; computer and map write their own. Map reads `Key::Character`, menus read `key.text`.
- Caret, 3 copies: `menus/name.rs:24`, `map/side_show.rs:43`, `computer/typing.rs:118`. Colour differs: `theme::INK` at `new_game/input.rs:24`, `theme::TEXT` at `multiplayer/form_view.rs:53`.
- Name limit in 3 places: `menus/name.rs:4 MAX_CHARS = 24` copies `wire::NAME_MAX = 24` (`crates/net/src/wire/mod.rs:64`); `name.rs:44` repeats "24" in a string.
- Underlined text field, 2 copies: `new_game/view.rs:31`, `multiplayer/form_view.rs:61`. The first rebuilds `ui::frame()` by hand (lines 46-48).

### C3. Screen layout and widget shapes missing from `crate::ui`

- Screen scaffold `root()` + `panel()` + `heading(..)` + `footer(..)`, 8 copies in two styles: `pause.rs:97`, `players.rs:23`, `update/view.rs:58`, `worlds/view.rs:114`, `new_game/view.rs:120`, `multiplayer/view.rs:158`, `multiplayer/joining.rs:48`, `options/view.rs:99`.
- Progress bar, 3 copies: `menus/update/parts.rs:46` (4 px), `menus/hud/panel.rs:52` (2 px), `screens/loading.rs:54` (2 px).
- Show/hide toggle `if node.display != d { node.display = d }` at about 16 sites in 13 files, plus local helpers `worlds/rows.rs:139`, `options/pages.rs:14`, `minimap/marks.rs:17`.
- HUD text line built at `hud/climate.rs:42`, `hud/coords.rs:41`, `computer/readout.rs:37`, `minimap/widget.rs:90`. HUD strip box twice: `hud/prompt.rs:54`, `hud/prompt/course.rs:32`.
- Icon + label row by hand at `main_menu.rs:34` and `update/home.rs:63` where `ui::entry` exists.
- One stray `TextFont` at `menus/hud/compass/mod.rs:78`, because `ui::icon_sized` takes a `Tone` and not a free colour.
- Raw `Node {}` literals: 53 in menus (29 in the HUD), 22 in map, 25 in computer, 10 in screens.
- Design rule conflict, for the owner to decide: AGENTS.md says no borders and no rounded corners, yet `ui/table.rs:20` and `ui/widgets.rs:91 hairline()` are borders, both text fields use one, and `computer/orbits.rs:51-52` sets `BorderRadius::MAX`.

### C4. Formatting done more than once

- Lat/lon, 2 copies printing the same `"{:.1}° N  {:.1}° E"`: `menus/hud/coords.rs:52 latlon` + `:61 ground`, and `map/geo.rs:18 lat_lon` + `:51 coords`. The HUD uses `normalize()` (NaN on a zero vector); the map uses `normalize_or(X)`. See A6.
- "Time ago", 2 copies with different output: `saves/format.rs:4 ago` ("5 min ago") and `menus/worlds/when.rs:18 relative` ("5 minutes ago", "yesterday"). `ago` is dead: only its own test and the re-export at `saves/mod.rs:24` use it.
- Durations live in saves: `saves::play_time` also formats `computer/course.rs:32`, `computer/facts.rs:43`, `computer/words.rs:77`.
- Units disagree: "deg" at `options/format.rs:39` and `computer/words.rs:71`, "°" at `geo.rs:57` and `coords.rs:66`. "65 %" at `hud/units.rs:21` and `computer/words.rs:49-50`, "40%" at `computer/course.rs:58` and `menus/update/format.rs:30`.
- Truncation, 6 ways: `computer/format.rs:35 clip` ends in ".."; bare `chars().take(N)` at `update/format.rs:48`, `options/format.rs:111`, `worlds/ops.rs:71`, `map/favourite.rs:37`, `update/release.rs:70`.

### C5. Network: each replicated thing is coded three times by hand

- Kind-to-index mapping twice: `crates/net/src/room/seat.rs:109 kind_index` with `KINDS = 3` (`:8`), and an inline match at `src/net/state/send/mod.rs:66-70` with a separate `cadence: [Cadence; 3]` (`:35`).
- No shared trait for bodies: same `encode`/`decode` signatures in `wire/robot.rs:69/115`, `wire/ship.rs:52/71`, `wire/companion.rs:47/64`.
- Client side per kind: `Remotes` holds 3 HashMaps (`state/track.rs:59-63`), `take` has 3 arms (`track.rs:128-142`), `forget.rs:37-39` sweeps 3 times, 3 draw systems (`state/mod.rs:44-46`).
- A 4th kind touches: `wire/frame.rs`, a new `wire/<x>.rs`, `wire/mod.rs`, `room/seat.rs`, `send/kinds.rs`, `send/mod.rs`, `track.rs`, `forget.rs`, `draw/<x>.rs`, `draw/mod.rs`, `state/mod.rs`, `crates/net/examples/bot/player.rs`.
- Frame enums, 4 of them: `space::Frame` (`space/anchor.rs:12`), `ship::SavedFrame` (`ship/body.rs:121`, a serde copy with two `From` impls), `wire::Frame` (`wire/frame.rs:11`), `net::state::Place` (`state/frame.rs:19`).
- `lan_ip`, 2 copies with different behaviour, untested: `src/net/host/open.rs:34` (probes 192.168.0.1:9, falls back to localhost), `crates/server/src/lan.rs:11` (probes 1.1.1.1:80, returns `None`).
- Reason wording in 3 places: `src/net/join/end.rs:31`, `crates/net/src/io/close.rs:41`, `crates/server/src/ping.rs:50`.

Any change to wire encoding breaks protocol compatibility. Keep bytes on the wire identical, or bump the protocol version on purpose.

### C6. Settings: one option touches 5 to 6 files

- Files for one option: `settings/model.rs`, `menus/options/field.rs`, `menus/options/format.rs`, `menus/options/pages.rs`, `settings/apply.rs` or a reader in `render/`, sometimes `options/hint.rs`.
- The 7 graphics flags are mapped twice: `field.rs:56 flag`, `format.rs:17 flag_value`.
- Ranges are not checked on load: bounds exist only in the `bump_*` calls (`field.rs:75-78`, `:106-110`). A hand-edited `settings.toml` with an out-of-range value goes through. This is a trust boundary: validate on load.
- Key bindings mirrored: the 27 key fields of `Keymap` and `ShipKeys` are repeated by the `Bind` enum, its `label` and its `slot` (`options/binds.rs:84-145`).
- One resource, too many reactions: `apply_window` and `apply_graphics` (`settings/apply.rs:46`) run on any change to `Persistent<Settings>`, including saving a server or player name (`multiplayer/act.rs:121,146,166`) and the view-preference save after a wheel zoom (`settings/prefs.rs:57`). Each run re-inserts the camera components (`render/camera.rs:29`) and sets the window mode again.
- The settings menu dims a value by comparing its text to `"Off"` (`options/view.rs:150`).

### C7. Per-frame cost and rebuilds in the interface

- The computer chart is rebuilt every 0.5 s on the System tab: `computer/show.rs:28 rebuild` writes to `Board` every `REFRESH`, which marks it changed, so `rows`, `tabs`, the card facts (`readout.rs:123 despawn_children`) and the whole chart (`draw.rs:239 despawn_related`) run again. Combined with A1, each rebuild also regenerates systems.
- Systems with no run condition: `ui/mod.rs:45` `fps::update` and `camera::sync_output`; `ui/mod.rs:47-55` `focus_on_hover` and `update_looks` (walks every button every frame); `ui/mod.rs:59-60` `sound::escaped` and `scroll::wheel`; `ui/backdrop.rs:113 attach`; `settings/prefs.rs:60 keep` (takes a `ResMut` every frame); `multiplayer/joining.rs:126`; `multiplayer/mod.rs:195 password::remember`; `update/systems.rs:44 poll`; `saves/autosave.rs:69 tick_flash`.
- `menus/update/view.rs:74 refresh` recomputes its page every frame while open, allocating a string on the disk-image page.
- `computer/show.rs:29,160` keep hand-rolled `Local<f32>` timers where the HUD uses `on_timer`.

### C8. Upward imports seen from this side (adds to A8)

| Pair | Imports | Types |
| --- | --- | --- |
| map to menus | 10 | `Marker`, `MarkerKind`, `MarkerPlace`, `Markers`, `MarkerSet` |
| saves to world | 7 | `Globe`, `Terrain`, `Chart`, `Face`, `WorldSeed` |
| net to screens | 4 | `Screen`, `in_control` |
| saves to screens, computer | 3 each | `Screen`, `playing`, `Atlas`, `Destination` |
| saves to ship, menus | 2 each | `SavedFrame`, `ShipState`, `Logbook`, `Menu` |
| net to menus | 2 | `Menu` (`driver.rs:12`), marker types (`friends.rs:13`) |
| settings to view, world, update, render | 2, 1, 1, 1 | `ViewPrefs`, `RENDER_STEPS`, `UpdatePrefs`, `configure_camera` |
| ui to audio | 1 | `AudioCue` |

- 64 files below the top layer import `Screen`, `playing`, `in_control` or `WarmUp`.
- The HUD marker types are imported by 12 files below `menus`; `Menu` by 6, including `space/clock.rs:5` and `gear/fx/mod.rs:74`.
- `saves` is listed as Base but imports about 25 items from 10 higher areas. It acts like a Session-layer module: either split the model from the systems, or fix the table in AGENTS.md.

### Part C, checked and clean

- Text, font and colour use. Distance formatting. Menu open and close (`OnEnter` spawn plus `DespawnOnExit`).
- The net protocol is defined once; `Room` sits behind the `Host` trait; the server crate is thin.
- The HUD is gated and reuses `Local<String>`. Every list `show` is change-gated.
- No gamepad support exists anywhere in `src`.

## Suggested order

1. Layer fixes first, because the rest depends on them: A8 and C8 (move `Screen`, `playing`, `in_control`, `Menu`, the marker types, `Link`, `spin`, `SYSTEM_REACH` down), then A7 (base `math` module).
2. Performance with the best ratio: A1, B1, C7, C6 (the settings reaction).
3. Pure deletions with no behaviour change: B4, B5, B6, C1, C2, C3, A2, A6, C4.
4. Changes the player can see, each one agreed with the owner: A3, A4, A5, B3, B7, the unit harmonisation of C4, the border conflict of C3.
5. C5 last, and only with wire bytes unchanged.

## Risks to keep in mind

- Generation is deterministic from the seed. Unifying hashes or RNG seeding (`climate/heat.rs:46`, `world/structures/site.rs:116`) changes existing worlds and the frozen-output tests. Share code only where output stays bit-identical.
- Nothing here was measured. Take a frame-time baseline before A1, B1 and C7.
- Never weaken a test. Where behaviour changes on purpose, rewrite the test and say so.


## Part D: Bevy practices from the web, applied to this review

One research pass on 2026-11-20: 4 searches, 5 pages read. The Bevy Book pages carry no publication date; they are the current official book at bevy.org.

### D1. Measure first, with Tracy

- The Bevy Book says to measure before optimising, with a baseline and a cost breakdown: https://bevy.org/learn/book/releasing-projects/profiling
- Bevy has built-in tracing spans for ECS systems. Run `cargo run --release --features bevy/trace_tracy`, and record with the Tracy capture tool (`./capture-release -o my_capture.tracy`) instead of the live GUI, which competes for the GPU: https://bevy.org/learn/book/releasing-projects/profiling
- Tracy shows mean time per call (MTPC) per system, and its `Compare` button loads a second trace to compare the same span before and after: https://bevy.org/learn/book/releasing-projects/profiling
- `bevy/trace_tracy_memory` also records the allocations made during a span, at a higher runtime cost: https://bevy.org/learn/book/releasing-projects/profiling
- Add your own spans with `info_span!("name", name = "name").entered()`: https://bevy.org/learn/book/releasing-projects/profiling
- The Tracy version must match the `tracing-tracy` version Bevy uses: https://bevy.org/learn/book/releasing-projects/profiling
- For the hot function inside a system, `cargo flamegraph` with `RUSTFLAGS='-C force-frame-pointers=y'` gives a call tree, with more overhead: https://bevy.org/learn/book/releasing-projects/profiling

Inference: for A1, B1 and C7, capture one Tracy trace on foot, one in flight with autopilot, one with the computer open on the System tab, before any change. Put an `info_span!` in `Universe::planet`, `Universe::star` and `Terrain::surface`, since they are plain functions and not systems, so Tracy will not show them by itself. Compare the same three captures after each change. Use the memory variant once to confirm the per-frame allocations.

Inference: the `dev` feature links Bevy dynamically, so profile a `--release` build without `dev`.

### D2. Release profile

- For desktop speed the Bevy Book shows `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `strip = "debuginfo"`: https://bevy.org/learn/book/releasing-projects/release-builds
- The same page calls `lto = "thin"` a good default with gains close to fat, and suggests trying fat for published builds at the cost of longer builds and more memory: https://bevy.org/learn/book/releasing-projects/release-builds
- It recommends `strip = "debuginfo"` over `strip = true` when binary size is not the goal, because profiling tools use the symbol table: https://bevy.org/learn/book/releasing-projects/release-builds
- It recommends `[profile.dev.package."*"] opt-level = 3`: https://bevy.org/learn/book/releasing-projects/release-builds

State of this repo (`Cargo.toml:65-80`): `dev` already has `opt-level = 1` and dependencies at 3; `release` has `codegen-units = 1`, `lto = "thin"`, `strip = "symbols"`.

Inference: the release profile is already close to the recommendation. Two cheap things to try, each measured: a `profiling` profile that inherits `release` with `strip = "none"` and `debug = "line-tables-only"`, so Tracy and flamegraph show function names; and `lto = "fat"` for tagged builds only, kept only if a trace shows a gain. Neither needs a new dependency.

### D3. Run conditions

- Skipping a system with `.run_if()` cancels it before dispatch, which costs less than an early return inside the system: https://bevy.org/learn/book/control-flow/run-conditions
- A run condition on a `SystemSet` is evaluated once per frame for the whole set: https://bevy.org/learn/book/control-flow/run-conditions
- Bevy ships common conditions for input, timers and states: https://bevy.org/learn/book/control-flow/run-conditions
- The `Single` system parameter skips the system when zero or several entities match: https://bevy.org/learn/book/control-flow/run-conditions

Inference: this covers the ungated systems listed in C7 and B1 (`ui/mod.rs:45-60`, `settings/prefs.rs:60`, `world/sea.rs:116`, `audio/feed.rs:56`). Put one condition on a set per area instead of one per system. Replace the hand-rolled `Local<f32>` timers in `computer/show.rs:29,160` with `on_timer`. Throttle `audio/sense.rs` `shore()` with `on_timer`, as `effects/sense.rs` already does by hand.

### D4. Change detection

- A mutable dereference always marks the value as changed; Bevy does not compare old and new values: https://bevy.org/learn/book/control-flow/change-detection
- `set_if_neq()` writes only when the value differs, and needs `PartialEq`: https://bevy.org/learn/book/control-flow/change-detection
- Reading through `ResMut` without dereferencing mutably does not mark it changed: https://bevy.org/learn/book/control-flow/change-detection
- `Added<T>` and `Changed<T>` filters skip unchanged entities but do not reduce the number of entities the query fetches; they perform the same as calling `.is_changed()`: https://bevy.org/learn/book/control-flow/change-detection
- A system sees every change since it last ran, so a system gated by a state or a run condition misses nothing: https://bevy.org/learn/book/control-flow/change-detection
- A change made by a system that runs after the reader is seen one frame late, unless the systems are ordered: https://bevy.org/learn/book/control-flow/change-detection

Inference: this is the direct fix for C7 and C6. `computer/show.rs:28 rebuild` should build the new rows and write them with `set_if_neq`, so the chart only respawns when a row really changed. `Persistent<Settings>` reactions should compare the fields they care about, or the settings should be split so a saved server name does not re-apply the camera and window. Because throttled systems miss no change, moving a reader to `on_timer` or `resource_changed` is safe. Because `Changed<T>` still walks the archetype, `ui::update_looks` over every button is better served by a run condition or an observer than by a filter alone.

### Not found or uncertain

- No source read on caching procedural generation, terrain sampling or chunk streaming in Bevy. The A1 and B1 designs rest on this review's reading of the code, not on a published practice.
- No source read on Bevy UI rebuild cost. The two search hits were stress-test examples, not guidance.
- The Unofficial Bevy Cheat Book performance page was fetched but its Bevy version is not stated on the page, and a sibling page is marked "0.12 (outdated!)". Weak evidence, not used: https://bevy-cheatbook.github.io/pitfalls/performance.html
- `docs/profiling.md` on docs.rs was fetched but not read; not used: https://docs.rs/crate/bevy/latest/source/docs/profiling.md
- A deeper pass (`/deepresearch`) would be needed for chunk streaming, floating origin and UI practices.

### Sources

- Bevy Book, Profiling: https://bevy.org/learn/book/releasing-projects/profiling
- Bevy Book, Compiling Release Builds: https://bevy.org/learn/book/releasing-projects/release-builds
- Bevy Book, Skipping Systems: https://bevy.org/learn/book/control-flow/run-conditions
- Bevy Book, Change Detection: https://bevy.org/learn/book/control-flow/change-detection
- Not used, weak: https://bevy-cheatbook.github.io/pitfalls/performance.html
- Not used, unread: https://docs.rs/crate/bevy/latest/source/docs/profiling.md
