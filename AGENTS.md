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
| `ROADMAP.md`, `CHANGELOG.md` | Where the game is going, and what each version changed. |

Areas of `src/`, from the base to the top. A lower area must not import a higher one.

| Layer | Areas |
| --- | --- |
| Base | `universe`, `space`, `climate`, `sim`, `settings`, `saves` (model) |
| Drawing | `render`, `weather`, `wind`, `ui` |
| World | `world`, `home`, `effects` |
| Actors | `player`, `view`, `gear` (wingsuit and boots), `avatar`, `companion`, `audio` |
| Navigation | `computer` (charts and the course), `map` |
| Ship | `ship` |
| Session | `net`, `update` |
| Screens | `menus` (with `menus/hud`), `screens` |

Some imports still point upward today, for example `screens` using `menus::Menu`. Do not add new ones. A type that two layers need belongs in the lower one.

## Commands

Run through `mise` where a task exists.

| Goal | Command |
| --- | --- |
| Play a dev build | `mise run dev` |
| Check on every save | `mise run watch` (bacon: `c` clippy, `t` tests, `n` net tests) |
| Edit systems in the running game | `mise run hot` (needs `dx`, the Dioxus CLI) |
| Test one area | `cargo test --features dev --lib -- gear::` |
| Test the net and server crates | `cargo test -p abstractions-net -p abstractions-server` |
| The full gate | `mise run check` |
| See what makes a build slow | `mise run timings` |

The `dev` feature links Bevy dynamically. A dev binary only starts through `cargo run --features dev`, not by its path.

## The loop

1. Edit, and let `cargo check` or bacon tell you about errors.
2. Run the tests of the area you touched, with a filter.
3. Run the full gate once, before you commit. Not after every edit: it takes minutes.

A change anywhere in `src/` rebuilds the whole game crate. Batch your edits between builds.

## The gate

All four must pass before a commit:

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

- No borders anywhere. No rounded corners.
- Never bold text. The game font renders badly in bold.
- Interface chrome is black and white. Colour is allowed only in the map picture, player dots and markers.
- Icons come from pixelarticons, through `crate::ui::Icon`. Build screens from the widgets in `crate::ui`.
- What the player sees is real: every light, cloud and landmark is an object at that place. Nothing is decoration.

## Commits

- Conventional Commits: `type(scope): summary`, imperative, lowercase, no trailing period, under 72 characters.
- The scope is the area touched, such as `map`, `gear` or `net`.
- No co-author line and no "Generated with" line.
- One logical change per commit. Update `CHANGELOG.md` and `README.md` when the player would notice the change.

## Several agents at once

- Each agent works in its own git worktree, on its own branch. Never commit on `next` or `main`: the lead merges.
- Each agent sets its own `CARGO_TARGET_DIR`. Two builds in one target directory block each other.
- At most two builds at a time on an 8-core, 16 GB machine. More makes every build slower than running them in turn.
- Do not push. Report the commit hashes, the gate output and what you could not check.

## Releases

A release goes from `next`: the full gate, a dry run of `.github/workflows/release.yml` on a throwaway branch, then `main` is fast-forwarded and the tag `vX.Y.Z` is pushed. The tag builds every platform and the server image.
