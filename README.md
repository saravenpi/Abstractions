# Abstractions

A space exploration game in a gigantic universe where every world is its own. You are a human in a control container on your home planet, driving a small robot over a quantum link. A second robot floats along with you. Walk around whole planets at their real size, glide over them in a wingsuit, then take the ship through the clouds to other planets, stars and galaxies. Planets that live have plants, herds and hunters you can meet, track and hunt. Everything comes from one seed.

Where the game is going: [ROADMAP.md](ROADMAP.md).

## Controls

On foot:

| Input | Action |
| --- | --- |
| W A S D | Move |
| Mouse | Look |
| Space | Jump, or swim up. Pressed again in the air: spread the wingsuit, and once more to fold it. Off a cliff or in a fall, one press is enough |
| Shift | Sprint. Pressed in the air: boot boosters, which slow a fall with the wingsuit folded and make you fly with it spread. A sprint held over an edge does not fire them: press Shift again |
| R | Spread or fold the wingsuit in the air, as Space pressed again does. Folded, you fall belly down in a skydiver arch |
| W S in the wingsuit | Dive and climb. Hold S to loop with the boosters on. A plain glide is too slow and stalls out of the climb. Let go and the wings fly on where you pointed them, easing back to the glide over a few seconds. Rolled on edge, S turns you round in a wide drift |
| A D in the wingsuit | Roll. Hold to roll over and on round. Let go and the wings level |
| Q E in the wingsuit | Turn |
| Left mouse held in the wingsuit | Steer toward where you look |
| Left Ctrl | Crouch: slower, quieter, and animals notice you less |
| V | First or third person |
| M | Map |
| N | Minimap range, then off |
| B | Drop a pin where you stand, or remove the pin within 10 m. On the map, a pin under the pointer. You keep up to eight |
| K | Field guide: the species of the planet underfoot |
| I | Pack and workshop: what you carry on the left, the tools you can make for an intent on the right. The robot wakes with a stone spear in slot 1 and a stone knife in slot 2, empty-handed. Tab moves between the two. X puts the chosen tool or stack down on the ground, Shift X half the stack |
| 1 to 6 | Hold the tool in that hotbar slot; the key of the held slot stows or draws it |
| Left mouse, right mouse | Swing the held tool or stab with a spear, guard |
| T held | Draw the spear in hand back; let go to throw it. A thrown spear is marked on the compass, the map and over the world until you pick it up |
| H | Break a piece off the rock you look at |
| U | Patch the worst hurt part of the robot with what you carry. Near the base every part mends itself |
| F | Board the ship. Cut a part off the carcass you look at, pull out a stuck spear, or take the tool or stone you look at off the ground |
| Look down + W | Dive while swimming |
| F1 | Hide the interface, and show it again: a clean view for a screenshot. It works in the ship too |
| Esc | Pause menu, or back |

In the ship:

| Input | Action |
| --- | --- |
| W S | Nose down, nose up |
| A D | Roll |
| Q E | Turn left, turn right |
| Shift, Ctrl | Throttle up, throttle down. Shift held at full throttle boosts. Ctrl held at idle brakes in space |
| Space, C | Vertical thrusters up and down |
| Mouse | Look around. With "Camera auto-follow" on, the view comes back to the nose by itself; off, it stays where you leave it |
| R | Bring the view back to the nose |
| X | Cut the throttle |
| Z | Flight assist on or off |
| J | Warp drive, away from a planet's gravity. Above the air of a planet, where the warp drive is locked, it starts and stops the orbital cruise: a fast flight around the planet at the height you hold. Shift and Ctrl set its speed, Q and E turn, Space and C change the height |
| M or Tab | Map. Wheel to zoom, drag to pan, double click to go in, Back or Backspace to go out. A trackpad scroll and a pinch also zoom; W A S D, and the arrows with the pointer on the map, also pan. 0 recentres. Click selects. The card of the selected place gives its chance of life, or what lives there once you have landed, and its Go button sets the course there and engages the autopilot. Enter also presses Go, and right click opens a menu, on the map or on a search result, pin or player in the side panel. The path under the Back button jumps to any level above. Esc lets go of the selection, then closes the map. Select your home planet and press Go ("Fly home") to fly home: above it the ship turns onto the course, cruises to the base and glides down into the air, handing back to you a few kilometres short of it for you to land. Pitch up or press J to take the glide back. In the air far from the base it climbs out first; from farther away it sets the course home and cruises on arrival |
| G | Go: engage the autopilot on the course. The line at the bottom of the screen says when it cannot, and why |
| V | First or third person |
| F | Leave the ship |

