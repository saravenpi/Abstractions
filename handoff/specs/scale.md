# Scale plan: real light-year, real star spacing, real galaxy size

Read-only audit of /home/yann/Code/Saravenpi/wt-int (branch integrate), 2026-11-20. Nothing was compiled.
Implementers: do NOT compile either; keep edits mechanical, run the listed tests only when the lead builds.

## 0. Decision summary

- One new unit: `universe::LY = 9.4607304725808e15` m. Delete `GLY`. Every "gly" in code, text and tests becomes "ly".
- Keep the sector grid in light-years: `SECTOR_LY = 10.0` (9.46e16 m). Sector counts per lookup stay identical to today, so cost stays identical.
- Real star density: 0.004 stars per ly^3 near the Sun (4 per 10-ly sector). Nearest star about 3.5 ly.
- Real galaxy size: radius 25,000 to 75,000 ly (Milky Way: 87,400 ly across). Real thin disc: about 1,000 ly thick.
- Galaxy grid cell: `GALAXY_CELL = 3.0e6 ly` (2.84e22 m). Neighbour galaxies 1.8 to 4.2 Mly apart (Local Group: Andromeda 3 Mly).
- `UniPos` gets a sector level (f64 alone cannot hold 1e22 m to 1 m).
- Warp: keep the proportional approach law, replace the caps by three tiers set from travel-time targets, shown in ly/s and in c.
- Home: today "nearest living planet to the galaxy centre" breaks at real scale (centre void is 500 to 1,500 ly wide and the core is saturated). Search around a Sun-like point, 0.52 R out in the disc. OWNER QUESTION: this changes the MAC.md rule "nearest to the centre of the home galaxy". Without it, the nearest-star test (3.5 ly) cannot hold.

## 1. Target model (every value with its source)

Sources opened (full URLs):
- https://en.wikipedia.org/wiki/Milky_Way : disc 87,400 ly across (+-3,600), thin disc 718 to 1,470 ly thick, 100 to 400 billion stars, Sun 25,000 to 28,000 ly from centre.
- https://en.wikipedia.org/wiki/Stellar_density : 0.14 stars per pc^3 = 0.004 per ly^3 near the Sun; density falls fast away from the plane.
- https://en.wikipedia.org/wiki/Local_Group : lobes 3 million ly apart (about 800 kpc); group 17 million ly across.
- https://en.wikipedia.org/wiki/Mass%E2%80%93luminosity_relation : exponents 2.3 (<0.43), 4 (0.43 to 2), 3.5 (2 to 55).
- https://en.wikipedia.org/wiki/Hill_sphere : r_H = r (m / 3M)^(1/3); no stable fraction given on that page.
- https://en.wikipedia.org/wiki/Roche_limit : fluid 2.44 R (rho_M / rho_m)^(1/3), rigid 1.26.
- https://en.wikipedia.org/wiki/Frost_line_(astrophysics) : water frost line 2.7 to 3.1 AU (Ceres 2.77).
- https://en.wikipedia.org/wiki/Super-Earth : rocky up to about 1.6 R_earth or 6 M_earth, above that volatile-rich.
- https://en.wikipedia.org/wiki/Initial_mass_function : Kroupa slopes 1.3 (0.08 to 0.5 Msun), 2.3 above. Page gives no red dwarf share.
Unverified (from memory, not opened): Gladman 1993 stability spacing 2*sqrt(3) mutual Hill radii; Domingos 2006 prograde stability 0.49 R_H; dust extinction about 1 mag per kpc in the plane (A_V); Kopparapu 2013 is already cited in `zone.rs` by the repo.

### Constants (`src/universe/units.rs`)
| Name | Value | Note |
| --- | --- | --- |
| `LY` | 9.4607304725808e15 m | exact: c x Julian year (299792458 x 3.15576e7) |
| `PARSEC` | 3.0856775814914e16 m | = AU x 648000 / pi |
| `GRAVITATION` | 6.6743e-11 | CODATA 2018; today 6.674e-11 is 0.005 % off, harmless; move it from `spec.rs` to `units.rs`, one source |
| `EARTH_MASS` | 5.9722e24 kg | new, used only by tests and displays |
| `JUPITER_RADIUS` | 6.9911e7 m | new, test of `GIANT_RADIUS.1` |
| `SPEED_OF_LIGHT` | 299_792_458.0 | new, for the "x c" display |
Keep AU 1.495978707e11, SUN_MASS 1.98847e30, SUN_RADIUS 6.957e8, YEAR 3.15576e7, EARTH_RADIUS 6.371e6, STANDARD_GRAVITY 9.80665.

