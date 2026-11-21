# Handoff after the 0.11.0 release (2026-11-22)

Read this first in the next session. It says what shipped, what is broken, what is unmerged and how the work is run.

## State

- **Released**: `v0.11.0`, tag on commit `93cfa96d` (`origin/main`). Release run 44956925641 succeeded: Linux, Windows, macOS universal dmg, server image. https://github.com/saravenpi/Abstractions/releases/tag/v0.11.0
- **Server**: space.facile.studio runs `ghcr.io/saravenpi/abstractions-server:0.11.0`, key `9C4B-499F-B5B5`, new world "Space". The 0.10 world is `/data/world.toml.0.10.bak`; a full volume backup is `~/backups/space-0.10-2338/space-data.tgz`. `dokploy compose one --json` failed (400), so no `compose-before.json` was saved. Runbook: `handoff/DEPLOY-0.11.md`.
- **Protocol**: 4 (raised by `home-air`: the welcome carries the home rule). 0.10 clients cannot join.
- **Branches**: `staging` (worktree `~/Code/Saravenpi/wt-int`) = `main` + this note. `play-0.11` = the owner's play branch (same code as the release). Push only to `play-0.11`, `landing` (website) and, for a release, `main` plus a tag.
- **Website**: not updated for 0.11.0 (still the interim black hero). The hero slideshow of HUD-free landscapes (probe `postcards` stages) is still to do.

## How work is run (owner rules)

- Agents never build. `~/.cache/abstractions-targets/int-logs/NOBUILD.md`: agents edit, `cargo fmt --all`, `filet check .`; the lead alone builds, tests and runs the probe, one build at a time (the VPS crashed on 2026-11-20 from 8 parallel builds).
- One worktree and branch per agent, cut from `staging`; the lead merges into `staging` (wt-int), builds with `CARGO_TARGET_DIR=~/.cache/abstractions-targets/int`, then pushes.
- Before every push to `play-0.11`: `cargo clippy --workspace --features dev --all-targets -- -D warnings`, then a headless boot: `SMOKE_SEED=57 PROBE_STAGES=tour SMOKE_ONLY=tp-stand LP_NUM_THREADS=8 sh ~/.cache/abstractions-targets/lvp/smoke.sh <display> <dir> probe`, and grep the log for `ERROR|panicked` (shader errors only show there; a lost `}` in `sky.wesl` once shipped).
- Merge helpers used: `/tmp/claude-1000/-home-yann/6b3b8623-644b-41d9-a40c-f97789c0bc8d/scratchpad/both.py` (keep both sides of doc/stage conflicts) and `stages.py` (dedupe `examples/probe/stages.rs` and fix its length). They live in a scratch dir; recreate if gone. Never resolve Rust conflicts with them.
- Common clippy fixes after merges: `type_complexity` (name a `type`), `assertions_on_constants` (`const { assert!(..) }`), `chunks_exact` with a constant (`as_chunks::<N>().0.iter()`), `manual_is_multiple_of`, unused re-exports (gate with `#[cfg(test)]`).
- Bevy 0.20 run conditions: use `.and_then` / `.or_else`, not `.and` / `.or`. A system param that only exists with another plugin must be `Option<Res<..>>` or the plugin must `init_resource`, or every test app panics.
- Owner style: plain answers, realism and physics, Swiss-design UI, research online before asking, use the ask tool for choices.

## Failing tests at release (42, CI run on main)

