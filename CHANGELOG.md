# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.10.0] - 2026-11-19

### Added

- Multiplayer: up to 16 players share one universe, with the same seed, clock and discoveries. Each player keeps their own robot, ship and companion, and sees everyone else's move smoothly, with their name above their head.
- Open to LAN: pause a world and open it to your network. Friends on the same network see it under Multiplayer and join with one click, and you keep playing without a reload. Close LAN saves the world as usual.
- The Multiplayer screen lists your servers with their players and ping, and finds games on your network by itself. Add a server by its address, join one once with Direct connect, and give a password when a server asks for one. Space (`space.facile.studio`) is in the list from the start.
- A planet a friend visits shows as visited on your board computer too.
- The game remembers each server's key the first time you join, and refuses to join if it changes.
- In a session the pause menu lists the players and has a Disconnect entry, and the world keeps going behind menus.
- `abstractions-server`: a small server for Linux (x86_64 and arm64) and a Docker image, `ghcr.io/saravenpi/abstractions-server`. It needs no configuration, saves the world every 30 seconds and can start from a single-player world. The guide is at https://abstractions.facile.studio/server/.
- A real cockpit: a seat, a dashboard, side consoles, canopy bars, a stick that moves with your steering and a throttle lever that follows the throttle. Black and white instruments show your speed against the top speed, the warp charge and the gear. Your robot sits in the seat with its hands on the controls, and you can see it through the canopy from outside.
- In the ship's first-person view the mouse turns your head inside the cockpit; you see your arms, legs and the dashboard.

### Changed

- Clouds are real: each planet has one cloud field that drifts with the wind and changes over minutes. The clouds you see from the ground, from orbit and on the far planet are the same clouds, in the same places.
- Weather happens somewhere: it rains or snows only under the thick cloud you can see above you and from orbit, and you climb out of the rain above the cloud tops. Weather changes no longer make the sky flicker.
- Flying into a cloud whites out the view, shakes the ship and sounds like it only inside a real cloud, not anywhere at cloud height.
- Light falls where the star hits the planet: the ground, the sea and far planets are lit by the sun's height at each place, not where you stand. From high up the night side stays dark while you are in sunlight, mountain tops catch the dawn first, and far planets have a dark night side.
- Stars are one real object: near a star you see a disc of its true size, far away it is the same point as in the star field. Its light falls off with distance, and between the stars there is no sun.
- The warp shows faint grey interstellar dust, inside galaxies only, and each star streaks by its real motion: near stars streak past, far ones stay points. The stars no longer sway.
- Choosing a new destination during a warp drops out of warp first, then turns to the new course and spools again.
- The mouse steers the wingsuit and the ship only while you hold the left mouse button. Without it the mouse looks around. With the wings out W and S move the aim like a stick, and A and D bank into a turn that the camera follows.
- The wings open in one smooth 0.4 second motion.

### Fixed

- The wingsuit trails no longer vanish or show through the sea, and they no longer draw a line across the sky after you leave the ship.
- The roof of the home base station, the stall posts of markets and many other drawn parts of buildings and camps were not solid. Everything drawn larger than a hand is now solid. Roofs are one solid piece.
- The ship no longer flies through buildings: it stops at walls and can land on flat roofs.
- The sea seen from a few hundred metres up no longer shows square rings.
- The companion follows a flying player without a jump, and no longer flies through buildings.

## [0.9.0] - 2026-11-18

### Added

- Climate: every place on a planet has its own temperature and rainfall, from its latitude, its height, the winds and the sea. Biomes, snow lines and deserts follow it, so poles are cold, equators warm and the lee of mountains dry.
- Terrain shaped by the planet: rivers and river beds, landforms that follow the climate and the gravity, and craters on worlds without air.
- Weather where you stand: clouds, rain, snow, dust storms, gusts and far thunder, with their sounds. It never rains or snows indoors.
- Seasons on tilted planets: the ground and the snow change through the year. Kodya is now tilted like the Earth, and a Kodya year lasts about 70 hours of play.
- The board computer shows each planet's season, its range of temperatures and a climate map. The weather is on the HUD.
- Self-update: the title screen tells you when a newer version is out, and "Check for updates" in the main menu looks again at any time. The update screen shows the notes and the size, downloads with a progress bar you can cancel, checks the download, installs it and offers to restart. On macOS it downloads the disk image and offers to open it. Settings > General > Check for updates at launch, on by default.
- The way home: the board computer always lists your home galaxy, home star and Kodya, marked with a house. Away from home a Home row stays on top of the lists, the map pins a house with the distance on its rim, the header says how far you are, and the compass points home in space and on other worlds. A "Course Home" button and the Home key set the course home one step at a time.
- The landing gear folds in flight and comes down near the ground. The flight instruments show it: up, moving or down.
- The warp rushes through streaks of dust, and the stars sway as the drive engages.
- Plants grow as thick as the climate lets them, and villages and camps stand only where their people could live.

