# Review of 0.11.0 and start of 0.11.1 (2026-11-22, partial)

Read with `handoff/NEXT-0.11.1.md`. Not committed. The session ran out of usage mid-review.

## Status

- Scope: `git diff v0.10.1 v0.11.0` (2558 files, +172840/-27897); `staging` adds only the handoff note.
- Baseline on ruche (`CARGO_TARGET_DIR=~/.cache/abstractions-targets/int cargo test --workspace --features dev --no-fail-fast`): 41 failing, 2823 passed; net and server crates green. Same set as CI run 44956932796, plus `world::scatter::tests_perch::stones_rest_only_where_they_can` (not in the handoff list). The list is at the end.
- filet: only 2 warnings beyond the 4 inert ones: `src/ship/board.rs` and `src/world/terrain/spawn.rs` (9 functions each). 20 info long lines.
- Reviewed by read-only agents: creature labels, world props, world ground + home, life + effects + companion.
- NOT reviewed yet (agents stopped): `src/ship`, `src/fauna`, `src/avatar` + `player` + `gear` + `view`, `src/menus` + `ui` + `screens` + `settings`, `src/map` + `computer` + `skin` + `condition` + `speed`, `src/render` + `weather` + `wind` + `climate`, `src/audio` + `craft`, `crates/net` + `crates/server` + `src/net` + `universe` + `saves` + `space` + `update` + scripts/CI. The brief they used is reproduced below.

## Owner requests for 0.11.1 (this session)

- Fix creature name labels and health bars: they stay on one spot and sink under the ground as animals walk away. Root cause found (section 1).
- Workflow: review first, then `/plan`, then parallel agents that only edit (never compile); the lead compiles and tests one branch at a time, then pushes to `play-0.11` so the owner can pull on the other laptop.
- Still pending from the handoff: keyboard layouts (branch `keys`, commit f1dc9bc1, unmerged), `far-bodies` (unmerged, conflicts with perf in `src/world/chunks.rs`, `chunks/far.rs`), frame rate on M2 Low (~40 fps).

## 1. Creature labels

`src/menus/hud/foe/place.rs:72-84` `crown` anchors the label on `root.translation`, the animal's frame origin (`Placed`, set at spawn on the sea-level sphere and only refit past DRIFT = 120 m, `fauna/tune.rs:43`, `walk.rs:245`). The body moves through part `Transform`s inside that frame. So the label stays over the spawn point, and its height (body top above the sea-level frame) sinks below ground when the animal walks downhill.

Fix: `crown(root, base, limbs)` with `foot = root.transform_point(beast.motion.root.place)`, height measured from `foot`; `top` returns `(foot, head)`; `gather` in `foe/mod.rs` uses `foot` for the looked midpoint and FAR check. No set changes (`show` runs after `fauna::Posed`).

Tests: rewrite `tests_track::a_label_tracks_a_running_animal` (it moved `Placed`, the reverse of the game); add `a_label_stays_over_an_animal_walking_downhill`. Filter `menus::hud::foe::`. Probe `PROBE_STAGES=play SMOKE_ONLY=play-herd` shots e30/e31, `PROBE_STAGES=fauna SMOKE_ONLY=hill`.

## world props (structures->home moved, blades, propmesh, scatter, rocks)
- IMPORTANT scatter/perch.rs:105-107 settle keeps keep*HOLLOW<welcome, HOLLOW=2: flat ground keeps 50% of stones. Fix: double stone weights in table.rs + flora stone chance.
- IMPORTANT world/props.rs:134-136 body_collider_on and rocks/stand.rs:30-37 mid ignore the lean tilt of Rig::pose (64ab7837): collider upright while drawn stone tilts ~0.6 m on 35 deg slope. Fix: use rig.pose(p).
- IMPORTANT propmesh/grown/book.rs:56-62 look book 253 pages wraps with two flora stages (144 looks each); kept placements (mend repair, canopy.rs:119) read overwritten page -> wrong species mesh. Fix: keep Arc<Soup>/Look in Stand or per-planet LRU.
- NIT blades/mod.rs:131 blow has no run condition. scatter/relief.rs near clamps ring at chunk edge. rocks/stand.rs:31 id_of twice.
- TEST world::scatter::tests_frozen::flat_output_is_frozen STALE: seed 1 -> 0xBE76_3CF0_6BD3_E3E3; collect all 3 seeds then assert; land after scatter fixes.
- TEST craft::pack::tests_pick::a_small_rock_goes_whole_and_is_gone STALE: Rock now basalt 3000 -> 2.77 kg > HANDFUL 2.5. Use smaller scale/volume from extent().
- TEST world::scatter::tests_perch::stones_rest_only_where_they_can (NOT in handoff list) CODE: only 20 stones < 30 floor; fixed by doubling weights.
- UNVERIFIED rocks/id.rs id_of rounds y to 0.5 m: id flip if heights differ between builds -> taken rock returns.

