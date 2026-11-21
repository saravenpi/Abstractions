# Working in this repo

Abstractions is a space exploration game written in Rust on Bevy 0.20. This file tells a coding agent how to build, check and change it. Read it before your first edit.

## Layout

| Path | What it holds |
| --- | --- |
| `src/` | The game crate, `abstractions`. One folder per area, see below. |
| `crates/net` | The multiplayer protocol and QUIC transport, shared by the game and the server. |
| `crates/server` | The dedicated server binary. |
| `assets/` | Fonts and other files the game embeds. |
| `packaging/` | Per-platform bundling for releases. |
| `docs/` | Design contracts and handoff notes. |
| `handoff/` | Notes of the 0.11 work: owner requests, plans, formats and design specs. |
| `examples/probe/` | The probe: drives the real game through a tour and saves screenshots. The lead runs it. |
| `ROADMAP.md`, `CHANGELOG.md` | Where the game is going, and what each version changed. |

Areas of `src/`, from the base to the top. A lower area must not import a higher one.

| Layer | Areas |
| --- | --- |
| Base | `universe`, `life`, `space`, `climate`, `sim`, `condition`, `speed`, `settings`, `saves` (model) |
| Drawing | `render`, `weather`, `wind`, `ui` |
| World | `world`, `home`, `effects` |
| Actors | `player`, `view`, `gear` (wingsuit and boots), `avatar`, `companion`, `fauna`, `craft`, `audio` |
| Navigation | `computer` (charts and the course), `map` |
| Ship | `ship` |
| Session | `net`, `update` |
| Screens | `menus` (with `menus/hud`), `screens` |

Some imports still point upward today, for example `screens` using `menus::Menu`. Do not add new ones. A type that two layers need belongs in the lower one.

## Commands

Run through `mise` where a task exists. The lead runs these. Agents do not: see The loop.

| Goal | Command |
| --- | --- |
| Play a dev build | `mise run dev` |
| Check on every save | `mise run watch` (bacon: `c` clippy, `t` tests, `n` net tests) |
| Edit systems in the running game | `mise run hot` (needs `dx`, the Dioxus CLI; its own cache in `target/hot`, about 30 minutes to build the first time) |
| Test one area | `cargo test --features dev --lib -- gear::` |
| Test the net and server crates | `cargo test -p abstractions-net -p abstractions-server` |
| The full gate | `mise run check` |
| See what makes a build slow | `mise run timings` |

The `dev` feature links Bevy dynamically. A dev binary only starts through `cargo run --features dev`, not by its path.

## The loop

Agents never compile. On the night of 2026-11-20 eight agents each ran a Bevy build at once and the build machine crashed. Since then the lead builds one branch at a time, and agents work without a compiler.

An agent may read and edit files, use `git` in its own worktree, run `cargo fmt --all` and run `filet check .`. An agent must not run `cargo build`, `check`, `clippy`, `test` or `run`, `bacon`, a `mise` task, `dx`, the probe, Xvfb or `rustc`, and must not kill a process.

1. Read the code that you change and the code around it. Check every type, import and signature, and every Bevy 0.20 API in `~/.cargo/registry/src/*/bevy*-0.20*`, by reading.
2. Write the change and its tests, as if you could not compile.
3. Before you commit: `cargo fmt --all`, then `filet check .`.
4. Report which tests to run (exact filters), which probe stages to run (`PROBE_STAGES` and the env), and what to look for on each screenshot.

Who checks what:

| Who | Checks | When |
| --- | --- | --- |
| An agent | Reads its change through, rewrites the tests it touches, runs `cargo fmt` and filet | Before each commit |
| The lead | It compiles, clippy, the tests of the areas touched, the probe, then the full gate: every test of the workspace | One branch at a time, after merging, before anything reaches `next` |
| The owner | Feel and looks: flight, camera, animation, plants, interface | As soon as the work lands, in a dev or hot build |

The lead drives the probe: `SMOKE_SEED=1 SMOKE_DIR=<dir> cargo run --features dev --example probe`, with stages picked by `PROBE_STAGES` (`water,grass,fauna,tour,zoo`). Passing tests are not enough: on 2026-11-20 every fauna test passed while the owner saw no animals in the game. An agent that changes something the player sees says in its report exactly what to look at.

The lead sends back the compile errors, the test output and the screenshot paths, and the agent fixes and reports again.

A change anywhere in `src/` rebuilds the whole game crate. The lead batches builds.

## The gate

The lead runs all four before a branch reaches `next`. An agent runs only `cargo fmt --all` and `filet check .` before a commit:

```sh
cargo fmt --all --check
cargo clippy --workspace --features dev --all-targets -- -D warnings
cargo test --workspace --features dev
filet check .
```

