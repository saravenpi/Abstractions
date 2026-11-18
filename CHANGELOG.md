# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.8.0] - 2026-11-18

### Added

- Every planet has its own climate, worked out from its star and its orbit: starlight, air pressure, temperature, water and axial tilt. Stars have the surface temperature of their class and a habitable zone.
- Six kinds of planet: ocean, island, desert, ice, lava and barren. Each has its own ground, sea level, life, sky and sound.
- Ocean worlds keep only small islands. Dry worlds have no sea, and you walk on the old sea bed. Lava worlds have a glowing sea of molten rock.
- Each planet has its own sky: butterscotch over deserts, smoky red over lava, pale over ice. Thin air gives a darker sky with stars by day; no air gives a black sky and no fog; thick air gives haze and a red, dim low sun. Cloud cover depends on the kind of planet.
- From space each planet shows its kind, and lava worlds glow on their night side.
- The ship and the wingsuit fly by the air of each planet. Without air there is no drag and no lift, and the boots are your only brake.
- Wind, waves and the music of a planet follow its air, its sea and its kind.
- The board computer shows each planet's kind and temperature, and a card with its air, water, light, habitable zone, tilt, size, gravity, day and year.
- The map in the board computer is interactive: point at a mark to select it, click twice to set the course, drag to pan, wheel to zoom, 0 to see it all. Zooming out of the system opens the galaxy. It draws orbits, the course with its distance, a scale bar, and names that never overlap.
- Type letters in the board computer to filter the list; F2 sorts by distance or name. The course line shows a rough time to arrival.
- A minimap in the top right corner: the ground around you on a planet, turning as you turn, and your star system in space. M steps through its ranges or turns it off, and the key can be changed in the settings.
- The home system always has at least five planets.
- Each download on the release page also exists under a name that says its version.

### Changed

- The board computer is a panel on the right side. The game keeps running beside it.
- With flight assist on, the throttle sets a speed and the ship flies where its nose points. At idle the ship stops by itself, in space and in the air, and hovers still and level at any height. Top speed grows with the distance from the ground. Turn the assist off to fall, coast and orbit as before.
- With flight assist on, level flight follows the curve of the planet, the ship slows down before it meets the ground, and a landed ship takes off only with the rise key.
- Warp plays as one sequence on the real stars: they crowd toward where you fly, turn blue ahead and red behind, stretch into streaks, and settle into the sky of where you arrive.
- The world loads ahead of you when you fly fast, and ground that still has to change in view fades in instead of popping. Above 120 m/s grass and the finest ground detail wait until you slow down.
- Menus sit on frosted glass instead of a darkened picture of the game.
- The compass marks are smaller.
- Kodya orbits where its star gives it the light of a temperate world. The other planets of the home system have moved; a ship left in open space there is no longer beside its planet.
- Planets other than Kodya look different now, including places you already visited.

### Fixed

- Holding back or forward on the wingsuit always takes you all the way round: a loop with enough speed, a stall turn from a slow glide. The robot's body and the camera follow without a snap.
- The ship no longer drifts or slides while it hovers with the engines idle.
- The wings, tail fin, canopy and landing gear are joined to the hull.
- The boot flame no longer lays a see-through orange slab over the robot and the ground. Its trail is a tapering line, only in fast flight.
- On dry worlds the low ground is ground: footsteps, no swimming, no underwater fog, dust instead of spray, and the ship parks on it.

## [0.7.1] - 2026-11-18

### Changed

- In the ship and on the wingsuit, A and D roll and Q and E turn. Saved bindings for these four keys are reset.
- The wingsuit flies whole loops: hold S and the body goes over the top and round. The view follows it.
- In space the flight assist makes the ship go where it points under thrust, instead of sliding on its old course. Ctrl held at idle throttle brakes. With the engine idle the ship coasts as before, so an orbit stays an orbit.
- The flight assist stops and starts a turn of the ship faster.

## [0.7.0] - 2026-11-18

### Added

- A compass at the top of the screen: one mark for the home base and one for the ship, each with its distance, while you are on their planet.
- A map in the board computer, beside the table: the system with its orbits, the known stars of the galaxy and the known galaxies, with the ship, the selected row and the destination marked.

### Changed

