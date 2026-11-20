# Roadmap

Abstractions is growing from an island game into a universe you explore as a small robot, with a second robot for company. This file says what each version adds and in what order. The website shows it at https://abstractions.facile.studio/roadmap/.

Every version below has a status: **released**, **in progress** or **planned**. Planned versions are plans, not promises. Order and scope can change.

## At a glance

| Version | Name | Status |
| --- | --- | --- |
| 0.5 | Islands | released |
| 0.6 | The universe | released |
| 0.7 | Flight and feel | released |
| 0.8 | Physical planets | released |
| 0.9 | Climate and terrain | released |
| 0.10 | Multiplayer | released |
| 0.11 | Living worlds at real size | in progress |
| 0.12 | Real space | planned |
| 0.13 | Hunting, mining and crafting | planned |
| 0.14 | Life odds | planned |
| 0.15 | Sapient species | planned |
| 0.16 | Religion and history | planned |
| 0.17 | Contact | planned |
| 0.18 | Economy | planned |
| 0.19 | Geopolitics and conflict | planned |

## The setting

You are a human who never leaves a control container on your home planet. From there you drive a robot over an instant quantum link. A smaller floating robot travels with you. Everything you see is generated from one seed.

## Rules that hold in every version

- Every object's seed comes from its parent's seed and its index. The same universe seed always gives the same universe.
- Nothing is stored except what you changed or discovered.
- Every generator is a pure function with a test that the same seed gives the same result.
- What you see is real. Every light, cloud, creature and landmark is an object that exists at that place, made by physical rules. Nothing is decoration.
- Values come from physics, not from tuning: gravity, day length, air and sizes follow from the planet and its star.
- A level of detail only exists near you. Far things are cheap.
- Only generate what shows up in the world. A trait nobody can see or hear is not generated.
- The game stays smooth. Heavy work runs off the main thread and arrives within a per-frame budget, and every feature lands with a cost test.

## 0.5: islands

**Status:** released

An endless seeded archipelago on a flat world: six biomes, villages with interiors, ruins and camps, a sea with reefs and kelp, a day and night cycle, birds, butterflies and fireflies.

## 0.6: the universe

**Status:** released

The foundation for everything after it.

- Galaxies on a grid, spiral arms, star classes, solar systems with 2 to 8 planets, generated names.
- True sphere planets you can walk around, with their own gravity, day length and orbit, and the island generator sampled in 3D so a planet has no seams.
- The home planet and its system: you wake up next to the control container, a small station and a landing pad.
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

**Status:** in progress

Planets at their real size, each with life that belongs to it.

- Real scale: stars, planets and orbits at their true sizes, gravity from each planet's mass, days from each planet's own spin. The home planet is chosen from the universe, not made by hand, and carries a generated name.
- Continents from tectonic plates, mountain ranges where plates meet, and rivers hundreds of kilometres long; real air and moons that form the way real moons do.
- Life chosen by physics: whether a planet lives and how far life has come, its chemistry, and plants shaped by its climate and its star's light.
- A food web per planet: grazers, hunters, scavengers, pollinators and builders, with herds, nests and young.
- Animals act from their needs: they graze, hunt, flee, flock and sleep. Hunters hunt each other's prey; the robot is not prey yet.
- Remove the hunters from a valley and the grazers rise, then the plants thin: cascades you can cause.
- Tracks, calls and nests lead you to animals, and the companion's field guide fills in as you watch.
- Worlds made with 0.10 keep opening with 0.10.

## 0.12: real space

**Status:** planned

Space is beautiful because of what is in it. Every object below comes from physical rules and is a place you can fly to.

- Star life stages from mass and age: giants, white dwarfs, neutron stars and black holes.
- Supernovae when a massive star's life runs out, then an expanding remnant; pulsars flashing at their real spin.
- Nebulae from the real gas of the spiral arms: red and teal glowing clouds around young hot stars, blue dust lit by nearby stars, dark clouds, teal shells around dying stars.
- Star clusters: young and blue in the arms, old and gold in the halo.
- Black holes at every galaxy centre and from dead stars, bending the light behind them, with a glowing disc only when something feeds them.
- Asteroid belts with real gaps from the giant planets' resonances, Trojans, Kuiper belts, and comets whose two tails point the way physics says.
- A hard frame budget: all of space together costs at most 1 ms of GPU time.

