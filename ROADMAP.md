# Roadmap

Abstractions is growing from an island game into a universe you explore as a small robot, with a second robot for company. This file says what each version adds and in what order. The website shows it at https://abstractions.facile.studio/roadmap/.

Every version below has a status: **released**, **in progress** or **planned**. Planned versions are plans, not promises. Order and scope can change.

The order from 0.12 on was set on 2026-11-20 from a study of what other exploration games did well and badly. Each planned version names the lesson it rests on, and the sources are at the end.

## At a glance

| Version | Name | Status |
| --- | --- | --- |
| 0.5 | Islands | released |
| 0.6 | The universe | released |
| 0.7 | Flight and feel | released |
| 0.8 | Physical planets | released |
| 0.9 | Climate and terrain | released |
| 0.10 | Multiplayer | released |
| 0.11 | Living worlds at real size | released |
| 0.12 | Places worth the trip | planned |
| 0.13 | Ground you can shape | planned |
| 0.14 | Real space | planned |
| 0.15 | Together | planned |
| 0.16 | The thread | planned |
| 0.17 | Hunting, mining, crafting and building | planned |
| 0.18 | Life odds | planned |
| 0.19 | Sapient species | planned |
| 0.20 | Religion and history | planned |
| 0.21 | Contact | planned |
| 0.22 | Economy | planned |
| 0.23 | Geopolitics and conflict | planned |

## The setting

You are a human who never leaves a control container on your home planet. From there you drive a robot over an instant quantum link. A smaller floating robot travels with you. Everything you see is generated from one seed.

## Rules that hold in every version

How the universe is made:

- Every object's seed comes from its parent's seed and its index. The same universe seed always gives the same universe.
- Nothing is stored except what you changed or discovered.
- Every generator is a pure function with a test that the same seed gives the same result.
- What you see is real. Every light, cloud, creature and landmark is an object that exists at that place, made by physical rules. Nothing is decoration.
- Values come from physics, not from tuning: gravity, day length, air and sizes follow from the planet and its star.
- A level of detail only exists near you. Far things are cheap.
- Only generate what shows up in the world. A trait nobody can see or hear is not generated.
- The game stays smooth. Heavy work runs off the main thread and arrives within a per-frame budget, and every feature lands with a cost test.

How it plays:

- **There is always something in sight worth going to.** You choose where to go because you saw it, not because a marker told you.
- **The trip is part of the game.** Nothing skips the surface of a planet. Walking, gliding and low flight are things you want to do.
- **A difference you can see changes what you can do.** A new planet, creature or material is not only a new look.
- **Size follows speed.** How far apart things are is set from how fast you move there.
- **The companion remembers for you.** Keeping track of what you learned is never the hard part.
- **Friends progress together.** Nothing separates players by power, and a newcomer is useful on the first evening.
- **A few rare things are truly rare.** Most places are quiet so that a find means something.
- **Every version is judged by play.** It ships when the people who play it still wander for hours.

## 0.5: islands

**Status:** released

An endless seeded archipelago on a flat world: six biomes, villages with interiors, ruins and camps, a sea with reefs and kelp, a day and night cycle, birds, butterflies and fireflies.

## 0.6: the universe

**Status:** released

The foundation for everything after it.

- Galaxies on a grid, spiral arms, star classes, solar systems with 2 to 8 planets, generated names.
- True sphere planets you can walk around, with their own gravity, day length and orbit, and the island generator sampled in 3D so a planet has no seams.
- Kodya and the Vereya system: you wake up next to the control container, a small station and a landing pad.
- A ship with Newtonian flight, landing gear, flight assist, working orbits, a warp drive and an autopilot.
- A sky lit by the real star, an atmosphere that fades to black, clouds to fly through, stars and galaxies in their true directions.
- A board computer with the systems and galaxies you discovered, a humanoid robot with a screen face, boot boosters and a wingsuit, a floating companion, generated music and sound, and a black and white interface.

## 0.7: flight and feel

**Status:** released

- A ship steered with the keys, and a flight assist that makes it go where it points.
- Roll and loops on the wingsuit.
- A compass and a map.
- Light derived from the real star.

## 0.8: physical planets

**Status:** released

Planets get a kind, and the kind comes from physics.

- A planet record: starlight, air pressure, albedo, greenhouse warming, temperature, water and axial tilt, from the star, the orbit and the seed.
- The habitable zone from the published fit by stellar temperature.
- Six solid classes: ocean, island, desert, ice, lava and barren, each with its own ground, sea level, sky, far view, flight and sound.
- The board computer as a panel beside the running game, with an interactive map and a card per planet, and a minimap.
- 0.8.1: gas giants, moons you can land on, rings, and tidal heating.

## 0.9: climate and terrain

**Status:** released

- A climate field from the planet record: temperature by latitude, altitude and season; moisture from the sea, the wind bands and the slopes.
- Biomes by ordered rules over temperature and moisture.
- Relief from physics, landforms by class (dunes, craters, glaciers, lava flows), rivers traced downhill, weather from the same field, and seasons you can see.
- The structure for multiplayer: the world is the seed plus a log of changes, players act through intents, one clock, a fixed simulation step, stable ids and seeded randomness.