- The ship is steered with the keys: W and S move the nose, A and D turn, Q and E roll, Shift and Ctrl move the throttle, Space and C fire the vertical thrusters. The mouse only looks around, and the view comes back to the nose by itself. Saved ship bindings are reset to these keys.
- The ship no longer has reverse thrust.
- The boots fire when you press jump again in the air, not on the jump itself.
- W, A, S and D steer the wingsuit: dive, climb and turn.
- Sunlight, sky, clouds and ambient light take their colour and strength from the real height of the star and from its own colour, not from the hour.
- The companion picks its own place beside you and keeps it when you turn your head. It moves when you leave it behind your back, when it is in your way, or when it feels like it.
- The companion speaks for real flights only, not for every jump, and waits between two chirps. Its voice comes from where it floats.
- Waves are heard near a shoreline only, from the side the shoreline is on.
- The boots hum lower and softer.
- The warp streaks start as points along the nose of the ship and stretch as the drive engages, and the stars dim behind them.
- A ship that drops out of warp stays at rest beside its destination.
- The board computer closes when you engage the autopilot.

### Fixed

- The first person view looked down into the neck of the robot. The eyes now sit on the face and swing forward when you look down.
- A new game could start under the ground: play began before the ground under the home base was loaded.
- A fast dive in the ship turned the whole view white.
- A body that sinks under the ground is put back on it.
- The Ship tab of the settings had no icon.

## [0.6.0] - 2026-11-17

### Added

- A universe: galaxies, spiral arms, star classes and solar systems with 2 to 8 planets, all from one seed.
- Planets are true spheres, 3 to 40 km in radius, with their own gravity, day length and orbit. Every planet uses the island generator for now.
- The home base on Kodya, in the Vereya system: the control container, a small station and a landing pad.
- A ship with Newtonian flight, flight assist, landing gear, a warp drive and an autopilot.
- A board computer that lists the systems and galaxies you have discovered and flies you there. Visiting a place reveals its neighbours.
- A sky made of real things: the sun is the star, the atmosphere fades to black, a cloud layer to fly through, stars and galaxies in their true directions.
- A humanoid robot body with a screen face, and first and third person views on foot and in the ship.
- Boot boosters that slow a fall, a wingsuit that glides, and powered flight with both. Wind and trails in flight.
- A floating robot companion with a screen face that follows you, reacts and lights the night a little.
- Sound: ambient music generated for each planet and silent in space, wind, sea, footsteps, ship and interface sounds.
- World management: rename, duplicate and delete.
- Render distance steps up to Extreme for strong GPUs.
- An app icon on Windows, macOS and Linux.
- The MIT licence and a roadmap.

### Changed

- The interface is black and white, with the Undefined font, pixelarticons and blurred backdrops.
- Terrain, plants, buildings and colliders are built off the main thread and arrive within a per-frame budget.
- Ray tracing now starts on Apple M1 and M2, with a lighter workload.
- A world from 0.5 still opens. It keeps its name and seed and starts again at the home base on Kodya.

### Fixed

- Door frames flickered against the wall they sat in.
- Roofs floated above the walls inside houses.
- Far water showed concentric rings.
- The field of view value wrapped onto two lines in the options.

## [0.5.0] - 2026-11-17

### Added

- House interiors: tables on legs, beds, cupboards, chests, rugs, a fireplace under every chimney and a candle or lantern in every house. The smithy, barn, tower and hall get their own furniture.
- Flames made of three glowing layers that sway with the wind, embers above campfires and braziers, and real lanterns on the lamp posts.
- With ray tracing on, torches, lamps and fires now light their surroundings.

### Changed

- The plaza cross is replaced by a stone sundial.
- Nights are darker, so torches, lamps and fires cast visible pools of light. Lights fade in and out instead of popping.
- Shore foam follows the distance to the shore: a narrow band that breathes slowly and thin lines that drift in. Waves die out in shallow water and ponds stay calm.
- The weeping willow is redrawn: a taller leaning trunk, a wide crown and hanging curtains of mixed lengths.
- Trees and rocks keep a minimum distance from each other.
- Camps are less frequent, so other structures show up more often.
- Colors are dithered before they are reduced, which removes the hard rings around lights at night.

### Fixed

- Floating pieces in villages: benches and stones around the plaza tree, crop rows, hedges and fences, potted palms and door lanterns, roads and the plaza floor.
- Trees, rocks and grass now stand on the visible ground, with trunks and rocks sunk into slopes.
- Footpaths no longer run through houses or fields, and plants no longer grow on them.
- Shore foam raced across shallow beaches and shifted when the camera moved.
- Firefly lights slid across the ground from one firefly to another.
- Water noise broke into flickering blocks far from the world origin.