The compass at the top of the screen shows north, the home base, the ship, your friends, your pins and the herds you have seen ahead of you, as icons on one row. Names and distances are on the map and on the markers over the world. They fade toward its ends and hide behind you. Pins, the course, home, your ship and your friends also show over the world at their real place, and slide to the screen edge with an arrow when out of view. A marker shrinks, then hides, when you are close to what it marks. Home shows from orbit too, with its distance over the ground; when the planet stands between you and the base its marker stays where the base is, faint, and says so. The HUD shows only what means something where you are: on foot, in the ship, in orbit, in open space or in warp.

The map shows each planet flat and in colour, but only the ground someone has explored. On a server, ground one player uncovers is uncovered for everyone. Right click a place on the map to drop a pin there, copy its coordinates or centre the map on it. Right click a planet or a pin to set the course to it or focus the map on it.

Rebind every key in Settings > Controls.

## Worlds

New world asks for a name and a seed. Leave the seed empty for a random world. The same seed always gives the same world.

The game saves where you are, your ship, what you have discovered and the time when you pause, when you quit, and every minute. Continue on the main menu reopens the last world. Worlds lists all of them, where you can rename, duplicate or delete a world.

A world records the version of the game that made it. A world made with 0.10 or earlier lists as "Made with 0.10" and does not open in 0.11. Play on it downloads the 0.10.1 release and opens the world with it (Linux and Windows).

Saves are one small file per world:

- Linux: `~/.local/share/abstractions/worlds/`
- macOS: `~/Library/Application Support/abstractions/worlds/`
- Windows: `%APPDATA%\abstractions\worlds\`

## Play together

Players share one universe: the same seed, the same clock and the same discoveries. A server takes 16 players by default, and up to 1000 when its owner says so. Each player keeps their own robot, ship and companion. What one player puts on the ground lies there for everyone until someone takes it, and the parts of a carcass go to whoever cuts them first.

- **On your network:** in a world, pause and pick Open to LAN. Friends on the same network see the world under Multiplayer, On your network, and join with one click. Close LAN saves the world as usual.
- **On a server:** Multiplayer lists your servers with their players and ping. Space (`space.facile.studio`) is there from the start. Add a server by its address, such as `play.example.com` or `203.0.113.4:24650`. Direct connect joins one without saving it.

The first time you join a server, the game remembers its key, as SSH does, and shows it in a banner. If the key changes later, the game refuses to join and says so. Known keys live next to the settings file, in `known_servers.toml`.

A world on a server is saved by the server. The game writes nothing to your local worlds while you play there.

## Host a server

The server is one small program for Linux, x86_64 or arm64 (a Raspberry Pi works). The guide at https://abstractions.facile.studio/server/ walks through every step. In short:

```sh
curl -fsSL https://abstractions.facile.studio/server.sh | sh
abstractions-server
```

Or with Docker:

```sh
docker run -d --name abstractions -p 24650:24650/udp -v abstractions:/data \
  --restart unless-stopped ghcr.io/saravenpi/abstractions-server