### Galaxy grid and size (`mod.rs`, `galaxy.rs`)
- `SECTOR_LY = 10.0`. `SECTOR_M = SECTOR_LY * LY`.
- `GALAXY_CELL = 3.0e6 * LY` (3e5 sectors per axis; i32 holds it).
- `RADIUS_LY = (25_000.0, 75_000.0)`; `CENTER_SPAN` unchanged (0.2, 0.8). Cell / radius = 40 to 120, the same ratio as today.
- Galaxy centre is snapped to a whole sector: `center_sector: IVec3` (new field of `GalaxySpec`, from the same draws, rounded). `center` stays as the derived DVec3.
- FILL 0.6, kind odds 60/25/15: keep (tuned, no physical anchor needed).
- Thickness: `thick` 0.008 to 0.015 of R (today 0.02 to 0.04, 2 to 4 times too fat). At R = 50,000 ly that is a sigma of 400 to 750 ly, FWHM about 1,000 to 1,700 ly.

### Star density (`density.rs`), per 10-ly sector
Reference: 0.004 / ly^3 = 4 per sector at 0.52 R in the plane (the Sun, Wikipedia above).
- `DISC` peak = 4 / exp(-0.52 / 0.3) = 22.6. The cap `SECTOR_STARS = 12` clips the inner disc to 0.012 / ly^3 (3 times solar). Say so in the doc: the model cannot show real core density (>1 / ly^3).
- `ARM` peak 1.5 x DISC (arm wave term stays), `CORE` peak 12 (saturated), `BALL` 12, `HALO` 4.0e-3 per sector (4e-6 / ly^3, keep).
- Total stars: test integral of `Shape::parts` over a 50,000-ly spiral = 3e10 to 4e11 (Milky Way 1e11 to 4e11, Wikipedia above). Estimate: pi x (5e4)^2 x ~1,000 ly x ~0.006 = 5e10.
- `Shape::parts` argument `p` is in ly (callers divide by LY, not GLY). `alive` void `smooth(0.01, 0.03, d)` stays (a 250 to 2,250 ly void; real bulge is dense, but the cap makes it a clamp anyway).

### Stars (`star.rs`): already physical, only the unit changes
`pos = (sector + within) * SECTOR_M` (was `* SECTOR_GLY * GLY`). Add `StarSpec.local: DVec3` = `within * SECTOR_M` (metres from the sector corner, exact), so `UniPos` can be built without the 1e5 m rounding of `pos`. `pos` stays for galaxy-frame use.
Class odds disc (M 75 %, K 13 %, G 8 %, F 3 %, A 0.8 %) match the local census; keep. Multiple stars: not modelled (about half of Sun-like stars are multiples): defer, say so in the doc.

### System reach (one definition, `system.rs`)
`system_reach(star) = 4 * ORBIT.1 * M / SUN_MASS` (today, about 2.4e13 m for the Sun, 160 AU). Keep that single function. Every other copy goes: `computer::SYSTEM_REACH` (0.1 GLY), `ship/transit.rs SYSTEM_REACH = ORBIT.1`. Margin to the nearest star: 3.5 ly = 3.3e16 m = 1,400 reaches. Test: over 2,000 stars of the home galaxy, no two stars are closer than `2 * system_reach` (expected: all pass; the nearest pair in the core is about 0.1 ly).
Note: the lead said another agent is unifying this now. Take that change as given; WP1 only deletes leftovers.

### Planets (unchanged formulas, verified)
Kepler, Hill, Roche, gravity, escape, break-up, tidal lock: keep. Fix list in section 5 (rings, spacing).

