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

You are a human who never leaves a control container on the planet Kodya, in the Vereya system. From there you drive a robot over an instant quantum link. A smaller floating robot travels with you. Everything you see is generated from one seed.

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

- Up to 16 players in one universe, with the same seed, clock and discoveries, each with their own robot, ship and companion.
- Open to LAN, a server list with ping, direct connect, passwords, and server keys remembered on first join.
- `abstractions-server` for Linux and Docker, and a public server at space.facile.studio.
- A real cockpit with a seat, dashboard and moving controls.
- Real clouds and weather: one cloud field per planet, seen the same from the ground and from orbit.
- 0.10.1: multiplayer polish, the map with explored ground, waypoints and favourites, the glider rework, step-up, and wing collisions.

## 0.11: living worlds at real size

**Status:** in progress

**Goal:** planets the size of real planets, each with life that belongs to it: recognisable, pretty, coherent in how it lives, and cheap to run. A player who lands on a new world should be able to guess its animals from its sky, and be right.

### Real-size planets

- Rocky planets from 1,500 to 12,000 km in radius, gas giants from 25,000 to 70,000 km, stars at their real sizes, orbits in AU.
- Kodya is Earth-like: 6,371 km, 9.81 m/s², one turn in 23 h 56 min. No day length is set anywhere; days follow from each planet's spin and orbit.
- Terrain on every scale: tectonic plates make continents, mountain ranges rise where plates meet, and rivers run hundreds of kilometres to the sea. Today's islands, villages and plants stay as the finest layer.
- Real air: density falls with the scale height of each planet, clouds sit from 1 to 12 km, and the sky and drag read the same model.
- Moons by how they form: about a quarter of rocky worlds have one large moon from a giant impact, some have small captured moons, giants have families of moons. Kodya gets a moon.
- One light model for every galaxy: the band across the night sky is the summed light of the real stars of your galaxy, with its real arms, core and dust. Other galaxies show their true shape and tilt, and grow into a band as you fly into them.
- Precision kept at every scale, the map and its explored ground at planet size, and a ship fast enough to reach orbit in a couple of minutes.
- Old worlds keep their version: a world made with 0.10 opens with 0.10, which the game downloads for you.

### Ecosystems: the chain, one pure step after another