```

The first start creates a world with a random seed in `./data` (`/data` in Docker) and prints the address and the key to give your friends. For players outside your network, forward UDP port 24650 to the machine, or point a domain at it. With Cloudflare, the record must be DNS only: its proxy does not carry UDP.

| Flag | Variable | Default | What it does |
| --- | --- | --- | --- |
| `--name` | `ABSTRACTIONS_NAME` | the world's name | Name in the server list |
| `--password` | `ABSTRACTIONS_PASSWORD` | none | Password players must give |
| `--port` | `ABSTRACTIONS_PORT` | 24650 | UDP port |
| `--max-players` | `ABSTRACTIONS_MAX_PLAYERS` | 16 | Players at once, 1 to 1000 |
| `--data` | `ABSTRACTIONS_DATA` | `./data` | Folder of the world and the key |
| `--import` | | | Start from a single-player world file |

`abstractions-server ping <address>` asks a server for its name, players, version and key, and exits 0 when it answers. The world is saved every 30 seconds and on shutdown, with the last 3 copies kept. Players need the same game version as the server: a 0.10 game cannot join a 0.11 server. A server that finds a 0.10 `world.toml` keeps it as `world.toml.0.10.bak` and starts a new world.

To build the image yourself: `docker build -f crates/server/Dockerfile -t abstractions-server .` from the repository root.

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

## Updates

The game looks for a newer release once per launch, when it reaches the title screen. If there is one, the main menu shows a line such as "Version 0.9.1 is available" and an "Update" entry. Nothing is downloaded until you choose it. "Check for updates" in the main menu runs the check again at any time. To stop the check at launch, turn off Settings > General > Check for updates at launch.

The check is one HTTPS request to GitHub's public releases API. It sends no identifier: only the user agent `abstractions-updater`.

What "Download and install" does:

- Linux and Windows: it downloads the plain binary of the release next to the game, checks its size and its SHA-256, then puts it in place of the running game and offers to restart. You can cancel until the swap. A failure leaves the installed game untouched.
- macOS: the game does not rewrite its own application. It downloads the disk image to your Downloads folder, with the same checks, and offers to open it. Drag Abstractions to Applications as for a first install. If macOS blocks the first launch again, repeat the quarantine step above.
- If the game cannot write to its own folder (a package manager install, a read-only folder), it says so and offers to open https://abstractions.facile.studio instead.

The update replaces the binary only. The `share` folder of the Linux archive (launcher entry and icons) is not touched.

What the checks prove: the SHA-256 comes from a `SHA256SUMS` file of the same release, so it proves the download is whole, not truncated or corrupted. It does not prove who built it. The builds are not signed, so authenticity rests on HTTPS to GitHub alone. The game downloads only from `https://github.com/saravenpi/Abstractions/releases/download/` and follows redirects only to GitHub's own hosts.

### Go back to the previous version

After an update the previous game stays next to the new one as `abstractions.old` (`abstractions.old.exe` on Windows). The new version deletes it once it has started. If the new version does not start, restore the old one by hand:

```sh
mv abstractions.old abstractions
```

On Windows, delete `abstractions.exe` and rename `abstractions.old.exe` to `abstractions.exe`.

## Settings

Settings live in `settings.toml` inside the `abstractions` folder of your config directory:

- Linux: `~/.config/abstractions/settings.toml`
- macOS: `~/Library/Application Support/abstractions/settings.toml`
- Windows: `%APPDATA%\abstractions\settings.toml`

A missing or broken file is replaced by the defaults.

## Source layout

- `src/lib.rs`: root plugin that wires every module together.
- `src/universe`: galaxies, stars, systems and planets, at real size, generated from the seed.
- `src/life`, `src/fauna`: whether a planet lives, its species and food web, and the herds, hunts and calls drawn around you.
- `src/craft`, `src/condition`: the pack, tools and the workshop, and the health of each part of the robot.
- `src/space`: frames of reference, the floating origin and the clock.
- `src/world`: planets as spheres, terrain, chunk streaming, plants and sea.
- `src/home`: the base on the home planet.
- `src/player`, `src/avatar`, `src/gear`, `src/view`: movement, the robot body, the wingsuit and boots, the camera.
- `src/ship`, `src/computer`: the ship, its autopilot, its course and the charts.
- `src/map`: the map, the explored ground, waypoints and favourite places.
- `src/companion`: the second robot.
- `src/audio`: generated music and sounds.
- `src/render`: low-resolution target, sun, sky, atmosphere, clouds, stars and fog.
- `src/effects`, `src/wind`, `src/weather`, `src/speed`: particles, the shared wind, clouds and weather, and what speed does to the picture and the sound.
- `src/screens`, `src/menus`, `src/ui`: states, menus and widgets.
- `src/saves`, `src/settings`: saved worlds and settings.
- `src/update`: the self-update: the check, the download, its checks and the swap.
- `src/net`: multiplayer in the game: sessions, the server list, Open to LAN, and the robots, ships and companions of other players.
- `crates/net`: the multiplayer protocol and the server's logic, without Bevy.
- `crates/server`: the `abstractions-server` program and its Docker image.

## Licence

The source is open to read, and the game is not for sale by anyone.

Abstractions is under the [PolyForm Noncommercial License 1.0.0](LICENSE). You may read the code, play, run a server, change the game and share your changes, as long as none of it is for commercial purposes. You may not sell the game, a copy of it or a game built from it.

Versions 0.10.1 and earlier were published under the MIT License and stay available under it. Font licences are in `LICENSES/`.