### Warp (ship) targets
Law kept: `speed = clamp(clearance / RAMP, FLOOR, cap)`, then limited to half the clearance per step. `RAMP = 2 s`, `FLOOR = 2000 m/s`. The law is scale-free: time to cover a gap D without a cap is about 4 ln(D / gap_at_start) seconds. It is a game rule, not physics; no law of nature sets it. What sets it is the owner's travel-time targets. Make them one table in `src/ship/warp/tune.rs` (new file) with doc comments:

| Tier | Used when | Cap | In c | Target |
| --- | --- | --- | --- | --- |
| `CRUISE_CAP` | route <= 50 ly, same galaxy | 1.0 ly/s = 9.4607e15 m/s | 3.2e7 c | nearest star (3.5 ly) in 60 to 100 s |
| `JUMP_CAP` | route > 50 ly, same galaxy | 170 ly/s = 1.608e18 m/s | 5.4e9 c | far side (1e5 ly) in 10 min |
| `GALAXY_CAP` | target in another galaxy | 3,300 ly/s = 3.122e19 m/s | 1.0e11 c | 3 Mly in 15 min |
Estimates (model: out ramp 2 ln(2v / c0) with c0 = 6e12 m leaving a system, decel 2 ln(2v / 3.5e8 m), plus cruise D/v, plus 4 s spool):
- 3.5 ly: 16 + 49 + 4 = about 69 s (never reaches the cap). 10 ly: 16 + 6 + 49 + 4 = about 75 s. 1 ly: about 60 s.
- 1e5 ly at 170 ly/s: 588 + 27 + 46 + 4 = about 665 s (11 min). Raise `JUMP_CAP` to 190 ly/s for 10 min flat: implementers pick the number that makes the test pass inside 540 to 660 s.
- 3 Mly at 3,300 ly/s: 909 + 30 + 46 + 4 = about 990 s. Use 3,600 ly/s for 15 min flat: test band 840 to 960 s.
`JUMP_DISTANCE` (route.rs) = 50 ly (same number as today, new unit). `fraction()` keeps its log scale from FLOOR to the largest cap (`GALAXY_CAP`).

## 2. Order

1. WP1 (universe) and WP2 (space) first: they define `LY`, `SECTOR_LY`, `GALAXY_CELL`, `RADIUS_LY`, `StarSpec.local`, `GalaxySpec.center_sector`, the new `UniPos`. Others may start at once from the contract in section 3, since they only read those names.
2. WP3 (ship), WP4 (sky), WP5 (computer, hud) in parallel after the names exist.
3. Lead runs the gate once, then `examples/probe.rs` shots: home sky, 2 ly out, galaxy edge, outside the galaxy.

## 3. Contract (names every package may rely on)

- `universe::{LY, PARSEC, SECTOR_LY, SECTOR_M, GALAXY_CELL, RADIUS_LY, GRAVITATION, EARTH_MASS, JUPITER_RADIUS, SPEED_OF_LIGHT}`; `GLY`, `SECTOR_GLY`, `RADIUS_GLY` are deleted.
- `universe::system_reach(&StarSpec) -> f64` (metres).
- `StarSpec.local: DVec3`, `GalaxySpec.center_sector: IVec3`.
- `space::UniPos { cell: IVec3, sector: IVec3, local: DVec3 }` with `new(cell, local)` kept (normalises), plus `UniPos::sectors(cell, sector, local)`. `Sub` returns exact differences: `((dcell * SECTORS_PER_CELL + dsector) as f64 * SECTOR_M) + dlocal`. `distance` unchanged.
- `GalaxySpec::locate(pos)` and `Universe::star_position(id)` build the position from integers (`center_sector`, `id.sector`, `star.local`), not from `star.pos`.

## 4. Work packages (disjoint files)