- `filet check .` must show nothing beyond the four `gen.config.inert` warnings on `filet.yml`.
- On macOS, `world::scatter::tests_frozen::flat_output_is_frozen` fails because of float rounding. CI runs on Linux, where it passes. Skip it locally with `-- --skip flat_output_is_frozen`. Skip no other test.
- Run `cargo fmt` before `filet`: formatting can push a function past its line limit.

## Code rules

`filet.yml` enforces most of these.

- A file has at most 250 lines and 8 functions. A function has at most 30 lines, 5 parameters, nesting 4 and complexity 10. Lines stop at 120 characters. A module that outgrows its file becomes a folder.
- No comments inside function bodies and no inline comments. No TODO.
- Every item, public or not, has a `///` doc comment. Every file starts with a `//!` module doc. Write plain, short sentences that say what the thing does.
- Tests live in their own files: `foo_tests.rs`, `tests_foo.rs` or a `tests/` folder. Shared test setups are `rig.rs` or `stage.rs`.
- No dead code. Delete what a change stops using.
- No new dependency without asking.
- Never weaken or delete a test to make it pass. When a change replaces a behaviour on purpose, rewrite the test to assert the new behaviour and say so in your report.
- Values come from physics where they can: gravity, air and sizes follow from the planet. Tuning constants live in a `tune.rs` with a doc comment that gives the unit.

## Bevy rules

- One plugin per area, registered in that area's `mod.rs`.
- Gate every system with a run condition. Nothing runs every frame without a reason.
- No allocation per frame in hot paths. Reuse a `Local<Vec<_>>` or `Local<String>`.
- Two queries in one system that reach the same component mutably need `Without<T>` filters. Otherwise the game panics at startup with error B0001, and every test that builds the app fails with it.
- Order systems through the existing sets, such as `ShipSet`, `ViewSet` and `MarkerSet`. Do not read last frame's `GlobalTransform` for something this frame moved.
- Shaders are `.wesl` files beside the code that uses them.

## Design rules from the owner

Direction of 2026-11-20. Tokens live in `src/ui/theme.rs`, durations in `src/ui/motion.rs`, icons in `src/ui/icon.rs`.

- No borders anywhere. Separate things with space, tone and translucency. No rounded corners: `BorderRadius` stays zero, a test checks it.
- The font is Undefined Medium only. Never bold text: the game font renders badly in bold.
- Panels are translucent. Over the live world use `PANEL`; big blurred panels use `PANEL_BLUR` on top of `Blurred`. Text without a panel gets a `TextShadow` and a `PANEL` strip.
- Colour has a meaning, one token each. `TEXT` is primary text, `TEXT_2` secondary text, `TEXT_OFF` disabled only. `ACCENT` marks the primary action, selection and the course. `PIN` marks pins. `WARN` marks a blocked course, weather and heat warnings, a frame under 50 percent. `DANGER` marks a frame under 25 percent and a broken part. A friend has one colour per player. Home and the ship use `TEXT` and differ by icon shape. Use no other colour in interface chrome. Colour is free in the map picture.
- Icons are real pictures from `crate::ui::Icon` (Material Symbols Sharp, square, no round caps). No black box behind an icon. Every icon has a name in words somewhere on screen, and an icon-only control carries a `Tip`.
- Type scale 16, 20, 32, 72 at the 720 pixel reference, at most three sizes on one screen. `UiScale` scales the whole interface with the window height.
- Margins are generous and on the spacing scale `S1` to `S6` (4, 8, 16, 24, 32, 48). The screen edge is `S5`. Panels pad `S4`. Every position is a multiple of 4.
- Every action shows its key. Every empty state says why, in words. The player never has to guess.
- What the HUD shows depends on where the player is: `menus::hud::context::shown` is the one rule. Never show a reading that means nothing where the player stands.
- What the player sees is real: every light, cloud and landmark is an object at that place. Nothing is decoration.

## Commits

- Conventional Commits: `type(scope): summary`, imperative, lowercase, no trailing period, under 72 characters.
- The scope is the area touched, such as `map`, `gear` or `net`.
- No co-author line and no "Generated with" line.
- One logical change per commit. Update `CHANGELOG.md` and `README.md` when the player would notice the change.

## Several agents at once

- Each agent works in its own git worktree, on its own branch. Never commit on `next` or `main`: the lead merges.
- No agent builds. The lead builds one branch at a time.
- Do not push. Report the commit hashes, the tests and probe stages the lead should run, and what you could not check.

## Releases

A release goes from `next`: the full gate, a dry run of `.github/workflows/release.yml` on a throwaway branch, then `main` is fast-forwarded and the tag `vX.Y.Z` is pushed. The tag builds every platform and the server image.