## 0.10: multiplayer

**Status:** released

- Players share one universe, as many as the server allows, with the same seed, clock and discoveries, each with their own robot, ship and companion.
- Open to LAN, a server list with ping, direct connect, passwords, and server keys remembered on first join.
- `abstractions-server` for Linux and Docker, and a public server at space.facile.studio.
- A real cockpit with a seat, dashboard and moving controls.
- Real clouds and weather: one cloud field per planet, seen the same from the ground and from orbit.
- 0.10.1: multiplayer polish, the map with explored ground, waypoints and favourites, the glider rework, step-up, and wing collisions.

## 0.11: living worlds at real size

**Status:** released

Planets and the galaxy at their real size, each living planet with life that belongs to it.

- Real scale: planets, stars, orbits and galaxies at their true sizes, gravity from mass and radius (9.81 m/s² on an Earth-like planet), days from each planet's own spin, and warp speeds to match.
- Continents from tectonic plates, mountain ranges where plates meet, and rivers that drain each planet; real air with its own gases and scale height, and moons that form the way real moons do.
- Home is chosen by one rule: a planet where life climbed to skeletons, with seas and land, a mild mean temperature and gravity between 0.7 and 1.3 times 9.81 m/s². You wake on a shore in daylight, with plants and herds around you. A tidally locked home is allowed.
- Life chosen by physics: whether a planet lives and how far life has come, its chemistry, and plants shaped by its climate and its star's light.
- A food web per planet: grazers, hunters and scavengers, with herds, tracks, calls, nests and carcasses. Animals act from their needs: they graze, hunt, flee and flock.
- Tracks and calls lead you to animals, and a field guide on K fills in as you watch.
- A first step of hunting: animals and the robot have health by part, blows have weight, animals bleed, you carry a pack and make tools, and rocks are quarried by mass. The full loop is 0.17.
- A redesigned ship with a working cockpit and real exhaust, a redesigned robot and companion, entry heat, and speed you can feel.
- Light that adapts: exposure and white balance follow the light of the star, one ground colour at every range, and a Lighting quality setting.
- A reworked interface: one Go command, pins in place of waypoints and favourites, a HUD that shows only what applies, and markers on screen.
- A saved place that is never lost: one seat per player on a server, orbits that come back, and a loading screen that always ends.
- Multiplayer protocol 3, save format 3. Worlds made with 0.10 keep opening with 0.10. A 0.10 game cannot join a 0.11 server, and the public server started a new world.

## 0.12: places worth the trip

**Status:** planned

A real-size planet has thousands of times more ground than the small ones before it. This version makes sure that ground is worth crossing.

**The lesson.** The team behind Breath of the Wild fixed how fast the player moves, then how many things pull the eye, and only then the size of the map. Starfield made large planets without fixing either number, and its own former designer says the planets "start to feel very samey". Size was not the mistake. Large planets also give what small ones cannot: a far horizon with something on it.

- **Two numbers come first.** For walking, gliding and low flight, we set how many distinct things come into sight per minute of travel. The generators are held to those numbers.
- **Landmarks from the physics that is already there,** seen from far away: volcano plumes, impact basins, canyons, river deltas, glacier fronts, storm cells, a herd raising dust.
- **Relief that hides and shows.** A range blocks the view and a pass opens on what comes next, so each crest is a small discovery. The plates of 0.11 are checked for relief at three scales: the landmark, the screen and the step.
- **Rare things stay rare.** A few kinds of landform exist on one planet in hundreds. The companion says so when you find one.
- **A hand-drawn set of landmark kinds** that the rules place and vary. Pure noise does not make a place you remember.
- **A planet's numbers change how you move.** Thin air does not carry a wingsuit, heavy gravity shortens a jump, a fast spin gives short days. Each world asks something different of you.
- **Low flight is a pleasure.** The wingsuit rides ridge lift and rising warm air from the real wind and the sunlit ground. The ship can hold a height above the terrain.
- **No shortcut across a planet.** You fly, glide or walk there. Warp stays for the distance between worlds.
- **Seen and not reached.** The companion keeps what you looked at from afar and did not visit, as open questions. It never tells you where to go.
- **Follow a friend.** From the board computer you choose a friend and tell your ship to follow theirs, and you stop it with one key. Your ship keeps a set distance behind and to the side, matches their speed, climbs, lands near them and never touches their ship. It works over a planet, in space and through warp: when they spool, you spool, and you drop out beside them. The computer says what it is doing and why it stopped. Both players see who follows whom.
- **A seat for the companion.** The ship is redesigned with a socket of its own for the companion, as an astromech droid has on a fighter. When you board, it flies to its place, locks in and rides there, seen from the cockpit and from outside, in single player and for friends. It no longer floats loose in the cabin.
- **Small worlds stay small.** Moons and asteroids at their real size are places you walk around in minutes.
- **The sight test.** The probe lands on a thousand seeded points per planet class, counts what is in sight, travels, and counts again. A class below the numbers fails the build. It also saves one picture per planet on a sheet, to catch planets that look alike.