### WP1 universe (model and constants)
Files: `src/universe/{units,mod,galaxy,density,dust,glow,sector,star,home,spec,system,hill,rings}.rs` and tests `src/universe/{tests,range_tests,home_tests}.rs`; new `src/universe/scale_tests.rs`.
Change:
- Constants as in section 1. Move `GRAVITATION` to `units.rs`; `spec.rs` re-exports it.
- `galaxy.rs`: `RADIUS_LY`, `center_sector`, thickness 0.008 to 0.015; `radius: shape.radius * LY`.
- `density.rs`: peaks as in section 1; `GalaxySpec::density` divides by `LY`.
- `dust.rs`: `DUST` is per ly of path. Real plane extinction about 2.8e-4 per ly (1 mag per kpc, unverified). Today 0.004 per gly. Set `DUST = 3.0e-4`, rename the doc "optical depth per ly through the densest lane".
- `glow.rs`: `/ LY` for `stars`, `dust`; `reach`, `slab` multiply by `LY`.
- `sector.rs`: `sector_of(pos) = (pos / SECTOR_M).floor()`; `stars_near(galaxy, pos, radius_ly)`; doc cost line: 55 ly reads about 2,200 sectors.
- `star.rs`: `pos` and the new `local`.
- `home.rs`: search around `P = (0.52 R, 0, 0) in the disc plane`. Replace `shell()` by a shell enumerator that visits only the sectors of the shell (per (x, z) column compute the y range), not the whole cube. `SHELL = 10.0` ly stays, `SHELLS` becomes `ceil(2 R / SHELL)`, `SEARCH = 300` stays. Today the cube enumeration would run 1e9 iterations at 1,500 ly (section 5, item 14).
- `system.rs`: delete leftovers of the old reach; add the spacing rule (see section 5, item 9) only if the lead agrees (it changes every system).
- `rings.rs`: ring outer edge `min(2.4, 2.44 * (rho_planet / rho_ring)^(1/3))`, rho_ring 900 (ice) or 3000 (dust).
Tests to write (expected numbers):
1. Kepler: planet at 1 AU around a 1 Msun star: `orbit_seconds(GRAVITATION * SUN_MASS, AU)` = 365.25 d within 0.1 %.
2. `gravity(EARTH_RADIUS, 4050.0)` = 9.81 within 0.5 % (computes 9.810).
3. `PlanetSpec::hill_radius` equals `hill::hill` within 1e-9 for 200 planets (two definitions, one result).
4. Star density: mean count of stars per 10-ly sector at 0.52 R in the plane, over 400 sectors, is 4 +- 1; over the whole neighbourhood the number within 10 ly of the home star is 10 to 40 for 95 % of 50 seeds.
5. Nearest star to the home star is 1 to 10 ly away for at least 95 % of 200 seeds.
6. Galaxies: `radius / LY` within (25,000, 75,000); neighbour galaxy centres 1.0 to 4.5 Mly apart for 95 % of 100 universes; share present 0.55 to 0.65 (existing).
7. Total stars of one spiral by Monte Carlo (20,000 random sectors x volume) between 3e10 and 4e11.
8. Mass-luminosity: M = 0.2 -> L = 0.23 x 0.2^2.3 = 0.0057 within 1 %; M = 1.5 -> 5.06; M = 10 -> 1.41 x 3162 = 4,470 (existing law, new assertion with Wikipedia exponents).
9. Frost line: flux at the line is 0.14 -> a star of L = 1 has it at 2.67 AU (Wikipedia 2.7 to 3.1); assert `sqrt(L / FROST) * AU` within 1 % of 2.67 AU.
10. No two stars of 2,000 sampled are closer than `2 * system_reach`; no planet orbit inside its star radius x 3.
11. Rings: every ring `outer` <= Roche limit for that planet and ring density (new bound).
12. `home::choose` runs under 2 s in a debug build for 20 seeds (timing guard for the shell rewrite).
13. Existing tests: change `GLY` to `LY`, radii ranges, `star.pos.distance(at) <= 60.0 * LY`. `near.len() > 10` within 60 ly stays true (about 3,600 expected).