## 0.13: hunting, mining and crafting

**Status:** planned

**Goal:** hunt what lives on a planet, take what lies on it, and build gear from both. No recipe and no weapon is written by hand. Crafting is a physics question with many right answers, and the planet decides which ones you can reach.

0.13.0 is the complete loop, playable end to end. The rest follows as 0.13.x.

### Crafting, the idea

**You ask for a function, the game solves it with what you carry.**

| Intent | What physics requires |
| --- | --- |
| Cut | An edge harder than the target, a thin edge angle that the material's toughness can hold, mass behind it |
| Pierce | A point harder than the target, a slender shaft that does not buckle |
| Crush | A heavy head, a tough head, a shaft that takes the shock |
| Block | Area, toughness, and a weight you can still move |

- **Material behaviour comes out of the tissue numbers, not out of a table.** Crystal is hard and brittle: a glass blade cuts flesh like nothing else and shatters on shell. Elastic bone makes shafts that absorb shock. Resin is glue and sets joint strength. Fibre is binding. A conductive shaft carries the charge of a gland to the edge. A heat gland makes a blade that burns. Layering hide over bone trades hardness for toughness.
- **A form grammar** assembles grip, shaft, head, guard and counterweight from hand-designed parametric pieces with sockets. Tissues dress the pieces, so gear looks like its source and shows its signature.
- **It explains itself.** Every build shows why it is good or bad in plain words: "edge too soft for shell", "weakest joint: resin binding", "too heavy to swing on this world". The companion proposes three builds for an intent; you swap one piece and the numbers update live.
- **The planet is the workshop.** Treatments are physical and use where you are: dry hide in strong sunlight, fire-harden bone at a lava shore, freeze-set gel on an ice world, etch crystal in an acid sea. A world with no heat source cannot harden anything; you carry the piece to one that can.
- **The planet is also the judge.** The same hammer is heavier on a heavy world: swing time follows the item's inertia, the robot's strength and local gravity. What was a fine weapon at home is a burden two planets later.
- **The right tool gives the right yield.** Skinning needs an edge sharper than the hide; cracking a shell needs a crusher; draining a gland whole needs a careful pierce. The wrong tool ruins the tissue. So each kill asks for a tool, and each tool comes from an earlier kill.
- **Patterns.** A build you liked is saved as a pattern (intent plus pieces), and can be made again from better tissues: same design, better bone.
- **Repair and swap.** Items wear and break at their weakest joint. You replace that piece and keep the rest.
- **Armour by the same grammar.** The intents are resist heat, cold, pressure, acid, blow. Insulation needs low conductivity and thickness; plates need hardness; all of it weighs, which costs speed and jump height, more so on heavy worlds.
- **Knowledge.** A tissue's numbers are unknown until the companion scans it or you use it. The field guide then remembers what it is good for.

### In 0.13.0

- **Inventory and HUD.**
  - A hotbar of six slots at the bottom centre: small, black and white, the held item marked, wear shown as a thin bar.
  - The inventory is a tab of the right-side panel (map, field guide, inventory and crafting share one panel, one key each). The game keeps running beside it.
  - A list, not a grid of tiny icons: each row has the item's silhouette, its name, its signature, mass and wear. Sort and type-to-filter work as in the map list. Tissues stack by species, tissue and quality; items do not.
  - Limits by mass, not slots: the robot carries what its frame can lift under this planet's gravity.
  - Robot status in a HUD corner: frame integrity, and one line per hazard only while it applies (heat, cold, pressure, acid).
- **Harvesting**, with the tool rule above. Breaking a part in a fight drops that part.
- **Surface mining.** Outcrops and boulders placed by geology rules: veins near basalt, dense metal in craters, minerals in the river beds from 0.9, ice cores. A pick must be harder than the rock. The scanner gives odds, not a map. Minerals are tissues too (hardness, density, conductivity), so the same grammar uses them.
- **Crafting** as described: the four intents, the grammar, the explanations, the three proposals, treatments, patterns, repair, armour.
- **Combat.** Creatures gain health per body part and attacks derived from their features: horns charge, tails sweep, necks lunge, and wind-up grows with mass so large animals telegraph. Aggression comes from guild and life history: hunters that take prey your size attack, and so does anything cornered and a parent near its young. Grazers still flee. Wounded animals behave wounded: they limp, run, or turn. The robot swings, thrusts and blocks along arcs set by the item's grip, mass and reach.
- **Tolerances.** Armour is checked against the planet record. A lava world breaks an unprotected frame; the plates of a heat-dweller open it. Each planet's life opens the next.
- **A ladder per planet** follows from the web: plants, grazers, hunters, then the apex, each rung's tissues beating the next. The apex is the planet's boss without a script.
- **Death.** A broken robot leaves the human unharmed: you wake in a spare body at the base and your gear waits where you fell. Items are saved as recipes (tissue origins, pieces, treatments) and rebuilt from the seed.