**Played, not assumed.** On a planet nobody has seen, each player can say at any moment where they are heading and why.

## 0.13: ground you can shape

**Status:** planned

You can dig into the ground anywhere, take what comes out, and reshape the land. This version also makes the ground as light in memory as it can be, because digging on a real-size planet is only possible if it is.

**The lesson.** No game studied runs a full volume at real size: the planets of Space Engineers stop at 120 km across. No Man's Sky caps how many edits it keeps, and past the cap the ground grows back over bases, which players took for a bug. Astroneer stopped at four players because the host works out the dug ground for each of them. Valheim had to rewrite its ground after one network object per dig made loading slow. Subnautica removed its digging tool because "you can just dig your way anywhere". Each of these costs has a rule below.

What you can do:

- **Dig, fill and flatten anywhere.** Pits, trenches, terraces, ramps, roads and walls of earth. The ground is smooth and sculpted, not made of cubes: a grid of cubes does not fit a round planet.
- **What comes out is real.** A dig yields material by mass, and which material follows from the geology of that spot: soil, then rock, with veins and ice where the planet put them.
- **A tool must be harder than the ground.** Soil gives way to anything. Hard rock needs a tool made for it. So a place that must be earned stays earned, and nobody digs straight to it.
- **Gravity and mass count.** Moving a hill is work, and more so on a heavy world. What you dug has to go somewhere.
- **Large works show.** A big enough pit or wall is seen from the air.
- **Everyone sees the same ground.** What a friend dug is there when you arrive, and still there next month.
- **Nothing grows back in silence.** There is no hidden limit. If a limit is ever reached, the game says so, and the ground around a base is never undone.

How it stays light:

- **Only what was dug is stored.** The planet comes from the seed. A dig is a change laid over it.
- **One record per piece of ground,** never one per stroke of the tool, compressed with zstd.
- **You receive the ground near you.** A joining player is not sent every dig in the universe.
- **The server stores and passes on.** It never builds ground.
- **The ground you see is rebuilt to be lean:** points shared between triangles, no constant data carried per point, and a collision shape made from heights.
- **Changes are kept in a compact binary file,** not as text in the save.

Budgets, each one a test that fails the build:

| What | Budget |
| --- | --- |
| One stroke of the tool | 5 bytes |
| One changed piece of ground (64 m) on disk | a few hundred bytes |
| A planet with 10,000 changed pieces | about 3 MB |
| The ground drawn near you | about a fifth of the memory it takes in 0.11 |

These numbers were computed from the data layout, not measured. The first task of this version is to measure them.

Tunnels, caves you dig and building arrive with 0.17.

**Played, not assumed.** Players reshape a place and come back to it.

## 0.14: real space

**Status:** planned

Space is beautiful because of what is in it. Every object below comes from physical rules and is a place you can fly to.

**The lesson.** A new sight that changes nothing in play is read as a backdrop. So each object below changes how you fly or what you can see when you are there.

- Star life stages from mass and age: giants, white dwarfs, neutron stars and black holes.
- Supernovae when a massive star's life runs out, then an expanding remnant; pulsars flashing at their real spin, with a beam you can fly through.
- Nebulae from the real gas of the spiral arms: red and teal glowing clouds around young hot stars, blue dust lit by nearby stars, dark clouds that hide what is behind them, teal shells around dying stars.
- Star clusters: young and blue in the arms, old and gold in the halo, with a night sky to match on their planets.
- Black holes at every galaxy centre and from dead stars, bending the light behind them, with a glowing disc only when something feeds them. Their pull is real, and an orbit near one is a piece of flying.
- Asteroid belts with real gaps from the giant planets' resonances, Trojans, Kuiper belts, and comets whose two tails point the way physics says.
- A hard frame budget: all of space together costs at most 1 ms of GPU time.

### Probes

You already drive a robot from far away. A probe is the same link to a smaller machine that you can afford to lose. It goes where the robot cannot go yet.

**The lesson.** The scout of Outer Wilds is a camera you launch: it shows a place and never marks a reward. Mass Effect 2 made scanning a slow sweep repeated for hours, and Elite Dangerous let a scanner stand in for the flight, after which an explorer wrote that strange planets "now appear all the same". Real probes are the better model: Venera 13 lasted 2 hours and 7 minutes on Venus, and the Galileo probe never reached a surface on Jupiter.