### WP2 space precision, saves, wire
Files: `src/space/{pos,mod,tests}.rs`; `src/universe/spec.rs` is WP1; `crates/net/src/ids.rs` (doc only: "in steps of `SECTOR_LY` light-years"); `handoff/FORMAT3.md` (one line: galaxy frame positions are three `f64`, lossless at 2.4e22 m); new tests `src/space/tests_far.rs`, `crates/net/src/wire/far_tests.rs`, `src/saves/tests_far.rs` (or the existing save test files).
Change:
- `UniPos` as in the contract. `SECTORS_PER_CELL = 300_000` (GALAXY_CELL / SECTOR_M). `new(cell, local)` normalises `local` into `sector` and `local` (floor division by `SECTOR_M`), then `sector` into `cell`.
- `Add<DVec3>`, `Sub<DVec3>`: add to `local`, renormalise. `Sub<UniPos>`: exact form above.
- Document the precision: f64 `local` < 9.46e16 gives 10 m steps; differences inside one galaxy are exact to 1e-16 of the result.
Tests:
1. Two positions in one galaxy 1,000 m apart, 3e20 m from the galaxy centre: `(a - b).length()` = 1,000 m within 20 m. (Old `UniPos` with the new cell would give 4e6 m noise.)
2. A star 4 ly away: `a - b` length 4 ly within 1e-12 relative.
3. Round trip `new(cell, local)` for 1,000 random values up to 2.8e22 m: `(p - q)` equals the f64 difference within 1e7 m (across galaxies).
4. Wire: galaxy-frame ship position (5e20, -3e20, 2.4e22) packed and unpacked equals the f64 (the three f64 path; if the ship uses `pack_cells`, error under 1e5 m because the input f64 step is 1e5 m).
5. Save: a ship at `Frame::Galaxy` with pos (4.7e20, 0, -1e21) written to TOML and read back is bit-equal.
No protocol or `SAVE_VERSION` bump: both are unreleased, formats do not change.

### WP3 ship (warp law, lead, transit, sky lookups)
Files: `src/ship/{warp,route,transit,sky}.rs`, `src/ship/route/{goal,align}.rs` (only constants), `src/ship/warp/tune.rs` (new), `src/ship/warp/tests.rs`, `src/ship/tests/{fx,resume}.rs`, `src/ship/route/*_tests.rs` that name GLY.
Change:
- `warp.rs:22,25`: `CRUISE_CAP`, `JUMP_CAP`, new `GALAXY_CAP` from `warp/tune.rs` (values in section 1). `fraction()` uses `GALAXY_CAP`.
- `route.rs:49` `JUMP_DISTANCE = 50.0 * LY`; `route.rs:142-143` cap choice by tier (same galaxy and <= 50 ly: cruise; same galaxy and farther: jump; other galaxy: galaxy cap). `autopilot.jump` stays a bool for "blind to stars"; add a second flag or an enum `Tier`.
- `route/goal.rs:40,53-54`: fallbacks `GLY` become `LY` (a target without a star: lock 1 ly). `goal.rs:49`: `stars_near(id, ZERO, 30.0)` becomes 30 ly; the centre of a galaxy is a void of 250 to 2,250 ly, so 30 ly finds nothing. Use the first star of a search that grows (10, 100, 1,000, 5,000 ly) or the galaxy `lock = 0.03 R` beacon. Do this with a loop of at most 4 calls; the 5,000-ly call reads (1000/10)^3 = 1e6 sectors: too slow. Use the density peak instead: sample `home`-like shells only up to 1,000 ly, else fall back to the galaxy beacon with `lock = LY`.
- `transit.rs:19`: delete the local `SYSTEM_REACH`; `leave_system` and `enter_system` use `system_reach(star)` x 1.05 margin. Increase hysteresis: enter at `reach`, leave at `1.05 reach` (already). Doc: a ship at 5e20 m in the galaxy frame has a 1e5 m position step; the 5 % margin (1e12 m) is far above it.
- `sky.rs:20`: `STAR_SCAN = 12.0` ly stays (same sector count as today: 64 to 125 sectors, expected 29 stars). Doc the unit.
- Precision note in `body.rs` doc: `Frame::Galaxy` position is f64 from the galaxy centre: step 1e5 m at 5e20 m, 4e6 m at 2.4e22 m. Slow motion (2,000 m/s x 16 ms = 33 m) is below that step: the ship does not move in the galaxy frame at the rim without warp. Acceptable: nothing runs at that speed in that frame; `FLOOR` applies only inside a lock sphere, where the ship is already in a system frame. Add a test (below) that fails if a well can be closer than `system_reach` while in the galaxy frame.
Tests (use the simulated step `DT = 1/30` as in `warp/tests.rs`):
1. Warp from 40 AU of a star to the nearest star (3.5 ly) takes 55 to 100 s of simulated time.
2. Warp over 10 ly takes 60 to 120 s; over 1 ly 50 to 100 s.
3. Jump over 1e5 ly takes 540 to 660 s (the bounds follow the chosen cap).
4. Galaxy jump over 3 Mly takes 840 to 960 s.
5. The ship never ends inside a star lock sphere: arrival clearance >= 0 for 20 stars.
6. `fraction(CRUISE_CAP)`: replace 0.711 by `ln(CRUISE_CAP / FLOOR) / ln(GALAXY_CAP / FLOOR)` = ln(4.73e12) / ln(1.56e16) = 29.2 / 37.3 = 0.783 (recompute from final caps).
7. `enter_system`: a ship 0.9 `system_reach` from a star enters; at 1.1 it does not.
8. `resume.rs`: replace `far * GLY` by `far * LY`; keep the frame.
9. Lead: a destination moving at 30 km/s over a 600-s flight is led by 1.8e7 m: `align::led` returns a point within 1e3 m of the true arrival point (f64 holds it; test at 1e17 m range).

