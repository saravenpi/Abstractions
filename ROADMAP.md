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

- **Stars by region.** Old yellow and red stars in the core and halo, hot young stars in the arms. Rocky planets are rare around low-metal stars.
- **A planet record.** Mass, radius, gravity, orbit, axial tilt, rotation, and a moon count, all derived from the star.
- **Atmosphere.** Composition, density capped by mass and distance, then albedo and greenhouse effect.
- **Surface.** Mean temperature from the star's luminosity, the orbit distance, albedo and greenhouse. Liquid water from temperature and pressure.
- **The habitable zone.** Inner and outer limits from the published fit by stellar temperature, so "can this world have oceans" has a real answer.
- **Planet classes.** Ocean, island, desert, ice, lava, barren and gas giant, each with its own palette, sea level and sky.
- **Moons and rings.** Moons you can land on. Tidal heating gives a moon a second route to being warm.

0.8.0 brings the record, the habitable zone and the six solid classes, with their ground, sea, sky, flight and sound. 0.8.1 adds gas giants, moons and rings.

One record per planet feeds everything below it. This is the version that makes the tenth landing different from the first.

## 0.9: climate and terrain

The planet record shapes the ground.

- **Temperature** by latitude and altitude, with seasons from the axial tilt.
- **Moisture** from distance to the sea and from wind bands.
- **Biomes** from temperature and moisture. This replaces the noise-based climate.
- **Gravity** sets how tall mountains and cliffs can be.
- **Radius** sets the scale of continents.
- **Kind** sets the rules: dunes on a desert world, glaciers and frozen seas on an ice world, basalt and lava rivers on a lava world, craters on a barren one.
- **Weather.** Rain, snow, dust and storms that follow the climate.
- **Erosion-like detail.** River valleys and coast shapes that read as formed, without simulating them.

Terrain parameters become functions of the planet record, not constants.

## 0.10: ecosystems

Life that fits its planet.

- **A biochemistry roll** per living planet: what life is built from, which sets its colours and shapes.
- **Producers per biome.** Plants, fungi or stranger things, drawn from the climate.
- **A food web** per planet from a niche model: who eats whom, how many levels.
- **Species.** Each has a mass, a diet, a niche and a range of biomes.
- **Populations** from an energy pyramid. Producers scale with light and water, and each level up keeps about a tenth.
- **Body plans,** not only skeletons: skeletal, amorphous (slimes), radial (jellies), segmented, colonial (swarms) and crystalline. Proportions follow mass and gravity. Heavy worlds make squat, sturdy animals; light worlds make tall, thin ones.
- **Tissues.** Every body, plant included, is built from tissues: bone, hide, shell, gel, crystal, fibre, resin, glands. One tissue record holds physical values (hardness, toughness, density, elasticity, heat tolerance, conductivity, reactivity) and its look (colour, pattern, gloss, translucency, glow). The physics, the renderer and 0.11 all read the same record.
- **Behaviour.** Grazing, hunting, fleeing, flocking, day and night habits.
- **Names that mean something.** Each region of the galaxy has its own sounds and a small generated vocabulary: words for light, hard, glass, acid, hunter, big, small. A species name is built from the words of its most unusual traits, so every glowing creature of a region shares a sound. The field guide gives the local name and a plain epithet, such as "Silvaren, the glowing hunter".
- **A field guide.** Your companion scans and records what you find: name, tissues, where it is soft.

Species are generated once per planet, then placed by the same scatter rules that place trees today.

## 0.11: hunting, mining and crafting

Hunt what lives on a planet, dig what lies under it, and build your gear from both. No recipe and no weapon is written by hand: the gear comes from the tissues and from physics.