- **A camera that falls, not a scanner.** You release the probe from the ship and watch through it on the way down. It gives no map, no icon and no percentage.
- **The planet decides how long it lives.** Its life follows from the planet record: minutes on a lava world, hours on a cold one. In a gas giant it never lands. It sinks until the pressure ends it.
- **The release is the skill.** Speed and angle matter under the real gravity and the real air. Too fast and it burns. On a small asteroid it bounces.
- **The ship is the relay.** When the ship passes behind the planet the picture stops, so a fast-spinning world gives you a window to hold.
- **A probe is lost for real.** There is no recall. It stays where it came to rest, and other players can find it.
- **What it brings back:** the view, and what a robot would need to survive there (heat, pressure, acid). That is a reason to build protection in 0.17.
- **First on a world.** The first probe to reach a world leaves its owner's name there.
- **A station for a crew.** One flies the ship and holds the link, the other guides the probe.

**Played, not assumed.** Players go out of their way to reach an object they first saw as a dot in the sky. After a probe shows a place, they want to go and stand there.

## 0.15: together

**Status:** planned

The people who play Abstractions play it with friends. This version is for them.

**The lesson.** The makers of Sea of Thieves share every reward among the crew "so there's no disharmony", and do not separate friends by power, because then "you either have to play at the same pace or very close". Hello Games found that being in the same world was "of almost no value to the player" until there were things worth doing together. Death Stranding showed that the tracks of people who are not there still make you feel them.

- **One crew, one progress.** What a crew finds, learns and earns belongs to all of it: the field guide, the charts, the pins and later the patterns. Nobody is left behind for missing an evening.
- **Useful on the first evening.** Nothing a friend owns is out of reach of a newcomer. Gear can be lent and tools can be handed over.
- **Names.** Whoever finds a place, a species or a comet names it, and the name carries who found it.
- **Traces.** Robots leave tracks like animals do. Ground walked often wears into a path. You can build a cairn and leave a mark that others find.
- **While you were away.** The companion tells you what your friends found and where they went since you last played.
- **Stations aboard.** One flies, one reads the chart, one watches and scans. The stations ask for attention, not for levels.
- **Things in the world, not menus.** A course is plotted on a chart everyone aboard can see and point at.
- **A shared stock,** and shared earthworks from 0.13. A shared base follows when building arrives in 0.17.
- **Faces and signals.** The robot's screen face shows expressions, and a light or a call carries across a valley.
- **No harm between players** unless both ask for it.
- **A cost test per player.** Players spread across a planet multiply the server's work, so each added player has a budget.

**Played, not assumed.** A friend who joins for the first time is doing something that matters within ten minutes.

## 0.16: the thread

**Status:** planned

A reason to go far: the story of the control container, of who is driving, and of what else is out there. It was under "Later" before. The owner writes the story; this version builds what carries it.

**The lesson.** Outer Wilds gives nothing but knowledge, and it works because the game keeps track of it for you: "Keeping track of what they've learned should not be the challenging part of the game." Caves of Qud keeps "a static backbone" of fixed, hand-made places inside its generated world. A generated universe needs that spine.

- **A hand-written spine** of fixed places and finds, set in the generated universe from the seed.
- **You advance by what you understand.** The places of the thread could always be reached. You did not know where they were or what they meant.
- **The companion's log** holds what you certainly saw, linked as open questions. It records and never guesses for you.
- **Clues are things in the world,** not pages of text: a structure, a signal, a track, an orbit that should not be there.
- **Each clue points somewhere,** and each important one exists in more than one place. A clue that players misread is a clue to fix.
- **Three depths.** What you see first raises a question. What you find by looking points onward. The answers are where nobody arrives by accident.
- **The thread crosses worlds.** It follows one question from planet to planet and gives a reason to visit the classes you would skip.
- **For a crew.** A clue found by one is in the log of all, with who found it. The large answers need the crew in the same place.
- **The control container opens.**

**Played, not assumed.** A player who stops for two weeks opens the log and knows what to do next.

## 0.17: hunting, mining, crafting and building

**Status:** planned

**Goal:** hunt what lives on a planet, take what lies on it, and build gear from both. No recipe and no weapon is written by hand. Crafting is a physics question with many right answers, and the planet decides which ones you can reach.

**The lesson.** Subnautica made sure "all the stuff that you crafted would help you traverse and find stuff". Valheim's food rewards and never kills. No Man's Sky spent years undoing small inventories and recharge timers. The makers of Noita warn that a physics simulation is hard to turn into a game. So this version starts small and every item must take you somewhere.

**A prototype comes first.** Four intents on ten tissues, played by the crew, before the full grammar is built. If asking for a function is not fun at that size, the design changes there.

0.17.0 is the complete loop, playable end to end. The rest follows as 0.17.x.

### Crafting, the idea

**You ask for a function, the game solves it with what you carry.**

| Intent | What physics requires |
| --- | --- |
| Cut | An edge harder than the target, a thin edge angle that the material's toughness can hold, mass behind it |
| Pierce | A point harder than the target, a slender shaft that does not buckle |
| Crush | A heavy head, a tough head, a shaft that takes the shock |
| Block | Area, toughness, and a weight you can still move |