## [0.4.0] - 2026-11-16

### Added

- Villages: a plaza with a well or fountain, bending streets, 6 to 14 buildings from ten house kinds, lamp posts, wheat fields and footpaths, styled per biome.
- Camps in five layouts, common enough to find one every few hundred metres.
- Torches, lamps, campfires and braziers that give light and flicker, with smoke rising from fires.
- Underwater world: a sandy shelf, coral reefs, kelp forests, seamounts, trenches down to about 40 m, and a few wrecks.
- Ponds with water lilies, lotus, cattails and weeping willows.
- New plants: birch, great oak, dead trees, bamboo, berry bushes, tall grass and more.
- Water shader: wind-driven waves, sky reflection, sun glitter and foam where waves meet the shore.
- Wind that drives the waves, the smoke and the particles.
- Particles: falling leaves, cherry petals, snow, desert dust, bubbles and plankton.
- Creatures: fireflies that light their surroundings at night, butterflies and flocks of birds by day.

### Changed

- The 3D view renders at the low resolution shown on screen instead of the full window resolution. On a high-density display this shades about 36 times fewer pixels.
- Terrain uses three levels of detail by distance. Far grass is hidden.
- Islands have bays, headlands, islet chains and cliffs. Biomes form larger regions with sensible neighbours.
- The spawn point prefers a gentle coast on a large island.
- Ray tracing applies without a restart, shows its real state in the settings and writes a diagnostic log. It now uses the engine's noise reduction.

### Fixed

- Ray tracing could quit the game at startup on M1 and M2 Macs, then switch itself off. The game now checks the GPU limits first and says why it is unavailable.
- Logs, trunks, cactus arms and other cut shapes were see-through at the ends.
- Floors, docks and awnings were invisible from below. Tent doors and ramps were one-sided.

## [0.3.0] - 2026-11-16

### Added

- Ray tracing as an experimental graphics setting, off by default. It needs a GPU with hardware ray tracing and a restart to apply.
- A startup guard: if the game fails to start with ray tracing on, the next launch turns it off.

### Changed

- World colors come from a palette texture instead of vertex colors, which ray tracing requires.

### Known limits

- Ray tracing has never run on real hardware. With it on, distance fog and underwater fog do not apply to the land, and nights may be darker.

## [0.2.0] - 2026-11-16

### Added

- Six biomes: meadow, forest, cherry grove, desert, taiga and tropical, each with its own ground colors.
- Mountains up to about 100 m, plateaus, irregular coastlines and islands of very different sizes.
- Biome plants: oaks, pines, cherry trees, palms, cacti, ferns, mushrooms, reeds and flowers in several colors.
- Eleven kinds of structures to find, such as stone circles, ruined towers, cabins, lighthouses, pyramids, docks and shrines.
- Day and night cycle of 20 minutes with a sun, a moon, stars and drifting clouds.
- Underwater fog and tint.
- Saved worlds: named worlds, a worlds list, Continue, and autosave of position, view and time of day.
- Graphics settings for ambient occlusion, bloom, clouds, color depth, field of view and an FPS counter.
- Keyboard navigation in every menu. Escape goes back.
- macOS release as a universal app in a `.dmg`.

### Changed

- Menus redesigned: left-aligned text, one accent color, no boxes.
- Swimming: you float with your eyes above the water, dive by looking down, and climb out at the shore.
- Each chunk is now at most 4 entities instead of up to 48, with colliders only near the player.
- Chunks appear at most two per frame while you move, to avoid stutter.

### Fixed

- Menus stayed painted over the world after closing. This also hid the world until the window was resized.
- The player slid down slopes while standing still.
- Walking uphill lost speed and bounced; walking downhill launched the player off the ground.
- The camera sat under the water surface while floating.

## [0.1.1] - 2026-11-16

### Fixed

- macOS release builds run on the `macos-latest` runner. The `macos-14` image is no longer available.

## [0.1.0] - 2026-11-16

### Added

- Procedural islands in an endless ocean, streamed in chunks around the player.
- Translucent sea between the islands.
- First-person movement with sprint.
- Jumping and swimming.
- Title, new game (seed), pause and settings menus, with key rebinding.
- Persistent settings file for graphics, controls and audio.
- Pixelate post-process shader with color banding, embedded in the binary.
- Warm sun with cascaded shadows, soft sky and distance fog.
- CI and tagged release builds for Linux, Windows and macOS.