### WP4 sky (starfield, galaxy light, sun, bodies)
Files: `src/render/starfield.rs`, `src/render/starfield/{build,specks,tests,tests_depth,tests_sight}.rs`, `src/render/galaxy_light.rs`, `src/render/galaxy_light/{march,tune,tests,tests_gap,tests_sight}.rs`, `src/render/sun.rs`, `src/render/bodies.rs`, `src/render/bodies/tests_star.rs`, `src/render/tests_moonday.rs`, `src/menus/hud/minimap/tests/home.rs` (one line).
Change:
- `GLY` to `LY` everywhere (`starfield.rs:31,62`, `specks.rs:34`, `build.rs:57`, `march.rs:95-99`, `galaxy_light.rs:37`). `star_light` flux is `L / d^2` with d in ly: unchanged formula, now real inverse square in ly.
- `REACH_GLY = 55.0` stays as `REACH_LY` (sphere holds 2,800 stars at 0.004; 8,400 in an arm; `MOST_STARS = 9000` stays). Rebuild trigger `SECTOR_LY * LY`.
- `galaxy_light.rs:37` `INSIDE = 50 GLY` becomes `0.02 x galaxy radius` (about 1,000 ly), the same ratio as `OUTSIDE = 0.02`. The old value was 2 to 6 % of R; this keeps it.
- `march.rs`: `NEAR = 40` ly stays. With R = 50,000 ly and 24 steps, step is 2,000 ly (was 8 gly): the NEAR cut is now smaller than one step; keep it (it only avoids double counting) but space the 24 steps geometrically from NEAR to the far edge (ratio r = (far/NEAR)^(1/24)), so the first step is 40 ly and structure near the camera resolves. Same step count, same bake time.
- `GAIN = 1/700`: surface brightness is proportional to the column density `n x path`. Column toward the core at home changes by (new n x new path) / (old n x old path): about (4 x 5e4 x w) / (0.5 x 2e3 x w), 200 times. Set `GAIN = 1/700 / 200 = 1/140_000` as a first value and fix it with the test below (do not tune by eye): the baked peak luminance at the home position must equal the 0.10.1 reference within 15 %. Reference value: run the old tree's `radiance` once (lead does this; record the number in `tune.rs` as a doc comment).
- `sun.rs:152`, `bodies.rs:115`, `specks.rs:49`: already call `system_reach`; no change besides imports.
Tests:
1. `star_speck` at 2 ly vs 30 ly: near brighter (existing, now in ly).
2. Stars drawn at home: `gather(universe, home galaxy, home pos)` returns 1,500 to 9,000 stars (a 55-ly sphere is 696,000 ly^3, times 0.004 = 2,800).
3. Gather time under 100 ms off-thread (2,200 sectors; today the same count).
4. Bake: band peak lies within 2 degrees of the disc plane; brightest toward the core (existing tests, keep thresholds). Brightness continuity along a warp from outside to inside: no step above 5 % (existing).
5. Aspect ratio of a spiral seen from outside equals tilt within 10 % (existing).
6. Bake time under 300 ms (existing, unchanged step count).
7. Dust: optical depth through the whole disc edge-on (ray along the plane, R = 50,000 ly) is between 1 and 6 (Milky Way is opaque toward the core in the plane); face-on through the centre is under 0.5.