- **Every item extends your reach.** A build is worth making because it opens a place, a material or an animal you could not get to before.
- **Material behaviour comes out of the tissue numbers, not out of a table.** Crystal is hard and brittle: a glass blade cuts flesh like nothing else and shatters on shell. Elastic bone makes shafts that absorb shock. Resin is glue and sets joint strength. Fibre is binding. A conductive shaft carries the charge of a gland to the edge. A heat gland makes a blade that burns. Layering hide over bone trades hardness for toughness.
- **A form grammar** assembles grip, shaft, head, guard and counterweight from hand-designed parametric pieces with sockets. Tissues dress the pieces, so gear looks like its source and shows its signature.
- **It explains itself.** Every build shows why it is good or bad in plain words: "edge too soft for shell", "weakest joint: resin binding", "too heavy to swing on this world". The companion proposes three builds for an intent; you swap one piece and the numbers update live.
- **The planet is the workshop.** Treatments are physical and use where you are: dry hide in strong sunlight, fire-harden bone at a lava shore, freeze-set gel on an ice world, etch crystal in an acid sea. A world with no heat source cannot harden anything; you carry the piece to one that can.
- **The planet is also the judge.** The same hammer is heavier on a heavy world: swing time follows the item's inertia, the robot's strength and local gravity. What was a fine weapon at home is a burden two planets later.
- **The right tool gives the right yield.** Skinning needs an edge sharper than the hide; cracking a shell needs a crusher; draining a gland whole needs a careful pierce. The wrong tool ruins the tissue. So each kill asks for a tool, and each tool comes from an earlier kill.
- **Patterns.** A build you liked is saved as a pattern (intent plus pieces), and can be made again from better tissues: same design, better bone.
- **Repair and swap.** Items wear and break at their weakest joint. You replace that piece and keep the rest. Better materials wear much more slowly, so a fine item is a thing you keep.
- **Armour by the same grammar.** The intents are resist heat, cold, pressure, acid, blow. Insulation needs low conductivity and thickness; plates need hardness; all of it weighs, which costs speed and jump height, more so on heavy worlds.
- **More than one way through a hazard.** A hot world can be met with plates, with a treatment, or by landing on the night side.
- **Knowledge.** A tissue's numbers are unknown until the companion scans it or you use it. The field guide then remembers what it is good for.

### In 0.17.0

- **Inventory and HUD.**
  - A hotbar of six slots at the bottom centre: small, black and white, the held item marked, wear shown as a thin bar.
  - The inventory is a tab of the right-side panel (map, field guide, inventory and crafting share one panel, one key each). The game keeps running beside it.
  - A list, not a grid of tiny icons: each row has the item's silhouette, its name, its signature, mass and wear. Sort and type-to-filter work as in the map list. Tissues stack by species, tissue and quality; items do not.
  - Limits by mass, not slots: the robot carries what its frame can lift under this planet's gravity. The load shows on the robot and in how it moves.
  - The ship and the base hold the rest, and sorting what you carry is never a task.
  - Robot status in a HUD corner: frame integrity, and one line per hazard only while it applies (heat, cold, pressure, acid).
- **No clock to feed.** The robot has no hunger and nothing that drains by time alone. Danger comes from where you are and how far you are from the ship.
- **Harvesting**, with the tool rule above. Breaking a part in a fight drops that part. A crew shares the yield.
- **Mining.** Outcrops and boulders placed by geology rules: veins near basalt, dense metal in craters, minerals in the river beds from 0.9, ice cores. A pick must be harder than the rock. The scanner gives odds, not a map. Minerals are tissues too (hardness, density, conductivity), so the same grammar uses them.
- **Tunnels.** Below the surface the ground becomes a volume, only where somebody dug: galleries, shafts and rooms, with overhangs. A tunnel is stored like any other dig, as a change over the seed, and exists in detail only near a player.
- **No floor set by the game.** A tunnel is kept as its path and its width, so a long one costs no more to store than a short one. How deep you get is decided by the planet: the rock gets harder, hotter and heavier above you, and your tools and plates must keep up. On a large hot world the depths stop you. A small cold moon or an asteroid can be dug right through.
- **Gravity inside is real.** It weakens as you go down and is zero at the centre. Jump into a shaft through an asteroid and you fall past the middle and rise to the far side.
- **Building.** A base of your own, from the same pieces, tissues and joints as gear: walls, floors, roofs and doors assembled by the grammar, not stacked cubes. A wall of hide and bone does not hold what a wall of stone holds.
- **Crafting** as described: the four intents, the grammar, the explanations, the three proposals, treatments, patterns, repair, armour.
- **Combat.** Creatures gain health per body part and attacks derived from their features: horns charge, tails sweep, necks lunge, and wind-up grows with mass so large animals telegraph. Aggression comes from guild and life history: hunters that take prey your size attack, and so does anything cornered and a parent near its young. Grazers still flee. Wounded animals behave wounded: they limp, run, or turn. The robot swings, thrusts and blocks along arcs set by the item's grip, mass and reach.
- **Fair fights.** Animals act for themselves, which can be unfair to a player. Every attack can be seen coming, and there is always a way to leave.
- **Tolerances.** Armour is checked against the planet record. A lava world breaks an unprotected frame; the plates of a heat-dweller open it. Each planet's life opens the next.
- **A ladder per planet** follows from the web: plants, grazers, hunters, then the apex, each rung's tissues beating the next. The apex is the planet's boss without a script.
- **Death.** A broken robot leaves the human unharmed: you wake in a spare body at the base and your gear waits where you fell. Nothing else is lost. Items are saved as recipes (tissue origins, pieces, treatments) and rebuilt from the seed.
- **A zen server setting.** A server can turn off attacks on the robot and damage from planets, for a quiet game. The default is the full game.