## world ground (terrain, globe, chunks, mesh, sea, mirror) + home
- IMPORTANT chunks/spawn.rs:98-108 + chunks/horizon.rs:146-160: Spawner::coarse spawns NotShadowCaster+Shadowless but Node{shadowless:false}; shade() early-returns when node.shadowless==far, so a leaf spawned near never casts. f7aeb1ce vs 2bd020d8 disagree on the rule. Fix: spawn with shadowless:true (+ matching components) so first shade settles it.
- NIT terrain/spawn.rs 9 fns: delete ring_points (one-line wrapper, used once landfall.rs:68) -> fixes filet warn.
- NIT sea/mirror/mod.rs:89-110 spawn 31 lines: extract fn camera().
- NIT blades/mod.rs:136 blow no run condition (dup of props report).
- NIT chunks/cut.rs:138 + fade.rs:113-120 one frame of dither gaps when fades.span changes.
- TEST world::terrain::tests_spawn::every_seed_spawns_on_land_in_time FLAKY: seed 36 0.226s > 0.2; landfalls() cache filled inside timed span. Fix: warm globe.landfalls() before timing.
- TEST home::tests::sites_are_dry_gentle_inland_and_off_the_edges STALE: bdffa4e3 runs all PASSES incl slack 2 per shore; seed 9 relief 7.45 > 7.0. Fix: assert against the slack of the pass used (record it) rather than MAX_RELIEF+1.
- UNVERIFIED horizon.rs:126-136 floor() max(lowest, sea) hides dry basins below sea level.
- UNVERIFIED mirror/aim.rs Plane::under reads water.eye from Update; may be one rebase stale for a frame after PostUpdate rebase move.

## life, effects, companion
- IMPORTANT life/herd/roll.rs:91 + herd/route.rs:113: herd home never checked for water; walkable() skips start point; via:roam.home and fallback->home leg unchecked -> herds walk through lakes (cause of far_herds_keep_out_of_lakes). Fix: home = cell centre when drawn spot under floor(species); drop Roam::fallback; pass ground.height into roll::around.
- IMPORTANT life/hunt/chase.rs:51: hunter chases slerp straight through water/slope. Fix: plan::one skip hunt when walkable(...) false.
- IMPORTANT effects/shed/gather.rs:14 + effects/respawn.rs:96: leaf Source.at stored eye-relative every 500 ms; settle uses current eye -> leaves spawn from empty air in low flight. Fix: store in anchor frame, recompute at respawn, skip out of box.
- IMPORTANT layer rule: life imports world (census.rs:10, census/open.rs:8 Biome; herd/cells.rs:14, herd/route.rs:22, hunt/prey.rs:9 Tangent); effects/sim.rs imports view::Target. Fix: move Biome/Tangent to base area; pass camera velocity.
- NIT effects/shed/gather.rs:62 tint repaints slot mid-air. effects/water/foam.rs:149 + foam.wesl:43 Clock::phase wraps 65536 s. life/herd/watch.rs:56 hunter dropped in outer band. effects/eye.rs:51 second impl block.
- TEST life::herd::tests_water::far_herds_keep_out_of_lakes CODE (fix above).
- TEST ship::exhaust::dust::tests_puff::a_puff_never_goes_under_the_ground STALE: assert flow.dot(at.normalize()) >= -1e-6 instead of dot(X).
- UNVERIFIED life/signs/calls.rs:155 dead slot asked to call.

## Baseline failing tests (41)