### WP5 computer, charts, HUD units
Files: `src/computer/{format,locate,reveal,atlas,track,scale,chart,rows}.rs`, `src/computer/tests*.rs` that name GLY or `SYSTEM_REACH`, `src/menus/hud/{units,flight}.rs`, `src/menus/hud/tests.rs`.
Change:
- `format.rs:3,30`: `distance(metres)` gains units: m, km, Mm, Gm (1e9), AU (from 0.1 AU = 1.5e10 m), ly (from 0.1 ly = 9.46e14 m), kly, Mly; three significant digits, at most `DISTANCE_CHARS` (test: `distance(102_007 ly)` = "102 kly", under the limit). Same table for `hud/units.rs` (heights and speeds); speeds above 1e9 m/s also show "x c": `3.2e7 c`.
- `locate.rs:10,44`: delete `SYSTEM_REACH = 0.1 GLY`; `whereabouts` uses `system_reach(star spec)`. `whereabouts` needs the star spec: `universe.star(id)` (O(1) sector fill). Update `tests_tables.rs:150,156` to use `system_reach`.
- `reveal.rs:8`: `REVEAL_GLY = 60` becomes `REVEAL_LY = 10.0`. At 0.004 per ly^3, 60 ly would list about 3,600 stars per visited system (today about 450); 10 ly lists about 17, a human size and close to the real 10-ly census. `systems_around` stays one `stars_near` call (about 8 sectors).
- `reveal.rs:52`: `safe = cells * GALAXY_CELL` stays (cell is now 3 Mly). Galaxy lists (`galaxies_near(id, 2)`): 125 cells, unchanged cost.
- `scale.rs`, `chart.rs`: charts take f32 offsets from the chart centre; system charts stay metres to 1e13 (fine). Galaxy and universe charts: distances up to 1e22 m in f32 (3e38 limit, relative 6e-8): fine. The sqrt squash keeps inner rows visible.
- `home_galaxies`: `galaxy_gap` uses the new `UniPos` difference (exact).
- Search lists sorted by distance keep working; label text "gly" to "ly".
- `src/map/`: only `Level::{Chart, Planet}`; the Galaxy and Universe charts live in `computer/`. `map/spots.rs` uses `UniPos` through its API: compile-check only.
Tests:
1. `distance(1.0e3)` "1.00 km"; `distance(1.5e11)` "1.00 AU"; `distance(9.4607e15)` "1.00 ly"; `distance(9.4607e18)` "1.00 kly"; `distance(3.0e6 * LY)` "3.00 Mly"; every output <= `DISTANCE_CHARS`.
2. `speed(3.0e8)` contains "1.00 c"; `speed(1.608e18)` contains "170 ly/s".
3. `systems_around(home)` returns 5 to 60 stars for 95 % of 50 seeds, sorted by distance.
4. `whereabouts` in `Frame::System` at 0.5 and 1.5 `system_reach`: inside, outside.

## 5. Audit result in short (details in the report)

Wrong today (all by the same cause: the galaxy was drawn 1e6 times too small):
1. `GLY` 1e10 m vs light-year 9.4607e15 m (factor 9.46e5).
2. `GALAXY_CELL` 1e15 m (1e5 gly) vs 2.8e22 m (3 Mly).
3. Galaxy radius 800 to 2,500 gly vs 25,000 to 75,000 ly.
4. Disc thickness 2 to 4 % of R vs about 1 %.
5. Star density 0.0005 per gly^3 (disc) vs 0.004 per ly^3.
6. `DUST` 0.004 per gly: at real path lengths it blacks out the sky (tau 4 over 1,000 ly).
7. `GAIN` 1/700 tied to old column densities.
8. `UniPos` local f64 inside a 1e22 m cell: 4e6 m noise, above planet lock radii.
9. Planet spacing: "Hill spheres do not touch" is looser than the stability limit (about 3.5 mutual Hill radii, unverified Gladman 1993). Optional: floor at `a_in + 3.5 * R_H(mutual)`. Not needed for scale; do only if the owner asks.
10. Ring outer edge 2.4 R can pass the real Roche limit for light giants with ice rings (Saturn: 2.2 R).
11. Greenhouse `33 K sqrt(P)`: Venus 92 bar gives 316 K vs about 500 K real: tuned.
12. `home::choose` shell loop is O(R^3) per shell: 1e9 iterations at the new void size. Must be rewritten (WP1).
13. Orbits are all circular and coplanar: simplification, keep.
14. Warp caps are tuned (5 gly/s, 5000 gly/s), not physical.

