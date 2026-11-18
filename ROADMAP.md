# Roadmap

Abstractions is growing from an island game into a universe you explore as a small robot, with a second robot for company. This file says what each version adds and in what order.

Versions after 0.8 are plans, not promises. Order and scope can change.

## The setting

You are a human who never leaves a control container on the planet Kodya, in the Vereya system. From there you drive a robot over an instant quantum link. A smaller floating robot travels with you. Everything you see is generated from one seed.

## Rules that hold in every version

- Every object's seed comes from its parent's seed and its index. The same universe seed always gives the same universe.
- Nothing is stored except what you changed or discovered.
- Every generator is a pure function with a test that the same seed gives the same result.
- A level of detail only exists near you. Far things are cheap.
- Only generate what shows up in the world. A trait nobody can see or hear is not generated.
- The game stays smooth. Heavy work runs off the main thread and arrives within a per-frame budget.

## 0.5: islands (released)

An endless seeded archipelago on a flat world: six biomes, villages with interiors, ruins and camps, a sea with reefs and kelp, a day and night cycle, birds, butterflies and fireflies.

## 0.6: the universe (released)

The foundation for everything after it.

| Area | What it adds |
| --- | --- |
| Universe | Galaxies on a grid, spiral arms, star classes, solar systems with 2 to 8 planets, generated names |
| Planets | True spheres you can walk around, 3 to 40 km in radius, with their own gravity, day length and orbit |
| Terrain | The island generator sampled in 3D, so a planet has no seams |
| Home | Kodya and the Vereya system. You wake up next to the control container, a small station and a landing pad |
| Ship | Newtonian flight, landing gear, flight assist, orbits that work, a warp drive, an autopilot |
| Sky | The sun is the real star. Atmosphere that fades to black, a cloud layer to fly through, stars and galaxies in their true directions |
| Board computer | Tables of systems and galaxies you have discovered, with fast travel. Visiting a place reveals its neighbours |
| Robot | A humanoid body with a screen face, first and third person views |
| Flight gear | Boot boosters that brake a fall, a wingsuit that glides, and powered flight with both. No fuel |
| Companion | A floating robot with a screen face that follows you, reacts to what happens and lights the night a little |
| Sound | Ambient music generated for each planet, silent in space. Wind, sea, footsteps, ship and interface sounds |
| Interface | Black and white, the Undefined font, pixelarticons, blurred backdrops. World management: rename, duplicate, delete |

In 0.6 every planet uses the same island generator. Planets differ by seed, size, gravity, day length, star and music, not by kind.

## 0.7: flight and feel (released)

A pass on how the game plays once you are in it: a ship steered with the keys, roll and loops on the wingsuit, a flight assist that makes the ship go where it points, a compass and a map, and light derived from the real star.

## 0.8: physical planets (in development)

Planets get a kind, and the kind comes from physics.

- **A planet record.** Starlight, air pressure, albedo, greenhouse warming, temperature, water and axial tilt, all derived from the star, the orbit and the seed.
- **The habitable zone.** Inner and outer limits from the published fit by stellar temperature, so "can this world have oceans" has a real answer.
- **Six solid classes.** Ocean, island, desert, ice, lava and barren, each with its own ground, sea level, life, sky, far view, flight and sound.
- **The board computer** as a panel beside the running game, with an interactive map and a card for each planet. A minimap in the corner.
- **Flight.** A ship that goes where it points and stops when you let go, wingsuit loops, a warp that reads as one sequence.

0.8.1 adds gas giants, moons you can land on and rings. Tidal heating gives a moon a second route to being warm.

One record per planet feeds everything below it. This is the version that makes the tenth landing different from the first.

## 0.9: climate and terrain

**Goal:** the planet record shapes the ground you walk on. Two island worlds no longer look alike, and a planet has poles, a warm belt, wet coasts and dry interiors.