- `audio::ship::flow_tests::a_sideslip_pans_the_air`
- `audio::tests::loudness::a_passing_ship_rises_over_a_calm_day`
- `audio::tests::loudness::the_mix_meets_its_targets`
- `audio::tests::pause::a_pause_silences_the_world`
- `audio::tests::slides::a_slide_rustles_about_as_loud_as_a_walk`
- `avatar::seated::tests_app::the_seated_body_is_drawn_and_has_no_collider`
- `avatar::strike::javelin::tests_pose::the_drawn_javelin_lies_along_the_aim`
- `avatar::strike::tests_pace::blows_take_human_times`
- `craft::pack::tests_pick::a_small_rock_goes_whole_and_is_gone`
- `fauna::attack::tests_rest::a_carcass_follows_the_ground_drawn_anew`
- `fauna::motion::tests_ground::a_body_coming_near_does_not_sink`
- `fauna::tests_ground::no_foot_of_a_drawn_animal_sinks`
- `fauna::tests_handover::no_drawn_animal_jumps`
- `fauna::tests_react::animals_flee_a_near_robot_and_come_back`
- `fauna::tests_spawn::animals_follow_the_herds`
- `fauna::tests_walk::animals_stand_on_the_ground`
- `gear::tests::start::every_start_reaches_a_flyable_glide_and_keeps_it`
- `home::tests::sites_are_dry_gentle_inland_and_off_the_edges`
- `life::herd::tests_water::far_herds_keep_out_of_lakes`
- `map::deeds_tests::the_card_follows_the_picked_row`
- `player::tests::landing::a_fast_glide_lands_on_a_slope`
- `player::tests::landing::a_fast_glide_lands_on_bare_ground`
- `render::bodies::tests_app::every_body_is_lit_as_its_own_place`
- `render::bodies::tests_app::moons_and_rings_follow_their_planet`
- `render::tests_season::snow_only_where_it_freezes_and_grows_toward_winter`
- `ship::exhaust::dust::tests_puff::a_puff_never_goes_under_the_ground`
- `ship::orbital::tests_glide::the_glide_ends_low_and_short_of_the_target_with_a_mild_glow`
- `ship::tests::airless::the_space_assist_works_down_to_an_airless_ground`
- `ship::tests::aloft::loops::a_held_key_loops_and_rolls_in_first_person`
- `ship::tests::aloft::loops::a_held_key_loops_and_rolls_in_third_person`
- `ship::tests::aloft::the_gear_flies_and_hands_back_to_walking_and_swimming`
- `ship::tests::cruise::idle_engines_stop_the_ship`
- `ship::tests::cruise::the_path_comes_onto_the_nose_after_a_quarter_turn`
- `ship::tests::fx::powered_flight_and_approaches_never_glow`
- `ship::tests::moon_route::a_route_to_a_moon_arrives_from_every_side`
- `ship::tests::voyage::a_new_game_flies_to_another_planet_and_resumes`
- `ship::tests::voyage::galaxy::a_course_to_another_galaxy_arrives_in_a_few_cycles`
- `ui::units::tests_rules::no_file_writes_a_unit_by_hand`
- `world::scatter::tests_frozen::flat_output_is_frozen`
- `world::scatter::tests_perch::stones_rest_only_where_they_can`
- `world::terrain::tests_spawn::every_seed_spawns_on_land_in_time`

## Review brief used by the agents

## Review brief (shared by every review agent)

Repo: /home/yann/Code/Saravenpi/wt-int (worktree on branch `staging`). Abstractions, a Rust space game on Bevy 0.20.
Task: review the 0.11.0 release, `git diff v0.10.1 v0.11.0 -- <your paths>`, for the areas you are given. `staging` only adds one handoff note on top.
Read first: `AGENTS.md` (code, Bevy and owner design rules), `handoff/NEXT-0.11.1.md` (state after release, known failing tests), the 0.11.0 section of `CHANGELOG.md` (intent).

### Hard rules

- READ ONLY. Do not edit, write, commit, stash, checkout or switch branches. Do not touch any file in the repo.
- Never compile: no `cargo build/check/clippy/test/run`, `bacon`, `mise`, `rustc`, the probe, Xvfb. Do not kill processes. (The build machine crashed when agents compiled.)
- Allowed: `git diff/log/show/grep/blame`, `grep`/`rg`, reading files, reading Bevy 0.20 source under `~/.cargo/registry/src/*/bevy*-0.20*`.

### Method (from the /review skill)

1. Size your diff with `git diff --stat v0.10.1 v0.11.0 -- <paths>`. Review group by group, biggest and riskiest first.
2. Design pass: does it fit the problem, simpler way, belongs elsewhere.
3. Line pass: correctness (edge cases, NaN/inf, division by zero, off-by-one, frame-order bugs such as reading last frame's GlobalTransform, B0001 query conflicts, missing run conditions), performance (per-frame allocation, work every frame without need, O(n^2) over entities), Bevy 0.20 API misuse, net/protocol issues, AGENTS.md rule breaks (design rules: borders, rounded corners, bold, colour tokens, BoxShadow; code rules: dead code, comments in bodies).
4. VERIFY every finding: open the file, read the whole function, grep callers and definitions, write a concrete scenario "when X, line N does Y, so Z breaks". No scenario: drop it, or list it as Unverified. Zero findings is fine. Pre-existing problems outside the diff are not findings unless the change makes them worse.
5. Failing tests: the CI failure log of the release is at `/tmp/claude-1000/-home-yann/96905b17-8f4d-4c58-b3c2-8ba3b92bbb0e/scratchpad/ci-failed.log` (grep your test names; panic messages follow each FAILED). For each failing test in your areas (list in `handoff/NEXT-0.11.1.md`), read the test and the code under test and decide: CODE bug (the game is wrong), STALE test (behaviour changed on purpose; say what the new assertion should be), or FLAKY/time limit. Give the evidence and the smallest fix.

### Output

Your final message is the report, at most ~1200 words, in this structure (omit empty sections):

```
### Area: <name>  (diff: N files, +A/-D)
#### 🔴 Critical
- path:line: what is wrong. Scenario: ... Fix: ...
#### 🟡 Important
#### 🟢 Nits (max 5)
#### Failing tests
- test::path: CODE|STALE|FLAKY. Evidence. Fix.
#### Unverified
#### Health
2 sentences.
```