### Follows in 0.17.x

Ranged weapons from energy-store tissues (elastic, gas bladder, reactive gland, charged tissue); drill rigs; decay of untreated organic tissue; a silhouette critic that picks the most readable of 20 candidate builds; gas harvesting from giants; the intents reach, throw, launch and contain.

**Played, not assumed.** Players make a tool because of a place they want to reach, and can say which place.

## 0.18: life odds

**Status:** planned

The core rule, which planets live and how far life got, arrives with 0.11. This version widens it.

- Tidal heating and thick ice shells allow life on moons and cold worlds.
- Life under the ice and in the deep, around vents.
- Dead worlds stay interesting: fossils, the ruins of ecosystems, strange geology, and the silence.
- The region around Vereya is tuned so that a first trip always finds something alive.

The dice are biased, not the physics. Every planet stays coherent; living ones are just more common than in reality.

## 0.19: sapient species

**Status:** planned

Some worlds have people.

**The lesson.** This is where comparable projects ran out of years. Ultima Ratio Regum, which generates cultures, reached its version 0.8 after almost ten years. The people in No Man's Sky stand still behind counters. So this version makes one people that is fully there before it makes many, and it shows a culture through things.

- **One people first.** The first release has a single sapient species, complete: bodies, homes, tools, daily life. More peoples follow once one of them holds your interest for an evening.
- A species at the top of a food web becomes sapient with a small chance.
- **They live like the animals do.** People act from their needs through the day: they work, eat, gather and sleep. Nobody waits in place for the player.
- **Culture** is a trait tree seeded by biome, diet and body plan: values, social structure, how they build, what they wear, how they name things.
- **Culture is in objects.** A trait exists only if you can see it in a house, a tool, a garment or a habit. A culture shares small things, such as one style of seat, that make it read as one people.
- **Variety by structure.** Not every culture varies in the same ways. One choice unlocks others, clashing options are removed, and a few hand-made archetypes are varied.
- **Uneven odds.** Common traits are common and rare ones are rare, so a rare culture reads as rare.
- **Villages and cities** take their style from the culture, not from the biome.
- **Technology levels** from stone to spaceflight.

**Played, not assumed.** A player can tell two peoples apart from their villages alone, before meeting anyone.

## 0.20: religion and history

**Status:** planned

What a people believes and remembers, shown in the world.

**The lesson.** Caves of Qud does not simulate centuries. It writes the account a people gives and lets the player join the dots, and it turns history into places and objects, so that knowing a story leads you somewhere. Dwarf Fortress simulates everything, and its author says "you need to do post-processing or investigation to find any good moments".

- **A creation myth** built from the sky they actually see: the colour of their sun, their moons, their neighbours.
- **Deities with domains,** tenets, and sects that split from them.
- **History as events** on rulers, places and relics. It is generated as the story a culture tells about itself, not as a simulation of centuries.
- **A small shared structure.** Each people has a few themes, and every event, relic and shrine carries one of them. Nothing stands alone.
- **Told through things:** shrines, ruins, murals, inscriptions, festivals, the way a temple is built.
- **History leads somewhere.** A place named in a story is a place you can find, and a relic named in a story is an object you can hold and use.
- **A myth can shape the land.** A story about a fallen star can put a crater where the story says.
- **The companion sorts it.** What you learned is kept by people and by time, so a story read in pieces holds together.

**Played, not assumed.** A player follows a story to a place and finds what the story said.

## 0.21: contact

**Status:** planned

Meeting them.

**The lesson.** Heaven's Vault and Chants of Sennaar choose each word for what the player needs next, and a word understood changes what the player can do. No Man's Sky hands out words in no useful order and explains each scene anyway, so the language can be ignored. No game found has made a generated language that players learn, so this part is new ground and is tested early.

- **Languages** learned word by word. They use the regional vocabulary from 0.11, so the creature names you heard for hours turn out to mean something.
- **The words you need come first.** What a people says to you draws on the words for what is in front of you.
- **Understanding changes what you can do.** Nothing explains a scene for you. A word you know opens a trade, a door or a warning.
- **A crew learns together.** A word learned by one is known to all.
- **Trade** and what each culture values. The economy behind it is 0.22.
- **Reputation** per culture, shaped by what you do.
- **Species with spaceflight** that you meet in other systems, with stations and ships.
- **Your companion as interpreter.** The more words of a people you know, the better you bargain.

This builds on the board computer and the discovery atlas from 0.6.