## 6. Performance budget

- Sector lookups: `sector_of` O(1); `stars_in_sector` O(stars in sector, at most 12).
- `stars_near(r)`: cost = (2r/10 + 1)^3 sectors, as today. 12 ly (ship scan): 64 to 125 sectors, on the main thread, only when the ship changes sector (every 10 s at cruise). 55 ly (starfield): about 2,200 sectors in a task. 10 ly (reveal): about 8 sectors. Nothing above 55 ly may be called anywhere; test: grep-style unit test is not possible, so review: `stars_near` calls with radius > 60 are bugs.
- Stars per frame: unchanged. The starfield is one mesh of at most 9,000 specks, rebuilt when the camera moves 10 ly (once per 10 s at cruise; at jump speed it is continuous background work on one pool thread, as today. Optional: skip rebuilds while `Lens.beta` says jump).
- A 1e11-star galaxy stays a pure function: no per-galaxy tables, no star counts stored. Per sector 1 to 12 stars from one RNG. Total count is an integral, computed in tests only.
- `Universe::new`: the home search visits shells around the home point; budget 300 systems and 2 s in debug (WP1 test 12).
- Galaxy light bake: 6 x 128 x 128 rays x 24 steps, off thread, under 300 ms (unchanged).

## 7. Risks

- Home point change (owner question above). Without it, home sits at the saturated core edge with 12 stars per sector and the neighbour test fails.
- Real density at 0.004 per ly^3 makes `stars_near(60)` return 3,600 stars; any caller left at 60 or 100 would stall a frame. WP5 fixes reveal; review the other callers (`ship/route/goal.rs`, `render/tests_moonday.rs`, `starfield`).
- `UniPos` change touches every caller that builds one from `center + pos`. Keep `new(cell, local)` so old call sites compile; only `locate` and `star_position` need the exact path.
- Position step in the galaxy frame (1e5 m at the rim, 4e6 m between galaxies): fine for warp, wrong for any slow motion. A future sublight drive in that frame needs a per-sector origin (`Frame::Sector`), which this plan does not add.
- `GAIN` and `DUST` are calibrated, not derived: the lead compares probe screenshots (home sky, edge, outside) before and after; a mismatch is a one-line constant fix.
- Warp tier numbers are estimates from a closed-form model. The tests carry bands, not points; adjust the caps, not the bands.
- All worlds and saves of dev builds change (positions, home, star ids). Fine: protocol 3 and `SAVE_VERSION` 3 are unreleased. Probe seeds will show different systems.
- Names and seeds: star seeds come from `hash::cell(shape.seed, sector)` and an RNG stream; star names from the star seed. Positions do not feed seeds. Star counts per sector do change (density), so names per sector index shift; accepted.
- Tests that freeze distances (all listed in the packages): universe `tests.rs`, `range_tests.rs`, `home_tests.rs`; starfield tests; ship `warp/tests.rs` (`fraction(CRUISE_CAP) = 0.711`), `tests/fx.rs` (speeds), `tests/resume.rs`; computer `tests_width.rs`, `tests_tables.rs`, `tests_homeward.rs`, `tests.rs`; hud `tests.rs` (`9.995e8` "0.10 gly"); minimap `tests/home.rs`; `galaxy_light/tests.rs` (offset 1e9 x GLY becomes 1e9 x LY = 9.5e24 m: beyond a cell; use 1e6 x LY).