### Changed

- Press Space again in the air to spread the wingsuit, and once more to fold it. The wings key (R) still works. Hold Shift in a fall to fire the boots and slow the descent; with the wings out, Shift is still the boost.
- The wingsuit glides much further, about four metres forward for one down, and the keys steer it.
- The ship handles more calmly near the ground, and its steering is locked while the warp is engaged: line up during the spool.
- The board computer selects on a click, not when the pointer passes over a row; a double click sets the course. The panel has more room, and the arrow on the maps points where the ship points.
- The companion looks around on its own, head first, instead of copying where you look.
- In third person you can turn the camera to see your face, and the body dives head first under water.
- The home base stands at a mild latitude. A world saved in 0.8.1 on Kodya starts at the new base with a fresh ship; discoveries and play time are kept. Some worlds now start in autumn.
- A new game opens under a calm sky: no storm, rain, snow or dust for the first five minutes on Kodya.
- Rivers and ponds are drawn on near ground only.
- The night sky shows only real moons. A world without a moon, like Kodya, has darker nights lit by the stars.
- Far planets and moons are lit as brightly as the ground.
- Releases also publish a plain Linux binary, a plain Windows `.exe` and SHA-256 checksum files, which the self-update uses.

### Removed

- Ray tracing: the "Ray tracing (experimental)" option, its status line and its log. The game looks as it did with the option off. Old settings files still load.

### Fixed

- A crash no longer bounces the ship back into the air, and the ship no longer turns into the ground.
- Roofs are solid where they are drawn: you can walk on them, and into the fishing shelter and under the shrine's eaves.
- A world saved in 0.8.1 on another planet no longer resumes under the ground.
- Loading a second world in one session no longer adds false first visits to its log.
- Seen from the air or from orbit, a planet no longer changes colour all over with the season under the camera.
- Dust storms no longer blow over meadows, forests or snow.
- Large planets load their distant ground and their climate map far faster.
- A loaded ship keeps its landing gear as it was saved.
- The sea no longer shows a dark patch or pale rings under you when you fly up.
- Lava seas glow from any height and from space, not only close to you.
- Planets and moons keep their real colours and clouds when you fly away from them.
- Shadows fade out as you climb instead of vanishing at once.

## [0.8.1] - 2026-11-18

### Added

- Gas giants: huge worlds of deep cloud in the cold outer reaches of many systems, with bands of cloud and a storm that turns with the planet.
- Moons: gas giants have 2 to 6 and some rocky planets 1 or 2. Each is a small world you can fly to, land on and walk on, and each shows the same face to its planet.
- Tidal heat: the closest moons of the biggest giants can be molten or hold liquid water.
- Rings of ice or dust around 4 in 10 gas giants and a few rocky planets, with the planet's shadow across them.
- Moons in the sky with real phases. Nights on a planet with moons are lit by them, brighter at full moon. A moon passing in front of the star darkens the land behind it.
- The board computer lists each planet's moons under it once you have visited the system, and a gas giant's card gives its cloud deck pressure, its moons and its rings.
- The system map and the minimap draw moons on small orbits around their planet when you zoom in, tick marks for them when zoomed out, and a line for rings. Click a moon to set a course to it.
- The autopilot can take you to a moon and stops beside it, clear of its planet.
- Gas giants can be flown into down to the cloud deck, where the flight assist holds you. Below the deck the air thickens, shakes the hull and pushes the ship back up.
- Gas giants have a deep, low wind.

### Changed

- The outermost planet of the home system is always a gas giant with moons to visit.
- Fewer ice worlds in outer systems: many of those planets formed as gas giants instead.
- You cannot land on a gas giant or step out of the ship over one. Over a giant the altitude reads height above the cloud deck.
- A save made on a planet that is now a gas giant starts again at the home base with a new ship berthed there.

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