- **Harvesting.** Creatures, plants and rocks give their tissues. Breaking a part in a fight drops that part.
- **Geology you can read.** Deposits are a pure function of the seed, the position and the depth, placed by simple geological processes: volcanic veins near lava and basalt, sediment layers on wet worlds, heavy minerals in river beds, dense metals in the craters of barren worlds, rare minerals at hot springs under the sea, cores and trapped gas in ice. Outcrop colours, surface rocks and plants that like rich soil show what lies below.
- **Prospecting.** The companion's scanner reads density and echo with limited range and precision. It gives odds, not a treasure map.
- **Digging and drilling.** A pick must be harder than the rock, the same rule as for shells. A drill needs an energy store, like a launcher: a spring crank, an explosive gland or a charged tissue. Heat rises with depth at a rate set by the planet record, and gas pockets, water and cave-ins wait below, so tolerances matter underground too.
- **Mining in steps.** First, outcrops and boulders placed like today's props, with no terrain change. Then open pits: shapes cut into the height field, stored as per-chunk changes and meshed again. Tunnels come with caves (see Later).
- **Drill rigs** are structures. Nothing runs while you are away: the yield follows from the elapsed time. Deposits run out, and only the extracted amount is stored.
- **Wear and decay.** Items wear and break at their weakest joint. Organic tissues decay unless treated: gel dries, hide rots. A broken robot leaves part of its gear behind.
- **Processing chains.** Raw tissue, then treatment (tanning, smelting, fire hardening), then piece, then item. Some treatments need a workshop.
- **Crafting by intent,** not by recipe. You ask for a function: cut, pierce, crush, block, reach, throw, launch, contain. A form grammar assembles a grip, a shaft, a head, a guard or a counterweight to meet it, from the tissues you carry.
- **Hand-made pieces, generated assemblies.** The pieces are modelled by hand, with dials and attachment points, so every result stays clean. The grammar picks and joins them. A critic renders 20 candidates at game resolution and keeps the one with the most readable silhouette.
- **Gear that looks like its source.** Tissues dress the pieces with their colour, pattern, gloss, translucency and glow. Each item shows one signature: the tissue value that differs most from the planet's average life. A slime gives a glassy coating, a jelly a glowing core, a crystal creature faceted edges.
- **Stats from physics.** Edge, reach, mass, inertia and the weakest joint are computed from the item itself. A blade cannot cut a shell harder than itself. Elements come from real tissues: fire from a heat gland, acid from a slime.
- **Ranged weapons.** The energy store decides the weapon: elastic tissue makes bows and slings, a gas bladder makes air guns, a reactive gland makes powder guns and rockets, a charged tissue makes coil guns. Projectiles are crafted too. Gravity and air pressure of the planet bend their flight.
- **Combat from bodies.** A creature's attacks come from its body plan: horns charge, tails sweep, long necks lunge. Wind-up time grows with mass, so big creatures telegraph slowly. Aggression comes from diet and niche.
- **Moves from physics.** Swing, thrust, smash, block, draw and throw arcs follow the item's grip, mass and reach. IK places the robot's hands.
- **The companion as engineer.** It searches your tissues for the best builds for an intent and shows three. You swap a part and the stats update live.
- **Tolerances.** The robot resists heat, cold, pressure, acid and radiation through the armour it wears. A lava world is out of reach until you wear the scales of a creature that lives in heat. Each planet's life opens the next planet.
- **A ladder per planet.** Plants, grazers, predators, then the apex. Each rung's tissues beat the next one. The apex is rare and large because of the energy pyramid, so it is the planet's boss without a script.
- **Death and saves.** A broken robot leaves the human in the container unharmed: you wake in a spare body at the base, and your gear waits where you fell. Items are saved as recipes (tissue origins, shaping, joints) and rebuilt from the seed. Kills are counted per species and region, and populations grow back by formula.

Items are named after their source: "Silvaren bow", "Velkra breastplate". The same tissues upgrade the ship and the companion.

Rocks and plants already exist, so the mineral and plant half can start before 0.10. Gas giants from 0.8.1 are harvested in flight, by passing the ship through their air. Creatures that eat minerals and crystal creatures live near deposits, so digging there has consequences.

## 0.12: life odds

A realistic universe would be almost empty. This one is tuned to be worth exploring.

- The chance of life is a product of factors: in the habitable zone, liquid water, an atmosphere, the star's age.
- Then the dice are loaded: about one system in three has a living world, and the region around Vereya is richer.
- Tidal heating and thick ice shells allow life on moons and cold worlds.
- Dead worlds stay interesting: ruins, fossils, strange geology, and the silence.

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