Single-player checks; multiplayer and server crates pass. First job of 0.11.1: run each, decide code or test, fix.
- audio: `ship::flow_tests::a_sideslip_pans_the_air`, `tests::loudness::{a_passing_ship_rises_over_a_calm_day, the_mix_meets_its_targets}`, `tests::pause::a_pause_silences_the_world`, `tests::slides::a_slide_rustles_about_as_loud_as_a_walk`
- avatar: `seated::tests_app::the_seated_body_is_drawn_and_has_no_collider`, `strike::javelin::tests_pose::the_drawn_javelin_lies_along_the_aim`, `strike::tests_pace::blows_take_human_times`
- craft: `pack::tests_pick::a_small_rock_goes_whole_and_is_gone` (stones changed what rocks give)
- fauna: `attack::tests_rest::a_carcass_follows_the_ground_drawn_anew`, `motion::tests_ground::a_body_coming_near_does_not_sink`, `tests_ground::no_foot_of_a_drawn_animal_sinks`, `tests_handover::no_drawn_animal_jumps`, `tests_react::animals_flee_a_near_robot_and_come_back`, `tests_spawn::animals_follow_the_herds`, `tests_walk::animals_stand_on_the_ground`; life: `herd::tests_water::far_herds_keep_out_of_lakes`
- gear: `tests::start::every_start_reaches_a_flyable_glide_and_keeps_it`; player: `tests::landing::{a_fast_glide_lands_on_a_slope, a_fast_glide_lands_on_bare_ground}`
- home: `tests::sites_are_dry_gentle_inland_and_off_the_edges`; world: `terrain::tests_spawn::every_seed_spawns_on_land_in_time` (CI only, likely a time limit)
- map: `deeds_tests::the_card_follows_the_picked_row`; hud: `minimap::tests::speed::a_fast_flight_never_shows_an_empty_texel`
- render: `bodies::tests_app::{every_body_is_lit_as_its_own_place, moons_and_rings_follow_their_planet}`, `tests_season::snow_only_where_it_freezes_and_grows_toward_winter`
- ship: `exhaust::dust::tests_puff::a_puff_never_goes_under_the_ground`, `orbital::tests_glide::the_glide_ends_low_and_short_of_the_target_with_a_mild_glow`, `tests::airless::the_space_assist_works_down_to_an_airless_ground`, `tests::aloft::loops::{a_held_key_loops_and_rolls_in_first_person, ..._third_person}`, `tests::aloft::the_gear_flies_and_hands_back_to_walking_and_swimming`, `tests::cruise::{idle_engines_stop_the_ship, the_path_comes_onto_the_nose_after_a_quarter_turn}`, `tests::fx::powered_flight_and_approaches_never_glow`, `tests::moon_route::a_route_to_a_moon_arrives_from_every_side`, `tests::voyage::a_new_game_flies_to_another_planet_and_resumes`, `tests::voyage::galaxy::a_course_to_another_galaxy_arrives_in_a_few_cycles`
- ui: `units::tests_rules::no_file_writes_a_unit_by_hand`
- `world::scatter::tests_frozen::flat_output_is_frozen`: the stones round changed placement on purpose; paste the printed digests for seeds 1, 5, 42.
- filet warnings to clear: `src/ship/board.rs` (9 functions), `src/universe/record.rs`, `src/world/terrain/spawn.rs` (9 functions), `src/world/sea/mirror/mod.rs` (`spawn` length), `src/avatar/mod.rs:188`. Code that fails filet must not be committed (owner rule).

## Unmerged or unfinished work

- `far-bodies` (worktree `wt-farbody`): far planets drawn as per-pixel lit balls with relief, thin air rim, dithered hand-over, ~37k triangles at 1.5R. Conflicts with `perf` in `src/world/chunks.rs` and `chunks/far.rs`. Merge by hand.
- Label tracking `540f0caf` on `javelin` is merged; verify in play that labels stay pinned on moving animals (owner reported they do not track).
- Owner still to confirm in play: flicker when flying low and fast (fix: anchor rebase moved to `PostUpdate`, `226b1153`, `8b09b11a`); ship landing with the lever up (`rest.rs` settle); camera after galaxy warps (`view/sane.rs`); live herd marks; animal attacks and collisions; landing dust; water near/far colour (judged physical Fresnel; ask the owner with reflections Off).
- Frame rate: owner got ~40 fps on Low on the M2 after the perf round. Measure with `PROBE_STAGES=perf` (prints coarse triangles, distinct visibility ranges, shadowless meshes) and cut: cloud slices (3 shells), 4 grass bands, 3 shadow cascades on Normal, the water mirror camera.
- Known gaps: rivers have no water level (animals cross at full speed); animals do not circle before attacking; snow depth not modelled for steps; the planet's mean temperature is used for the cold-snow squeak.
- Ideas the owner agreed to for later: home gravity band 0.85-1.05 g is in (maybe tighten); ROADMAP has 0.12 "a seat for the companion" and "follow a friend"; keyboard layouts are 0.11.1.

## Agents of this session (all stopped; their worktrees remain)

wt-camera, wt-glide (glide-start), wt-orbital (orbital-cruise), wt-entry (entry), wt-clouds, wt-light3, wt-perf (perf), wt-fground (fauna-ground), wt-javelin, wt-share (share-items), wt-reflect, wt-shipdust, wt-stones, wt-air (home-air), wt-maptiles, wt-grass, wt-setfit, wt-craftui, wt-creaturesnd, wt-combatfeel, wt-farbody. Each branch is merged into `staging` except `far-bodies`. Clean them up with `git worktree remove` once nothing is pending.