- **A climate field**, a pure function of (planet record, point on the sphere, season). No grid and no solver: it must be sampled per chunk at streaming speed.
  - Temperature: the record's mean, minus a pole-to-equator gradient that weakens with thick air, minus a lapse rate with altitude (6.5 K per km at home air, scaled by gravity and air), plus a seasonal swing from the axial tilt and the position on the orbit.
  - Moisture: the ocean share times a circulation factor by latitude (wet belt at the equator, dry belts near 30 degrees, wet mid-latitudes), fading with distance from the sea, raised on slopes that face the prevailing wind and lowered behind them. Wind bands follow the spin (a slow rotator has one cell per hemisphere, a fast one three).
  - Latitude is warped by low-frequency noise so no band is a straight line.
- **Biomes by ordered rules** over temperature and moisture (ice, tundra, taiga, forest, meadow, steppe, desert, tropical, plus the class biomes). Noise never picks a biome; it only perturbs the inputs. This replaces the noise climate.
- **Relief from physics.** Mountain height scales with the inverse of gravity; continent size scales with radius; old small worlds are smoother.
- **Landforms by class**, as shape functions added to the ground: dune fields on deserts, impact craters on barren worlds (size-frequency from the seed), glaciers and frozen seas you can walk on for ice, basalt flows and lava channels for lava.
- **Rivers without simulation:** valley lines traced downhill from wet highlands on a coarse grid per region, cached, carved as a smooth channel. They feed 0.11's river-bed minerals.
- **Weather** from the same field: rain, snow, dust and storms, as particles and sound, where the local climate says so, with wind from the band.
- **Seasons** you can see: the snow line and the plant tint move with the orbit.

### Ready for multiplayer

No networking is written. The structure is put in place so that adding a server later is an addition, not a rewrite.

| Rule | What it means in the code |
| --- | --- |
| The world is the seed plus a list of changes | One `Changes` log holds everything a player altered: harvested deposits, kills, dropped gear, pits. Saves store that log. A server would hold the same log. |
| Players act through intents | Input becomes an `Intent` message (move, harvest, craft, strike). Simulation systems read intents, never the keyboard. A remote player is the same message from another source. |
| No singleton player in the rules | Every rule takes a player by `PlayerId`. Today one player exists; only the camera, the HUD and the input code may assume it. |
| One clock | The universe clock is the only time the simulation reads. No wall clock, no frame count. |
| Fixed step for the simulation | Creatures, combat and harvest run at 30 Hz in `FixedUpdate`. Drawing interpolates. |
| Stable ids | `SpeciesId` (planet, index), `CreatureId` (chunk, slot, generation), `ItemId`. No `Entity` is stored in a save or a message. |
| Seeded randomness | Every roll is `hash(seed, id, tick)`. Nothing uses a thread RNG. |

## 0.10: ecosystems

**Goal:** life that belongs to its planet: recognisable, pretty, coherent in how it lives, and cheap to run. A player who lands on a new world should be able to guess its animals from its sky, and be right.

### The chain, one pure step after another

