# Abstractions

A space exploration game with a pixelated, low-poly look. You are a human in a control container on the planet Kodya, in the Vereya system, driving a small robot over a quantum link. A second robot floats along with you. Walk around whole planets, glide over them in a wingsuit, then take the ship through the clouds to other planets, stars and galaxies. Everything comes from one seed.

Where the game is going: [ROADMAP.md](ROADMAP.md).

## Controls

On foot:

| Input | Action |
| --- | --- |
| W A S D | Move |
| Mouse | Look |
| Space | Jump, or swim up. Pressed again in the air and held: boot boosters, which slow a fall |
| Shift | Sprint. Held with the wingsuit spread: boot boosters, which make you fly |
| R | Spread or fold the wingsuit in the air |
| W S in the wingsuit | Dive and climb. Hold S to fly a loop |
| A D in the wingsuit | Roll into a banked turn |
| Q E in the wingsuit | Turn |
| V | First or third person |
| F | Board the ship |
| Look down + W | Dive while swimming |
| Esc | Pause menu, or back |

In the ship:

| Input | Action |
| --- | --- |
| W S | Nose down, nose up |
| A D | Roll |
| Q E | Turn left, turn right |
| Shift, Ctrl | Throttle up, throttle down. Shift held at full throttle boosts. Ctrl held at idle brakes in space |
| Space, C | Vertical thrusters up and down |
| Mouse | Look around. The view comes back to the nose by itself |
| X | Cut the throttle |
| Z | Flight assist on or off |
| J | Warp drive, away from a planet's gravity |
| Tab | Board computer: places you have discovered |
| G | Send the computer's destination to the autopilot |
| V | First or third person |
| F | Leave the ship |

A bar at the top of the screen points to the home base and to the ship while you are on their planet.

Rebind every key in Settings > Controls.

## Worlds

New world asks for a name and a seed. Leave the seed empty for a random world. The same seed always gives the same world.

The game saves where you are, your ship, what you have discovered and the time when you pause, when you quit, and every minute. Continue on the main menu reopens the last world. Worlds lists all of them, where you can rename, duplicate or delete a world.

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

## Install

Every release publishes the same three file names, so these links always fetch the latest version:

- Linux, x86_64: https://github.com/saravenpi/Abstractions/releases/latest/download/abstractions-linux-x86_64.tar.gz
- Windows, x86_64: https://github.com/saravenpi/Abstractions/releases/latest/download/abstractions-windows-x86_64.zip
- macOS, Apple Silicon and Intel: https://github.com/saravenpi/Abstractions/releases/latest/download/Abstractions-macos-universal.dmg

The version is in the folder name inside each archive, and in the app's Get Info window on macOS.

### Linux

Extract the archive and run `./abstractions` from its folder.

A tarball cannot register a launcher entry or an icon. To add them, copy the binary onto your `PATH` and the `share` folder into your user data folder:

```sh
install -Dm755 abstractions ~/.local/bin/abstractions
cp -R share ~/.local/
```

That installs `~/.local/share/applications/abstractions.desktop` and the icons under `~/.local/share/icons/hicolor/`. The desktop entry runs `abstractions`, so `~/.local/bin` must be on your `PATH`.

### Windows

Extract the zip and run `abstractions.exe`.

### macOS

1. Download `Abstractions-macos-universal.dmg`. It runs on Apple Silicon and Intel, macOS 11 or later.
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
- On a Mac: Apple Silicon and macOS 15 or later. M1 and M2 run a lighter version.

Limits while it is on: no distance fog on the land, and torches, lamps, campfires and fireflies do not light the ground. The image has a little grain, because the engine's only denoiser is for NVIDIA cards.

It has run on a software renderer but not yet on a real GPU, and the M1 and M2 path is new in 0.6. If it does not turn on, the reason is in `ray-tracing.log`, next to the settings file. If the game fails to start with it on, start the game again: it turns ray tracing off by itself.

## Settings

Settings live in `settings.toml` inside the `abstractions` folder of your config directory:

- Linux: `~/.config/abstractions/settings.toml`
- macOS: `~/Library/Application Support/abstractions/settings.toml`
- Windows: `%APPDATA%\abstractions\settings.toml`

A missing or broken file is replaced by the defaults.

## Source layout

- `src/lib.rs`: root plugin that wires every module together.
- `src/universe`: galaxies, stars, systems and planets generated from the seed.
- `src/space`: frames of reference, the floating origin and the clock.
- `src/world`: planets as spheres, terrain, chunk streaming, plants, structures and sea.
- `src/home`: the base on Kodya.
- `src/player`, `src/avatar`, `src/gear`, `src/view`: movement, the robot body, the wingsuit and boots, the camera.
- `src/ship`, `src/computer`: the ship, its autopilot and the board computer.
- `src/companion`: the second robot.
- `src/audio`: generated music and sounds.
- `src/render`: low-resolution target, sun, sky, atmosphere, clouds, stars, fog and ray tracing.
- `src/effects`, `src/wind`: particles, creatures and the shared wind.
- `src/screens`, `src/menus`, `src/ui`: states, menus and widgets.
- `src/saves`, `src/settings`: saved worlds and settings.
- `vendor/bevy_solari`: the engine's ray tracer with two patches, listed in `vendor/bevy_solari/PATCHES.md`.

## Licence

MIT. See [LICENSE](LICENSE). Font licences are in `LICENSES/`.