### Follows in 0.13.x

Ranged weapons from energy-store tissues (elastic, gas bladder, reactive gland, charged tissue); open pits cut into the height field; drill rigs; decay of untreated organic tissue; a silhouette critic that picks the most readable of 20 candidate builds; gas harvesting from giants; the intents reach, throw, launch and contain.

## 0.14: life odds

**Status:** planned

The core rule, which planets live and how far life got, arrives with 0.11. This version widens it.

- Tidal heating and thick ice shells allow life on moons and cold worlds.
- Life under the ice and in the deep, around vents.
- Dead worlds stay interesting: fossils, the ruins of ecosystems, strange geology, and the silence.
- Life follows the same rules everywhere, the home system included: a first trip finds life where the physics allows it.

The dice are biased, not the physics. Every planet stays coherent; living ones are just more common than in reality.

## 0.15: sapient species

**Status:** planned

Some worlds have people.

- A species at the top of a food web becomes sapient with a small chance.
- **Culture** is a trait tree seeded by biome, diet and body plan: values, social structure, how they build, what they wear, how they name things.
- **Variety by structure.** Not every culture varies in the same ways. One choice unlocks others, clashing options are removed, and a few hand-made archetypes are varied.
- **Uneven odds.** Common traits are common and rare ones are rare, so a rare culture reads as rare.
- **Villages and cities** take their style from the culture, not from the biome.
- **Technology levels** from stone to spaceflight.

## 0.16: religion and history

**Status:** planned

What a people believes and remembers, shown in the world.

- **A creation myth** built from the sky they actually see: the colour of their sun, their moons, their neighbours.
- **Deities with domains,** tenets, and sects that split from them.
- **History as events** on rulers, places and relics. It is generated as the story a culture tells about itself, not as a simulation of centuries.
- **Told through things:** shrines, ruins, murals, inscriptions, festivals, the way a temple is built.
- **A myth can shape the land.** A story about a fallen star can put a crater where the story says.

## 0.17: contact

**Status:** planned

Meeting them.

- **Languages** learned word by word. They use the regional vocabulary from 0.11, so the creature names you heard for hours turn out to mean something.
- **Trade** and what each culture values. The economy behind it is 0.18.
- **Reputation** per culture, shaped by what you do.
- **Species with spaceflight** that you meet in other systems, with stations and ships.
- **Your companion as interpreter.** The more words of a people you know, the better you bargain.

This builds on the board computer and the discovery atlas from 0.6.

## 0.18: economy

**Status:** planned

A local economy that comes from the same data as everything else. What a settlement sells depends on what lives and grows around it. What it buys depends on what it lacks and what its culture values.

