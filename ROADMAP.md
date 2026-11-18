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

## 0.11: hunting and crafting

Hunt what lives on a planet, and build your gear from it. No recipe and no weapon is written by hand: the gear comes from the tissues and from physics.

- **Harvesting.** Creatures, plants and rocks give their tissues. Breaking a part in a fight drops that part.
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

Rocks and plants already exist, so the mineral and plant half can start before 0.10.

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
- **Trade** and what each culture values.
- **Reputation** per culture, shaped by what you do.
- **Species with spaceflight** that you meet in other systems, with stations and ships.
- **Your companion as interpreter.**

This builds on the board computer and the discovery atlas from 0.6.

## Later, not yet ordered

- Building: a base of your own, from the same pieces, tissues and joints as 0.11 gear.
- The control container opens, and you find out more about who is driving.
- Caves, underwater structures and things under the ice.
- Other humans and their robots: co-op exploration.
- A galaxy map you can fly through.
- Mod support for generators.

## Research behind this

The techniques and sources for 0.8 to 0.14 are in the research report:
https://mycelium.facile.studio/artifacts/2026-11-17-procedural-universe-for-abstractions-research-and-roadmap-november-2026-5fb680