**Played, not assumed.** A player uses a word they learned to get something they wanted.

## 0.22: economy

**Status:** planned

A local economy that comes from the same data as everything else. What a settlement sells depends on what lives and grows around it. What it buys depends on what it lacks and what its culture values.

This version and the next are far ahead. Each will be cut to what a player can see and act on when its turn comes.

- **Stock from the area.** Each settlement produces from its ecosystem and its ground, and consumes by its population. A village by a slime marsh sells gel; a mountain village sells crystal.
- **Prices from scarcity and culture.** Rare here means dear here: heat scales are worth a fortune on an ice world and little at home. Culture adds taste: a people that values light pays well for anything that glows.
- **Local makers use the 0.17 grammar.** Goods for sale are generated by the same crafting system, from local tissues and in the culture's style. One system serves the player and the people, so a trader's gear tells where it comes from.
- **Ways to exchange** follow culture and technology: barter for small groups; object money for village networks, where a rare, durable, light tissue of the region becomes the currency (shells, crystals, hard seeds); minted coins for advanced cultures, limited by their real reserves; gift and debt for communal cultures, where reputation is the currency.
- **Exchange rates** come from the value of each currency's backing tissue in each region. Gaps between planets make trade worth it, and the ship becomes a freighter.
- **Order books.** Settlements post buy and sell orders from their stock and shortages. In multiplayer, players add their own orders on top.
- **Transport costs physics, not fuel.** Cargo mass changes the Newtonian flight: a full ship accelerates less and brakes earlier on landing.
- **Risk pays.** Rare tissues sit where planets are dangerous: hazards, dense predators, hostile peoples.
- **Information is a resource.** You only know the prices of places you visited, as of your visit. The companion records them with their date. A deposit survey can be sold.
- **Requests from shortages.** A settlement short of something posts a request, because 0.9 weather or overhunting really cut its supply. No quest is written by hand.
- **Taboos.** A religion from 0.20 can make a species or a mountain sacred. Hunting or drilling there closes that people's market, or earns a contract to protect it.
- **Another way through tolerances.** If you cannot survive the lava world yet, buy the scales from a people who live there.
- **Courier contracts with a deposit:** carry goods for a settlement or another player, and the deposit covers the cargo.
- **Nothing runs in the background.** A settlement's stock and prices are a pure function of its seed and the time. Only what you changed is stored. Sell 50 gels and the price drops, then recovers by formula, like populations.
- **Everything is physical.** No money comes from nowhere. Life and ground produce; wear, decay and consumption destroy. A view of what enters and what leaves the economy catches inflation early.
- **The companion summarises:** where to sell, what is missing, which price is old. No spreadsheet needed.

Hunter, trader and maker are three ways to progress, and any mix works. Perishable goods, cargo mass and local currencies keep markets local, so no single hub swallows all trade.

## 0.23: geopolitics and conflict

**Status:** planned

Peoples with enough technology form states, draw borders, trade, ally, expand and fight, within their own species and between species. It stays coherent because it follows the same rule as everything else: history is a pure function of the seed, the universe clock and what players changed. Nothing runs in the background.

- **Polities from geography and culture.** States form around fertile basins, rivers and coasts (from 0.11 terrain and 0.22 stock). Their size is limited by how far their technology can hold land, so stone-age peoples have tribes and valleys, and industrial ones have nations and empires. Culture from 0.19 sets how they govern and what they value.
- **Borders and claims.** A border is where two polities' reach meets, shaped by mountains, rivers and seas. Contested land is where reach overlaps.
- **Diplomacy with reasons.** Alliances, rivalries and treaties come from shared or opposed interests: the same scarce resource, the same enemy, the same religion (0.20), trade that pays both sides (0.22).
- **Wars that make sense.** A war starts from a cause the world holds: scarcity after a bad season, a border on a rich deposit, a sacred site, a broken treaty. Its course follows strength, supply lines, terrain and technology. Its outcome moves borders, ruins towns and shows in the world as forts, battlefields, refugees and new ruins. Wars are scheduled events like hunts, so every player sees the same war.
- **Between species.** Two sapient species on one planet, or a spacefaring one meeting another, compete for the same land or worlds, or find a niche apart. Body plans and needs matter: a species that breathes ammonia does not want the same planets as one that breathes water, so some neighbours never fight at all.
- **Expansion into space.** A spacefaring species spreads through nearby systems over centuries, its frontier growing along the stars it can reach and colonise (a percolation model: not every colony sends more colonies, so empires have holes and edges). The universe at a given time shows where each frontier stands.
- **Players change it.** Trade, gifts, taking sides and broken deals enter the change log. A polity's state is the formula plus that log, so what a player does shifts history for everyone on that server.
- **The robot is a visitor, not a ruler.** You can help, trade, mediate and take sides. Running a state is not part of the game.

## Later

**Status:** not yet ordered

- Natural caves, underwater structures and things under the ice. They use the same local volumes as the tunnels of 0.17.
- Crews that specialise (one hunts, one crafts, one hauls) and trade in any local currency. Items carry their maker's name.
- Binary and multiple star systems, and eccentric orbits.
- A galaxy map you can fly through.
- Mod support for generators.