- **Stock from the area.** Each settlement produces from its ecosystem and its ground, and consumes by its population. A village by a slime marsh sells gel; a mountain village sells crystal.
- **Prices from scarcity and culture.** Rare here means dear here: heat scales are worth a fortune on an ice world and little at home. Culture adds taste: a people that values light pays well for anything that glows.
- **Local makers use the 0.13 grammar.** Goods for sale are generated by the same crafting system, from local tissues and in the culture's style. One system serves the player and the people, so a trader's gear tells where it comes from.
- **Ways to exchange** follow culture and technology: barter for small groups; object money for village networks, where a rare, durable, light tissue of the region becomes the currency (shells, crystals, hard seeds); minted coins for advanced cultures, limited by their real reserves; gift and debt for communal cultures, where reputation is the currency.
- **Exchange rates** come from the value of each currency's backing tissue in each region. Gaps between planets make trade worth it, and the ship becomes a freighter.
- **Order books.** Settlements post buy and sell orders from their stock and shortages. In multiplayer, players add their own orders on top.
- **Transport costs physics, not fuel.** Cargo mass changes the Newtonian flight: a full ship accelerates less and brakes earlier on landing.
- **Risk pays.** Rare tissues sit where planets are dangerous: hazards, dense predators, hostile peoples.
- **Information is a resource.** You only know the prices of places you visited, as of your visit. The companion records them with their date. A deposit survey can be sold.
- **Requests from shortages.** A settlement short of something posts a request, because 0.9 weather or overhunting really cut its supply. No quest is written by hand.
- **Taboos.** A religion from 0.16 can make a species or a mountain sacred. Hunting or drilling there closes that people's market, or earns a contract to protect it.
- **Another way through tolerances.** If you cannot survive the lava world yet, buy the scales from a people who live there.
- **Courier contracts with a deposit:** carry goods for a settlement or another player, and the deposit covers the cargo.
- **Nothing runs in the background.** A settlement's stock and prices are a pure function of its seed and the time. Only what you changed is stored. Sell 50 gels and the price drops, then recovers by formula, like populations.
- **Everything is physical.** No money comes from nowhere. Life and ground produce; wear, decay and consumption destroy. A view of what enters and what leaves the economy catches inflation early.
- **The companion summarises:** where to sell, what is missing, which price is old. No spreadsheet needed.

Hunter, trader and maker are three ways to progress, and any mix works. Perishable goods, cargo mass and local currencies keep markets local, so no single hub swallows all trade.

## 0.19: geopolitics and conflict

**Status:** planned

Peoples with enough technology form states, draw borders, trade, ally, expand and fight, within their own species and between species. It stays coherent because it follows the same rule as everything else: history is a pure function of the seed, the universe clock and what players changed. Nothing runs in the background.

- **Polities from geography and culture.** States form around fertile basins, rivers and coasts (from 0.11 terrain and 0.18 stock). Their size is limited by how far their technology can hold land, so stone-age peoples have tribes and valleys, and industrial ones have nations and empires. Culture from 0.15 sets how they govern and what they value.
- **Borders and claims.** A border is where two polities' reach meets, shaped by mountains, rivers and seas. Contested land is where reach overlaps.
- **Diplomacy with reasons.** Alliances, rivalries and treaties come from shared or opposed interests: the same scarce resource, the same enemy, the same religion (0.16), trade that pays both sides (0.18).
- **Wars that make sense.** A war starts from a cause the world holds: scarcity after a bad season, a border on a rich deposit, a sacred site, a broken treaty. Its course follows strength, supply lines, terrain and technology. Its outcome moves borders, ruins towns and shows in the world as forts, battlefields, refugees and new ruins. Wars are scheduled events like hunts, so every player sees the same war.
- **Between species.** Two sapient species on one planet, or a spacefaring one meeting another, compete for the same land or worlds, or find a niche apart. Body plans and needs matter: a species that breathes ammonia does not want the same planets as one that breathes water, so some neighbours never fight at all.
- **Expansion into space.** A spacefaring species spreads through nearby systems over centuries, its frontier growing along the stars it can reach and colonise (a percolation model: not every colony sends more colonies, so empires have holes and edges). The universe at a given time shows where each frontier stands.
- **Players change it.** Trade, gifts, taking sides and broken deals enter the change log. A polity's state is the formula plus that log, so what a player does shifts history for everyone on that server.
- **The robot is a visitor, not a ruler.** You can help, trade, mediate and take sides. Running a state is not part of the game.

## Later

**Status:** not yet ordered

- Building: a base of your own, from the same pieces, tissues and joints as 0.13 gear.
- The control container opens, and you find out more about who is driving.
- Caves, underwater structures and things under the ice. Mine tunnels share their local volume representation.
- Groups of players: specialise (one hunts, one crafts, one hauls), trade by barter or in any local currency, share a base and a stock. Items carry their maker's name. Fights between players are opt-in.
- Binary and multiple star systems.
- A galaxy map you can fly through.
- Mod support for generators.

## Research behind this

- Universe, planets and life (0.8 to 0.18): https://mycelium.facile.studio/artifacts/2026-11-17-procedural-universe-for-abstractions-research-and-roadmap-november-2026-5fb680
