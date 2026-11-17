# Roadmap

Abstractions is growing from an island game into a universe you explore as a small robot, with a second robot for company. This file says what each version adds and in what order.

Versions after 0.6 are plans, not promises. Order and scope can change.

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

## 0.6: the universe (in development)

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

## 0.7: physical planets

Planets get a kind, and the kind comes from physics.

- **Stars by region.** Old yellow and red stars in the core and halo, hot young stars in the arms. Rocky planets are rare around low-metal stars.
- **A planet record.** Mass, radius, gravity, orbit, axial tilt, rotation, and a moon count, all derived from the star.
- **Atmosphere.** Composition, density capped by mass and distance, then albedo and greenhouse effect.
- **Surface.** Mean temperature from the star's luminosity, the orbit distance, albedo and greenhouse. Liquid water from temperature and pressure.
- **The habitable zone.** Inner and outer limits from the published fit by stellar temperature, so "can this world have oceans" has a real answer.
- **Planet classes.** Ocean, island, desert, ice, lava, barren and gas giant, each with its own palette, sea level and sky.
- **Moons and rings.** Moons you can land on. Tidal heating gives a moon a second route to being warm.

One record per planet feeds everything below it. This is the version that makes the tenth landing different from the first.

## 0.8: climate and terrain

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

## 0.9: ecosystems

Life that fits its planet.

- **A biochemistry roll** per living planet: what life is built from, which sets its colours and shapes.
- **Producers per biome.** Plants, fungi or stranger things, drawn from the climate.
- **A food web** per planet from a niche model: who eats whom, how many levels.
- **Species.** Each has a mass, a diet, a niche and a range of biomes.
- **Populations** from an energy pyramid. Producers scale with light and water, and each level up keeps about a tenth.
- **Bodies** from 6 to 8 skeleton templates, with proportions driven by mass and gravity. Heavy worlds make squat, sturdy animals; light worlds make tall, thin ones.
- **Behaviour.** Grazing, hunting, fleeing, flocking, day and night habits.
- **A field guide.** Your companion scans and records what you find.

Species are generated once per planet, then placed by the same scatter rules that place trees today.

## 0.10: life odds

A realistic universe would be almost empty. This one is tuned to be worth exploring.

- The chance of life is a product of factors: in the habitable zone, liquid water, an atmosphere, the star's age.
- Then the dice are loaded: about one system in three has a living world, and the region around Vereya is richer.
- Tidal heating and thick ice shells allow life on moons and cold worlds.
- Dead worlds stay interesting: ruins, fossils, strange geology, and the silence.

The dice are biased, not the physics. Every planet stays coherent; living ones are just more common than in reality.

## 0.11: sapient species

Some worlds have people.

- A species at the top of a food web becomes sapient with a small chance.
- **Culture** is a trait tree seeded by biome, diet and body plan: values, social structure, how they build, what they wear, how they name things.
- **Variety by structure.** Not every culture varies in the same ways. One choice unlocks others, clashing options are removed, and a few hand-made archetypes are varied.
- **Uneven odds.** Common traits are common and rare ones are rare, so a rare culture reads as rare.
- **Villages and cities** take their style from the culture, not from the biome.
- **Technology levels** from stone to spaceflight.

## 0.12: religion and history

What a people believes and remembers, shown in the world.

- **A creation myth** built from the sky they actually see: the colour of their sun, their moons, their neighbours.
- **Deities with domains,** tenets, and sects that split from them.
- **History as events** on rulers, places and relics. It is generated as the story a culture tells about itself, not as a simulation of centuries.
- **Told through things:** shrines, ruins, murals, inscriptions, festivals, the way a temple is built.
- **A myth can shape the land.** A story about a fallen star can put a crater where the story says.

## 0.13: contact

Meeting them.

- **Languages** learned word by word.
- **Trade** and what each culture values.
- **Reputation** per culture, shaped by what you do.
- **Species with spaceflight** that you meet in other systems, with stations and ships.
- **Your companion as interpreter.**

This builds on the board computer and the discovery atlas from 0.6.

## Later, not yet ordered

- Building: a base of your own, upgrades for the robot, the ship and the companion.
- The control container opens, and you find out more about who is driving.
- Caves, underwater structures and things under the ice.
- Other humans and their robots: co-op exploration.
- A galaxy map you can fly through.
- Mod support for generators.

## Research behind this

The techniques and sources for 0.7 to 0.12 are in the research report:
https://mycelium.facile.studio/artifacts/2026-11-17-procedural-universe-for-abstractions-research-and-roadmap-november-2026-5fb680
