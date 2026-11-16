# Abstractions

A first-person exploration game with a pixelated, low-poly look. You wake up in an endless ocean of procedurally generated islands. Cross six biomes, climb mountains, dive on coral reefs, and find villages, camps, ruins and lighthouses. The day lasts ten minutes, and so does the night.

## Controls

| Input | Action |
| --- | --- |
| W A S D | Move |
| Mouse | Look |
| Space | Jump, or swim up |
| Look down + W | Dive while swimming |
| Shift | Sprint |
| Esc | Pause menu, or back |
| Arrows, Enter | Move and select in menus |

Rebind every key in Settings > Controls.

## Worlds

New world asks for a name and a seed. Leave the seed empty for a random world. The same seed always gives the same world.

The game saves your position, view and time of day when you pause, when you quit, and every minute. Continue on the main menu reopens the last world. Worlds lists all of them; press Delete on a world to remove it.

Saves are one small file per world:

- Linux: `~/.local/share/abstractions/worlds/`
- macOS: `~/Library/Application Support/abstractions/worlds/`
- Windows: `%APPDATA%\abstractions\worlds\`

## Run

You need Rust and [mise](https://mise.jdx.dev).

```sh
mise run dev
mise run run
```

`dev` builds fast with dynamic linking. `run` builds an optimized release. The release binary has no assets folder: the shader is embedded.

Other tasks: `mise run test`, `mise run check`, `mise run build-release`.

## macOS

1. Download `Abstractions-<version>-macos-universal.dmg` from the latest release. It runs on Apple Silicon and Intel, macOS 11 or later.
2. Open the .dmg.
3. Drag Abstractions onto the Applications folder.

The app is not notarized. Notarization needs a paid Apple Developer ID, so macOS blocks the first launch. Pick one way to open it:

- macOS 14 and older: right-click Abstractions in Applications, choose Open, then click Open.
- macOS 15 and later: try to open the app once. Then go to System Settings, Privacy & Security, scroll to "Abstractions was blocked" and click Open Anyway.
- Terminal, any version:

```sh
xattr -dr com.apple.quarantine /Applications/Abstractions.app
```

You only need to do this once.

## Ray tracing

Settings > Graphics > Ray tracing (experimental) turns on ray traced lighting: traced shadows from every plant and building, and bounce light. It is off by default and applies at once, without a restart. The line under the settings list shows its real state: on, off, or not available with the reason.

What it needs:

- A GPU with hardware ray tracing.
- On a Mac: an M3 or later and macOS 15 or later. M1 and M2 cannot run it, because the engine's ray tracer needs a GPU limit they do not have.

Limits while it is on: no distance fog on the land, and torches, lamps, campfires and fireflies do not light the ground. The image has a little grain, because the engine's only denoiser is for NVIDIA cards.

It has run on a software renderer but never on a real GPU. If it does not turn on, the reason is in `ray-tracing.log`, next to the settings file. If the game fails to start with it on, start the game again: it turns ray tracing off by itself.

## Settings

Settings live in `settings.toml` inside the `abstractions` folder of your config directory:

- Linux: `~/.config/abstractions/settings.toml`
- macOS: `~/Library/Application Support/abstractions/settings.toml`
- Windows: `%APPDATA%\abstractions\settings.toml`

A missing or broken file is replaced by the defaults.

## Source layout

- `src/lib.rs`: root plugin that wires every module together.
- `src/screens`: the `Screen` state (title, loading, gameplay).
- `src/menus`: main, worlds, new world, pause and settings menus.
- `src/saves`: saved worlds and autosave.
- `src/ui`: shared UI widgets.
- `src/world`: terrain, biomes, chunk streaming, plants, structures and sea.
- `src/player`: first-person movement, jump and swimming.
- `src/render`: low-resolution target, day clock, sun, moon, sky dome, fog and ray tracing.
- `src/effects`: particles, fireflies, butterflies and birds.
- `src/wind.rs`: the wind shared by water, smoke and particles.
- `src/settings`: persisted settings and the code that applies them.