**1. Does this planet live, and how far has life come?** Two numbers from the record.
- *Life or not:* a product of factors (a liquid, air, temperature, the star's age), then loaded dice: about one system in three has a living world and the region around Vereya is richer. Dead worlds keep ruins. (This pulls the core of "life odds" forward; tidal moons and fossils stay in 0.12.)
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

**3. Tissues are the contract.** Every body, plants included, is built from tissue records: bone, hide, shell, gel, crystal, fibre, resin, gland. One record holds physics (hardness, toughness, density, elasticity, heat tolerance, conductivity, reactivity: acid, venom, charge) and look (colour, pattern, gloss, translucency, glow), plus an optional rigid shape (spike, shard, ridge). Biochemistry sets the planet's baseline; habitat and niche shift it. Each species has a **signature**: the tissue value farthest from its planet's average. 0.11 crafting reads tissues and nothing else.

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

Looks stay clean by rule: bilateral or radial symmetry always, at most two accent tissues, strong value contrast so the shape reads in the pixel pass. Features are data, not decoration: horns, tails, necks, jaws, plates and glands are listed with size and tissue, because behaviour reads them now and attacks read them in 0.11. Animation is procedural on rigid parts (gaits from leg count and mass, pulsing for jellies, flow for slimes). No skinning.

**8. Life history.** Small species are many, quick and short-lived; large ones are few, slow and guard their young. That gives swarms and clutches of eggs on one end and a lone giant with one calf on the other. Nests, eggs and young exist in the world.

**9. Behaviour from needs.** One small utility model for every animal: hunger, thirst, fear, rest, company, curiosity, care for young. Diet, guild and life history set the weights. From them come grazing, browsing, hunting, stalking, ambush, fleeing, flocking, burrowing, sleeping, guarding a nest. Day, night or twilight habits follow eye size and glow.
- Hunters pick the weak and the small; herds bunch and put their young in the middle; prey that cannot outrun a hunter hides or shows its warning colours.
- Herds migrate with the seasons from 0.9, along latitude, and gather at water in dry seasons.
- **The robot is not prey in 0.10.** One rule decides it, and 0.11 turns it on. Animals still notice the robot: curious ones approach, shy ones keep their distance, herds part around it, a parent stands between it and the nest.

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

**16. The field guide.** The companion scans what you look at and records name, guild, who it eats and who eats it, tissues, and where it is soft. It fills in as you observe: a web you draw by watching. It lives in the right-side panel, next to the map.

## 0.11: hunting, mining and crafting

**Goal:** hunt what lives on a planet, take what lies on it, and build gear from both. No recipe and no weapon is written by hand. Crafting is a physics question with many right answers, and the planet decides which ones you can reach.

0.11.0 is the complete loop, playable end to end. The rest follows as 0.11.x.

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

### In 0.11.0

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

### Follows in 0.11.x

Ranged weapons from energy-store tissues (elastic, gas bladder, reactive gland, charged tissue); open pits cut into the height field; drill rigs; decay of untreated organic tissue; a silhouette critic that picks the most readable of 20 candidate builds; gas harvesting from giants; the intents reach, throw, launch and contain.

## 0.12: life odds

The core rule, which planets live and how far life got, arrives with 0.10. This version widens it.

- Tidal heating and thick ice shells allow life on moons and cold worlds.
- Life under the ice and in the deep, around vents.
- Dead worlds stay interesting: fossils, the ruins of ecosystems, strange geology, and the silence.
- The region around Vereya is tuned so that a first trip always finds something alive.

The dice are biased, not the physics. Every planet stays coherent; living ones are just more common than in reality.

## 0.13: sapient species

Some worlds have people.

- A species at the top of a food web becomes sapient with a small chance.
- **Culture** is a trait tree seeded by biome, diet and body plan: values, social structure, how they build, what they wear, how they name things.
- **Variety by structure.** Not every culture varies in the same ways. One choice unlocks others, clashing options are removed, and a few hand-made archetypes are varied.
- **Uneven odds.** Common traits are common and rare ones are rare, so a rare culture reads as rare.
- **Villages and cities** take their style from the culture, not from the biome.
- **Technology levels** from stone to spaceflight.

## 0.14: religion and history

What a people believes and remembers, shown in the world.

- **A creation myth** built from the sky they actually see: the colour of their sun, their moons, their neighbours.
- **Deities with domains,** tenets, and sects that split from them.
- **History as events** on rulers, places and relics. It is generated as the story a culture tells about itself, not as a simulation of centuries.
- **Told through things:** shrines, ruins, murals, inscriptions, festivals, the way a temple is built.
- **A myth can shape the land.** A story about a fallen star can put a crater where the story says.

## 0.15: contact

Meeting them.

- **Languages** learned word by word. They use the regional vocabulary from 0.10, so the creature names you heard for hours turn out to mean something.
- **Trade** and what each culture values. The economy behind it is 0.16.
- **Reputation** per culture, shaped by what you do.
- **Species with spaceflight** that you meet in other systems, with stations and ships.
- **Your companion as interpreter.** The more words of a people you know, the better you bargain.

This builds on the board computer and the discovery atlas from 0.6.

## 0.16: economy

A local economy that comes from the same data as everything else. What a settlement sells depends on what lives and grows around it. What it buys depends on what it lacks and what its culture values.

- **Stock from the area.** Each settlement produces from its ecosystem and its ground, and consumes by its population. A village by a slime marsh sells gel; a mountain village sells crystal.
- **Prices from scarcity and culture.** Rare here means dear here: heat scales are worth a fortune on an ice world and little at home. Culture adds taste: a people that values light pays well for anything that glows.
- **Local makers use the 0.11 grammar.** Goods for sale are generated by the same crafting system, from local tissues and in the culture's style. One system serves the player and the people, so a trader's gear tells where it comes from.
- **Ways to exchange** follow culture and technology: barter for small groups; object money for village networks, where a rare, durable, light tissue of the region becomes the currency (shells, crystals, hard seeds); minted coins for advanced cultures, limited by their real reserves; gift and debt for communal cultures, where reputation is the currency.
- **Exchange rates** come from the value of each currency's backing tissue in each region. Gaps between planets make trade worth it, and the ship becomes a freighter.
- **Order books.** Settlements post buy and sell orders from their stock and shortages. In multiplayer, players add their own orders on top.
- **Transport costs physics, not fuel.** Cargo mass changes the Newtonian flight: a full ship accelerates less and brakes earlier on landing.
- **Risk pays.** Rare tissues sit where planets are dangerous: hazards, dense predators, hostile peoples.
- **Information is a resource.** You only know the prices of places you visited, as of your visit. The companion records them with their date. A deposit survey can be sold.
- **Requests from shortages.** A settlement short of something posts a request, because 0.9 weather or overhunting really cut its supply. No quest is written by hand.
- **Taboos.** A religion from 0.14 can make a species or a mountain sacred. Hunting or drilling there closes that people's market, or earns a contract to protect it.
- **Another way through tolerances.** If you cannot survive the lava world yet, buy the scales from a people who live there.
- **Courier contracts with a deposit:** carry goods for a settlement or another player, and the deposit covers the cargo.
- **Nothing runs in the background.** A settlement's stock and prices are a pure function of its seed and the time. Only what you changed is stored. Sell 50 gels and the price drops, then recovers by formula, like populations.
- **Everything is physical.** No money comes from nowhere. Life and ground produce; wear, decay and consumption destroy. A view of what enters and what leaves the economy catches inflation early.
- **The companion summarises:** where to sell, what is missing, which price is old. No spreadsheet needed.

Hunter, trader and maker are three ways to progress, and any mix works. Perishable goods, cargo mass and local currencies keep markets local, so no single hub swallows all trade.

## Later, not yet ordered

- Building: a base of your own, from the same pieces, tissues and joints as 0.11 gear.
- The control container opens, and you find out more about who is driving.
- Caves, underwater structures and things under the ice. Mine tunnels share their local volume representation.
- Other humans and their robots: co-op exploration. One seed, one universe, and a server that holds what players changed: stocks, prices, kills, deposits. Players specialise (one hunts, one crafts, one hauls), trade by barter or in any local currency, and form groups that share a base and a stock. Items carry their maker's name. Fights between players are opt-in.
- A galaxy map you can fly through.
- Mod support for generators.

## Research behind this

The techniques and sources for 0.8 to 0.16 are in the research report:
https://mycelium.facile.studio/artifacts/2026-11-17-procedural-universe-for-abstractions-research-and-roadmap-november-2026-5fb680