## Research behind this

- Universe, planets and life (0.8 to 0.22): https://mycelium.facile.studio/artifacts/2026-11-17-procedural-universe-for-abstractions-research-and-roadmap-november-2026-5fb680
- What exploration games do well and badly (the order from 0.12 on): https://mycelium.facile.studio/artifacts/2026-11-20-what-exploration-games-do-well-and-badly-and-what-it-changes-in-the-abstractions-roadmap-november-2026-2b8d92

The sources quoted in this file:

- Breath of the Wild, the CEDEC 2017 talk on pull and density: https://game.watch.impress.co.jp/docs/news/1078569.html
- Starfield's former designer on its planets: https://www.pcgamer.com/games/rpg/former-bethesda-designer-says-starfield-fell-flat-because-of-its-reliance-on-procgen-planets-i-dont-think-its-in-the-same-calibre-as-the-other-two-fallout-or-skyrim
- Outer Wilds on wandering with a reason and on small planets: https://www.mobiusdigitalgames.com/news/the-intentionality-of-wandering
- Outer Wilds on knowledge and the ship log: https://media.gdcvault.com/GDC+2021/beachum_gdc_2021(1).pdf
- Kate Compton on generated things that look alike: https://galaxykate0.tumblr.com/post/139774965871/so-you-want-to-build-a-generator
- Hello Games on looking at generated planets at scale: https://gamesbeat.com/unraveling-the-mysteries-of-no-mans-skys-18-quintillion-planets/
- Space Engineers on the size of its planets: https://blog.marekrosa.org/2015/11/planets-because-you-wanted-them_12
- A player's study of the edit cap in No Man's Sky: https://steamcommunity.com/sharedfiles/filedetails?id=2526352095
- Astroneer on the cost of dug ground with several players: https://blog.astroneer.space/p/day-1-multiplayer-and-beyond/
- Valheim's rewrite of its ground: https://steamcommunity.com/games/892970/announcements/detail/5411613798265437193
- Subnautica's director on removing digging: https://forums.unknownworlds.com/discussion/comment/2287209
- The scout of Outer Wilds: https://outerwilds.fandom.com/wiki/Scout_Launcher
- Scanning in Mass Effect 2, in its director's words: https://www.gamewatcher.com/2010-21-05-mass-effect-3-will-keep-the-mining-mini-game-it-serves-a-purpose
- An explorer on the Elite Dangerous scanner: https://forums.frontier.co.uk/threads/why-fss-erased-a-lot-of-what-made-exploration-interesting.467000/
- Venera 13 on Venus: https://nssdc.gsfc.nasa.gov/imgcat/html/object_page/v13_vg261_262.html
- The Galileo probe in Jupiter: https://science.nasa.gov/mission/galileo-jupiter-atmospheric-probe/
- Sea of Thieves on shared progress: https://www.windowscentral.com/the-sea-of-thieves-vision-interview
- Sean Murray on multiplayer in No Man's Sky: https://www.eurogamer.net/the-big-sean-murray-interview
- Hideo Kojima on traces in Death Stranding: https://gameinformer.com/interview/2019/09/16/hideo-kojima-answers-our-questions-about-death-stranding
- Caves of Qud on a fixed backbone: https://www.gamedeveloper.com/design/tapping-into-the-potential-of-procedural-generation-in-caves-of-qud
- Caves of Qud on generated history: https://www.pcgworkshop.com/archive/grinblat2017subverting.pdf
- Charlie Cleveland on crafting in Subnautica: https://cdn.arstechnica.net/wp-content/uploads/2019/06/b6791daa-1334-4615-9693-0da4af5a297dmezzanine.txt
- Iron Gate on food in Valheim: https://www.pcgamer.com/valheim-developer-says-zelda-and-skyrim-were-bigger-influences-than-survival-games/
- No Man's Sky's inventory rework: https://www.nomanssky.com/2022/10/waypoint-update
- The makers of Noita on physics and play: https://www.rockpapershotgun.com/the-noita-devs-on-how-to-make-a-fun-game-when-everything-is-falling
- Rain World on animals that act for themselves: https://www.gamedeveloper.com/design/crafting-the-complex-chaotic-ecosystem-of-i-rain-world-i-
- Ultima Ratio Regum, ten years of generated culture: https://www.gamedeveloper.com/design/the-10-year-journey-of-ultima-ratio-regum-the-culture-generating-roguelike
- Mark Johnson on culture you can see: https://www.rockpapershotgun.com/how-to-procedurally-generate-culture
- Tarn Adams on simulated history: https://www.gamedeveloper.com/design/q-a-dissecting-the-development-of-i-dwarf-fortress-i-with-creator-tarn-adams
- Heaven's Vault on a language you decipher: https://www.gamedeveloper.com/design/how-inkle-developed-its-own-ancient-language-for-i-heaven-s-vault-i-