**1. Does this planet live, and how far has life come?** Two numbers from the record.
- *Life or not:* a product of factors (a liquid, air, temperature, the star's age), then loaded dice: about one system in three has a living world and the region around Vereya is richer. Dead worlds keep ruins. (This pulls the core of "life odds" forward; tidal moons and fossils stay in 0.14.)
- *Complexity:* how far evolution got, from the star's age and how stable the climate is. It is a ladder, and each rung unlocks body plans:

| Rung | What lives there |
| --- | --- |
| 1 | Mats and films only: coloured ground, glowing shores, nothing that moves |
| 2 | Soft life: slimes, jellies, drifting colonies, simple growths |
| 3 | Shelled and segmented life, first plants with structure, first hunters |
| 4 | Skeletal animals, forests with layers, herds, flyers where the air allows |
| 5 | An old ecosystem: armour, venom, speed, mimicry, long food chains |

So some worlds have only a red film on the rocks and a sea that glows at night, and that is a discovery of its own. It is also free to run.

**2. Biochemistry, one roll per living planet,** from the record:

| Choice | Driven by | What it changes |
| --- | --- | --- |
| Solvent: water, ammonia, hydrocarbon, molten silicate | temperature and pressure | which classes can live, body fluids, slime and gel tissues |
| Energy: light, chemistry, heat | flux, volcanism, depth | where producers grow: open ground, vents, lava shores |
| Structure: carbon fibre, silica and crystal, metal-rich shell | temperature, the star's metal content | hardness and heat tolerance of every tissue on the planet |
| Pigment | the colour of the star's light | plant colour: green under a yellow star, red to black under a red dwarf, pale under a hot blue star |

A lava world's life is crystalline and heat-proof, an ice world's is slow and insulated, and a red dwarf's forests are dark.

**3. Tissues are the contract.** Every body, plants included, is built from tissue records: bone, hide, shell, gel, crystal, fibre, resin, gland. One record holds physics (hardness, toughness, density, elasticity, heat tolerance, conductivity, reactivity: acid, venom, charge) and look (colour, pattern, gloss, translucency, glow), plus an optional rigid shape (spike, shard, ridge). Biochemistry sets the planet's baseline; habitat and niche shift it. Each species has a **signature**: the tissue value farthest from its planet's average. 0.13 crafting reads tissues and nothing else.

**4. Plants and other producers.** Not a list of tree models: a small set of growth architectures (trunk and branches, rosette, mat, vine, bulb, stalk and cap, kelp, crystal lattice), each with dials that the climate turns:
- Height falls with gravity and with wind; canopy layers (ground, shrub, canopy, emergent) grow in number with productivity.
- Leaf size grows with moisture and shrinks with light; needles in the cold; wax, spines and water-storing bodies in dry places.
- Plants defend themselves with tissues: thorns, resin, toxins. Those are harvestable and a grazer's mouth has to cope with them.
- What a plant offers decides who eats it: leaves, fruit, nectar, seeds, dead wood. Flowers exist only where a pollinator species exists; otherwise plants are wind-sown and plain. Fruit exists only where something carries seeds.
- Seasons from 0.9 show: bloom, fruit, leaf fall, dormancy.

Their productivity (light times moisture times temperature fit) is the energy budget of the biome.

**5. A food web per planet** from the niche model: each species gets a niche value, a feeding range and a range centre, and eats whatever falls inside. Species count and connectance scale with productivity and with the complexity rung, so rich old worlds have long chains and harsh young ones have two levels. Each consumer also gets a guild that says how it eats: grazer, browser, fruit eater, nectar feeder, seed eater, filter feeder, hunter, ambusher, scavenger, decomposer, parasite. The guild shapes the mouth, the senses and the behaviour.

**6. More than eating.** A second, sparse network of relationships, generated from who lives where:
- **Pollinators and seed carriers** tie animals to plants (see 4).
- **Scavengers and decomposers** clear carcasses, so a kill is found and eaten by something, and fungi grow where things die.
- **Riders and cleaners:** a small species that lives on a large grazer.
- **Mimics:** a harmless species that copies the warning pattern of a toxic one in the same biome.
- **Builders:** species that leave structures you can find: mounds, nests, hives, burrows, webs, reefs.
- **A keystone** per biome: the species whose loss moves the most others. Usually the top hunter or a builder.

**7. Bodies.** Six plans: skeletal, amorphous (slimes), radial (jellies), segmented, colonial (swarms), crystalline. Each plan has two to four archetype silhouettes built from hand-designed parametric parts with sockets, the way the ship and the props are built in code today. Rules turn the planet into the body:

| Rule | Effect |
| --- | --- |
| Gravity and the square-cube law | Squat, thick-limbed bodies on heavy worlds; tall and thin on light ones; a cap on the largest land animal |
| Air density against gravity | Thick air and low gravity allow large flyers and gas-bag floaters; thin air allows none; no air, no wings |
| Cold (Bergmann, Allen) | Larger, rounder bodies with short limbs and thick insulating hide in the cold; long thin limbs and large radiating surfaces in the heat |
| Small worlds and islands | Giants shrink and the small grow |
| Humidity | Darker in wet climates, pale in dry ones |
| Light | Large eyes on dim worlds; no eyes and long feelers under permanent cloud or in the deep; forward eyes on hunters, side eyes on prey |
| Sound | Calls are low and carry far in thick air; in thin air species signal with light and pattern instead |
| Guild | Flat wide mouths for grazers, long necks for browsers, jaws and claws for hunters, sieves for filter feeders |
| Arms race | Predator weapon hardness and prey armour hardness are drawn together and rise with the complexity rung |

Looks stay clean by rule: bilateral or radial symmetry always, at most two accent tissues, strong value contrast so the shape reads in the pixel pass. Features are data, not decoration: horns, tails, necks, jaws, plates and glands are listed with size and tissue, because behaviour reads them now and attacks read them in 0.13. Animation is procedural on rigid parts (gaits from leg count and mass, pulsing for jellies, flow for slimes). No skinning.

**8. Life history.** Small species are many, quick and short-lived; large ones are few, slow and guard their young. That gives swarms and clutches of eggs on one end and a lone giant with one calf on the other. Nests, eggs and young exist in the world.

**9. Behaviour from needs.** One small utility model for every animal: hunger, thirst, fear, rest, company, curiosity, care for young. Diet, guild and life history set the weights. From them come grazing, browsing, hunting, stalking, ambush, fleeing, flocking, burrowing, sleeping, guarding a nest. Day, night or twilight habits follow eye size and glow.
- Hunters pick the weak and the small; herds bunch and put their young in the middle; prey that cannot outrun a hunter hides or shows its warning colours.
- Herds migrate with the seasons from 0.9, along latitude, and gather at water in dry seasons.
- **The robot is not prey in 0.11.** One rule decides it, and 0.13 turns it on. Hunters do hunt each other's prey: hunts, kills and carcasses are pure functions of the seed and the clock, so every player sees the same hunt. Animals still notice the robot: curious ones approach, shy ones keep their distance, herds part around it, a parent stands between it and the nest.

**10. Bioluminescence, by rule.** Glow appears where it pays: night-active species, deep water, caves, worlds under thick cloud or dim stars. It has one of four uses, which sets where it sits and how it behaves: lure (a hunter's bait), warning (toxic tissue), courtship (pulsing patterns at dusk), hiding from below (belly lights in swimmers). Its colour comes from the biochemistry, and plants and mats can glow too.

**11. Populations from energy.** Each level keeps about a tenth of the level below. Density falls with body mass to the power three quarters and home range grows with mass, so the apex is rare, large and wide-ranging without a script.

**12. Cascades you can cause.** The population field is a steady state of the web, computed by formula, and the kill log shifts it. Remove hunters from a valley and grazers rise there, then plants thin, then the grazers starve back. Nothing is simulated while you are away: the state is a function of the seed, the log and the elapsed time.

**13. Signs.** Tracks, droppings, grazed patches, bones, nests and calls let you find animals before you see them. The companion reads them for the field guide. Calls are synthesised from body size and air density, so a planet has a dawn chorus of its own.

**14. Three levels of simulation,** which is what keeps it fast:

| Where | What runs | Cost |
| --- | --- | --- |
| The whole planet | The population field: a pure function of seed, place, season and the change log. Nothing is stored or ticked. | free |
| Within about 1 km | Herd agents: one point per group at 5 Hz, moving by the same needs | microseconds |
| Within about 150 m | Individuals at 30 Hz with full behaviour and animation, capped at a fixed count, instanced flat-shaded meshes | inside the 2 ms budget |

A planet has 12 to 40 species and a biome shows 4 to 10 of them. Meshes are built once per planet off the main thread. Individuals appear from the field and dissolve back into it.

**15. Names.** Each region of the galaxy has its own sounds and a small vocabulary (light, hard, glass, acid, hunter, big, small). A species name is built from the words of its signature traits, with a plain epithet: "Silvaren, the glowing hunter".

**16. The field guide.** The companion scans what you look at and records name, guild, who it eats and who eats it, tissues, and where it is soft. It fills in as you observe: a web you draw by watching. It opens with K, in its own panel beside the running game.

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
- The region around Vereya is tuned so that a first trip always finds something alive.

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
