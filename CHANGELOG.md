# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- The robot wakes with a kit: a stone spear in hotbar slot 1 and a stone knife in slot 2, knapped from the stone of the home planet, and nothing in its hands: press 1 to draw the spear. They are the lightest tools the workshop proposes, with their real mass and wear, so the first animal that goes for you can be fought and every tool you make after is better. A world saved before gets the kit once on its next load, when its pack holds no tool. In a session every player gets their own.
- A thrown tool is marked: a spear out in the world shows as "Your" and its name on the compass, the minimap, the map and over the world, where it came to rest, also in an animal that carries it off, until it is back in the pack.

### Removed

- The first-time hints are gone: the prompt line no longer suggests opening the map, dropping a pin, the minimap range or the stabiliser. It shows only what a key does right now. Every key stays on the controls page and in the menus, and a settings file that kept the hints done still loads.

### Changed

- The HUD no longer sits on dark boxes. The compass, the place lines top left, the ship and glide readout bottom left, the prompt, the planet banner, the frame rows, the labels of the markers over the world and the caption of the minimap are drawn straight over the world, on the edge margin, with a faint one-pixel shadow under their text and icons so they read over bright ground and sky. Warnings in orange and red have a slightly deeper shadow. The text of the HUD is white throughout. The minimap keeps its picture, with no shadow on its marks, and its caption now sits under it. The hotbar slots keep a very light tint, so a full slot and an empty one still read apart. Menus, the map and the other full screens keep their panels.
- The map of a planet zooms ten times closer: about 500 m across at the closest on a planet the size of Earth, under a metre a pixel, so the base, the pad, a pin and a herd beside them are each their own mark. Close up the map draws the ground itself, its land and sea colours with the light on its relief, where you have explored it. Marks and the ground under them stay on their pixel at every zoom, wherever you are on the planet.
- The ship flies twice as fast low over a planet. Its nose is clean, a quarter of the drag it had, so its engines hold 275 m/s in air at sea level instead of 137, and more in thinner air higher up. With the stabiliser the full lever asks for 219 m/s at 286 m over the ground, where it asked for 112, and the speed still drops near the ground: take-off, hover and landing are as before. In thick air the ship flies no faster than its engines can push against the drag.
- The fire of the main engines stays narrow. In thin air and in space the plume spread to more than three times the width of the nozzle, and four times with the boost. Its bright core now stays inside the nozzle and the faint envelope around it reaches about twice its width, fading as it widens; the boost makes the fire longer, not wider.
- The lift nozzles under the belly fire only to hold the ship up: in a hover, on take-off and landing and in slow flight, and they fade out as the ship gathers speed along its nose. A pitch, a roll or a turn in forward flight is made with the wing and the small attitude jets at the tail and the wing tips, in air and in space. The ships of other players hover on their lift nozzles too.
- The wing tip trails and lamp streaks stay whole at the new speeds.
- The ship sounds like a ship. Its engines roar with the noise of a real jet: a low broad roar that deepens as the throttle opens, the hiss of the nozzle, the crackle of a hard burn in air, and the whine of the pumps, which lags the lever and winds down after a shutdown. The lift nozzles have their own higher voice and grow harsh near the ground. The gear whirs and locks with a thump, the canopy latches and seals, the stabiliser hisses, hot nozzles tick as they cool, and the cabin hums. The warp drive spools as rising, beating tones and cruises on a bed of its own.
- Where you listen from changes what you hear. In the seat the canopy muffles the engines and the airframe carries their rumble, the wind over the canopy grows with speed and thick air, and the hull shakes through the speed of sound. Outside, a ship is heard from where it is, quieter and duller with distance, late by the time its sound takes to arrive, higher as it comes and lower as it goes, and with a boom when it passes faster than sound. Thin air carries less, and in a vacuum there is silence outside and only the airframe inside. The ships of other players are heard the same way.
- An atmospheric entry looks like one. A bow shock of glowing air wraps the nose and the wings, the nose and the leading edges burn hottest in the colour their temperature gives them, dull red, then orange, streamers peel off the wing tips and the tail, and a long faint wake trails behind. The gas of the planet's air colours the shock: pink and orange in nitrogen and oxygen, other hues in other airs. The shock lights the hull, and from the seat you see it through the canopy, ahead of the nose and around the windows. The hull roars deeper and harder as the heating grows. The glow follows the real heating of the nose: from a low orbit it starts high in the air, peaks where the air and the speed together heat the most, near 55 km on an Earth-like world, and dies as the ship slows. A thin air glows later and lower. Other players see the same sheath on your ship.
- Leaving the air is felt as it really is. The glow obeys the same rule on the way up, so a climb glows only if it is truly fast in thick air. The autopilot's climb to orbit now goes slowly through thick air and fast only where the air is thin, as a real ascent does, so its nose stays dark; it still reaches orbit in about a minute and a half. Through the speed of sound under the clouds, or in cloud or rain, the hull wraps itself in a vapour cone, on the way up as on the way down. High up, where the air is colder than the threshold for condensation, the engines leave a white contrail, short in dry air and long in wet air. The wind and the buffet thin out with the air to silence. Other players see your cone and your contrail.
- The Audio page has two new options. "Dynamic range" set to Reduced raises quiet moments and lowers loud ones, for laptop speakers, a noisy room or the night. "Mono" sends the same sound to both ears.
- The mix is balanced by measurement. Wind and rain no longer drown everything: a storm now sits under a ship at full throttle, where it was the loudest thing in the game. Calm moments are quieter, the music in play sits under the title music, and blows, steps and voices read over the beds.
- Footsteps sound like feet on real ground. Each step comes from the foot that touches down, heel then toe, in time with the walk, the run or the crouch, and both feet at once on a landing from a jump or a glide. Grass swishes, soil thuds, gravel and snow crunch, snow squeaks, sand hisses, mud sucks at the foot, shallow water splashes, the metal deck and the landing pad of the base ring short and hollow, and the wing of the ship drums. A run is louder than a walk, a crouched step is quiet, a heavier world makes every step harder, and no two steps are the same. The robot's armour ticks faintly on a hard landing. Other players' steps come from where they are. On a world with no air you hear only your own steps, through your body.
- The output is guarded by a limiter that turns the whole mix down for a moment when a sum of sounds would be too loud, in place of a curve that distorted it, and no peak goes over -1 dBTP. The music gives way by up to 6 dB under loud effects such as the engines.
- The pack and the workshop are one screen. The pack is on the left, grouped by kind (tools, stone, bone, horn and shell, hide, wood), each entry with its mass and a wear bar for a tool. The workshop is on the right: four tiles for cut, pierce, crush and throw, each with its icon and what the head must be, and three tool cards for the chosen one with the silhouette of the tool, the one reason it is good or bad, and its reach, mass, edge and swing as bars. The focused card lights the materials it takes in the pack, with the mass it takes. Make is a button with its key, and the new tool shows going into its hotbar slot, which runs under both panes. When nothing can be made the workshop says what is missing and how to get it, and an empty pack says how to fill it. Every action is a button with its key, and the mouse works everywhere: click a row, a tile, a card, a slot or an action.
- Every tool has its own icon, drawn from its parts: the head shows what it is (knife, blade, axe, spear, javelin, club, hammer), the length and the grey of each part show its shaft and what it is made of. The same icon shows on its workshop card, in the pack and on the hotbar, crisp at every interface size. Each material in the pack has the icon of what it is: stone, flint, ice, bone, horn and shell, hide, wood, resin.

### Removed

- No village, camp, cabin, lighthouse, dock, stilt hut, shrine, ruin, tower, pyramid, obelisk or stone circle stands on a planet any more, with their rooms, lamps, torches, fires and smoke. Peoples and what they build come back with the intelligent species of a later version. The home base and its landing pad stay. Worlds saved before open as they were, without them.
- The sea floor holds no more sunken boats, anchors, amphorae or chests: kelp, boulders, driftwood and sea life stay.
- With the buildings go what came with them: the companion's curiosity about them, the roofs that kept rain and snow off, and the footstep on planks.

### Fixed

- A spear no longer leaves the hand by a click. A click stabs with it; the throw key (T, rebindable on the controls page) held draws it back and its release throws it. A click held a moment used to throw the spear the owner meant to stab with.
- A spear of your own is picked up when you walk over it, and pulled out of an animal, whatever your hands hold. With a knife drawn you used to walk over your spear without taking it back.
- A spear thrown far no longer falls through the ground where it has no collider yet and flies on forever: it lands on the ground there.
- X in the pack drops a material only. On a tool it used to delete the tool for good.
- A robot that falls does not get a second kit in its spare body: the first waits in the cache where it fell.
- The camera in the ship stays where you put it when "Camera auto-follow" is off. In 0.11.0 the view around the ship sprang back to the nose 1.6 seconds after you let go of the mouse, in the cockpit and behind the ship alike, whatever the setting, and the chase camera at speed low over a planet turned toward where the ship went. Now, with auto-follow off, the angle you chose holds relative to the ship through turns, climbs out of the air, terrain in the way and a switch to the cockpit and back. R brings the view back to the nose; rebind it in Settings > Ship. With auto-follow on the view still comes back by itself.
- Leaving the landed ship works on the frame you press the key. In 0.11.0 the prompt offered "Leave the ship" while the ground beside the hatch was not loaded yet, the key then did nothing for up to 10 seconds, each press started the wait again, and the robot was finally put down on ground that had nothing to stand on, fell through it and was put back again and again. Now "Leave the ship" shows only once the ground beside the hatch is solid; until then the line says "Ground not ready." and the key keeps you seated. You step out onto that ground, on a slope too, or onto the sea.
- A robot that falls under the ground is put back once, onto solid ground. Where the ground is not loaded yet it waits still at the surface until it is, instead of falling through and coming back in a loop.
- The crouched robot looks where you look. In 0.11.0 its head pointed at the ground the whole time it crouched, still or walking, because the neck nodded it further down with the lean of the trunk instead of taking the lean back. Now the robot crouches as a hunter does: hips down and back over the heels, knees bent, the chest leaned 13 degrees for balance, and the head level, turning up, down and to the sides with the view and steady through every step. The crouch is a little shallower, so the knees and the head stay inside the body of the player. Other players see the same crouch.
- The stars show in space again. In 0.11.0 a sunlit ship, cockpit or planet hid every star and the band of the galaxy, because the camera judged the sky as if a sunlit day landscape stood around it under air. Now the stars hide behind a lit scene only as much as there is air over you: in space and on a world without air you see the whole star field past a sunlit hull, as a dark adapted eye would. In space the star now has the glare an eye sees: a bright, soft glow around its disc in the colour of the star, which fades smoothly into black space. Close to the star the glow outshines the stars, the faint ones first, with no dark ring between the glow and the star field. Where the star stands in front of the band of the galaxy, the band shows through the glow and fades smoothly into it, its dust lanes and clumps together, with no hole or notch around the star. On the ground of a world with air the day still hides them and the night shows them.
- The fog wall around the player is gone. On foot the view ended about 200 m away in a wall of horizon colour, because the fog hid the edge of the detailed ground. The fog is now the haze of the air: on an ordinary day the far hills and the sea show to about 23 km, a ridge 2 km away is already a little paler and one 10 km away clearly so, valleys and the sea hold more haze than the peaks, and the haze glows warm toward a low star. Damp air under cloud, rain and dust shorten the view for a real reason, and a dusty world is hazier than a clean one. The sea and the clouds fade by the same rule.
- The map of a planet zooms in to the ground. Its closest zoom showed about 1,200 km on a planet the size of Earth, so the ground a new player had explored was a dot under their own marker and the map read as a black rectangle. It now opens on about 5 km around the player, where the 2 km explored around the home base and the 300 m uncovered on foot fill the picture. One notch of the wheel zooms three times as far on a planet, so the whole planet is still about twenty notches away.
- Explored ground is filled on the map at every zoom. Close up it was drawn as one dot for each 500 m cell.
- A thrown spear leaves along the view. It left turned by the heading of the body a second time, so it flew true only when the robot faced north.
- The wingsuit controls keep their sense when you fly on your back. After a half loop the mouse turned the camera the wrong way round, left for right, for as long as the view was over the top with you. The turn keys turned the flight path toward the wrong wing on your back and did nothing with the nose straight up. Holding the steer button after a half loop rolled you back upside down and turned away from where you looked. The roll, pitch and turn keys now always work in the frame of your body, wherever the camera is, and the mouse always follows the screen.
- No number in the interface reads "-0.0". An empty pack said "-0.0 of 29.7 kg".
- Each frame row of the HUD has the colour of its own part: a broken part is in the danger colour and a part under half in the warning colour, also while the frame as a whole is sound.
- One press of the interact key (F) does one thing, the thing the prompt line names. Next to a carcass with a spear in it beside the ship, one press cut a part, pulled the spear and could board. Now the spear comes out first, then the cut, then the ship, and each shows on the prompt line before the press.
- The prompt line shows the key of four more actions when they are possible: "Cut the thigh" and "Pull the spear" (F), "Break stone off" (H) and "Mend the left arm" (U). The controls page names the F key "Interact, board".
- A thrown spear is never lost. When the animal it stands in leaves the world, or the world is closed and opened again, the spear falls to the ground where it last was and waits there. The pack keeps it in the save. A robot that is destroyed keeps the spears it has out.
- A blow of a big animal throws the robot more than it crushes it. The robot is a free body of 80 kg, so a blow lands with at most that mass, and a charge is as fast as its run-up allows, not at top speed from a standstill. A 300 kg animal that charges from 4 m leaves 1.2 kJ where it left 19 kJ: it breaks an arm and knocks the robot down, and no longer destroys a whole robot in one blow. A long charge of a 600 kg animal still does. A trample breaks the leg it lands on.
- An animal reads the pose of the robot of this machine, never the pose of another player seated in a ship.
- Animals react to the robot with their bodies. A hunter that goes for you stalks up, then runs the last 12 m and closes until its bite reaches, where it used to stop two metres plus its length away and stand. An animal that stands its ground faces you and goes for you once you come within 3 m of its reach, where it stood like a statue and struck only what walked into it. A grazing herd steps from patch to patch as it feeds, where its members stood still. A charge runs on the legs, which now gallop, instead of sliding the body.
- A herd no longer moves in lockstep. Each animal starts its stride, its tail swish, its wing beat or pulse and its weight shift at its own phase, so a herd spawned together no longer breathes and swishes as one. Each breath is a little longer or shorter than the last, a heavy animal breathes slower than a light one (about 15 a minute at 70 kg, 9 at 600 kg), and an animal that ran or fought pants and calms down over about 20 s. A herd cry reaches each member up to 0.4 s apart, so a herd calls as a ragged chorus instead of pumping every chest at once.
- An attack of an animal bends its body instead of sliding its parts. The head no longer sinks into the torso and the neck no longer vanishes: in the wind-up the neck draws the head back while the body crouches, then it throws the head out along its reach; a forefoot lifts and strikes, a trample lifts the forefeet and stamps, a tail cocks and sweeps. The neck and the tail now stay on the body when it crouches or rears, and the jaw opens before a bite.
- A pin lies on the ground where you drop it. It was kept at sea level under that place, so a pin dropped at your feet on ground 600 m high showed 600 m away, and you never arrived at it. Pins of older worlds and the pins of your friends find their ground when you reach their planet.

## [0.11.0] - 2026-11-21

Players and servers must both run 0.11.0: the multiplayer protocol and the world format changed. A 0.10 game cannot join a 0.11 server. Worlds from 0.10 open with 0.10 only.

### The universe at real size

- Planets, stars, orbits and galaxies have their real size. A rocky planet is 1,500 to 12,000 km in radius and a gas giant 25,000 to 70,000 km. A star has the radius its mass gives, and planets orbit at 0.3 to 40 AU by Kepler's laws. A galaxy is 25,000 to 75,000 light-years in radius, and its stars lie as far apart as real ones do. The light-year is the real one.
- Gravity follows from mass and radius: 9.81 m/s² on an Earth-like planet. The robot, the wingsuit, the boots and the ship are retuned to it: a standing jump rises as high as before, and the ship keeps its push over its weight.
- Every planet has its own spin, and its day follows from the spin and the orbit. A planet that a star or a planet pulls on hard turns one face to it. Moons form the way real ones do (impact, capture, or in the disc of a giant), stay inside the Hill sphere and outside the Roche limit, and lock to their planet by the same tide rule as planets do.
- Continents come from tectonic plates: ranges where plates meet, trenches, island arcs, shelves and coasts. Rivers drain each planet by one grid, and run to the sea or into a closed basin.
- Every planet has its own air. The gases follow from the planet: oxygen only where life makes it, carbon dioxide on dead rocky worlds, hydrogen and helium on giants, sulphur dioxide over molten ground. The air thins by a factor e over one scale height, 8.4 km on an Earth-like planet. The ship's drag, the wingsuit, the wind, the sounds and the sky all read the same curve. Clouds start where rising air condenses and end at the top of the weather layer.
- Water on far planets is rarer and more honest: ices thin inside the frost line, red dwarfs dry the planets close to them, and the warming a planet's air gives is capped by physics: no liquid water beyond the outer edge of the habitable zone.
- The galaxy is lit by one model. The band of your galaxy and every galaxy in sight come from their real star density, with dust lanes, baked into one sky map. The round smudges are gone.
- A planet is drawn by the world out to one and a half radii and by the sky beyond, with no far limit. The cloud shell follows the true sphere to 50 m.

### Where you wake up

- The home planet is chosen by one rule, the same for every planet: life that reached skeletons or beyond, built on water, with liquid seas and land, a mean temperature between 0 and 35 C and a gravity between 0.7 and 1.3 times 9.81 m/s². The game picks the nearest such planet to a Sun-like point in the middle galaxy and wakes you on a shore, with plants, herds and seas around you. The home planet, its star and its galaxy carry generated names.
- You wake in daylight. The base and the landing spot stand where the star climbs at least 25 degrees over a day and the ground is green: a mean air temperature between 0 C and 299 K, under the heat where dry land turns to desert. It holds on turning planets and on planets locked to their star. A locked planet whose lit ground is all desert is not a home.
- A tidally locked home is allowed. Many homes are worlds of red dwarfs.
- No plant grows on the landing pad square. The turf covers the rest of the base.

### Flight and the ship

- The ship is redesigned: a sleek one-seat dart with a spear nose, a faceted canopy that runs back into a spine and one tall raked fin, cranked wings, two engine nacelles with real nozzle bells, four lift nozzles under the belly and three folding legs. Plates sit over a dark frame. Every part is welded to the hull or sunk in it, a test checks that the hull is one piece, and the colliders follow what is drawn: the wing is two convex pieces, so a body on the wing meets a wall at the root. A ship far away is one cheap mesh.
- You climb in by the canopy: it lifts when you stand at the boarding spot beside the cockpit, within 7 m of the landed ship, and shuts once you sit. The pilot stays in the seat until the ground beside the ship is solid.
- The cockpit works. The stick follows pitch, roll and yaw, the throttle lever the throttle, the pedals the yaw (the boots of the pilot turn with them), the gear lever the landing gear and a switch the warp drive. The dashboard has three screens in the game font: the drive (throttle, warp, gear), the flight (speed, height, an attitude ball with the course and the nearest body) and six lamps. Every instrument reads the numbers the HUD reads and goes dark when it has nothing true to show. The hood does not hide the top row of the displays, and every display keeps its margin free. The stick and the throttle sit on pedestals beside the knees, within reach of the pilot's arms. A panel glow lights the seat at night, and a cabin light is on while you are landed. Other players' ships show the same cockpit.
- The exhaust is the engine's real exhaust. Each main engine, lift nozzle and attitude jet draws a core, a mantle and a glow that flow at the exhaust speed. In thick air the jet is short, narrow and bright with shock diamonds. In thin air and vacuum it is long, wide and faint. It rises and dies over the spool time of the engine. The warp drive has its own long violet streak. The lift nozzles blow dust off dry ground. The displays, the cabin lights and the plumes are fitted to the ship in the game itself.
- Entering an atmosphere burns. The glow follows the real heating of the nose, which grows with the root of the air density and the cube of your speed through it. Its colour follows the main gas of the air. The cockpit glass glows, and from outside a shock sheath with streaks and embers wraps the ship or the gliding robot.
- The autopilot aligns the ship with the course before it warps and never turns it in warp. It aims at where a moving planet will be on arrival. A warp to another planet of the same system engages and arrives, and the place shown follows the arrival. A route to the planet you are flying over climbs to a circular orbit 200 km up.
- The warp follows the scale. It locks only within 1.2 planet radii of a body, the speed hold flies up to twice the local orbital speed near a big planet, and a body carries your warp bubble inside half its Hill sphere. Trip times on the cards come from the warp tiers of the drive.
- Speed you can feel, from real quantities. In the wingsuit the view trembles with the dynamic pressure of the air, so it shakes in thick air at speed and not in thin air. The wind pitch follows the speed and its loudness follows the thinning of the air with height. Wingtip trails brighten in wet air. A ship that passes the speed of sound in dense wet air wraps itself for a moment in a vapour cone. Options, Controls, Speed effects scales the widening of the view, the shake and the speed lines, and turns them off at 0.
- The wingsuit answers the pitch keys about twice as fast, a banked pull turns no faster than 40 degrees a second, and a folded wing keeps you belly down in a skydiver arch. Landings match the impact: a soft one bends the knees, a hard one crouches and puts a hand down, a fast one rolls, skids or tumbles.
- Hold the run key in the water to jet with the boots: you dive, climb and turn where you look, at the speed the planet's own liquid allows. A fast or low pass over the sea leaves a foam wake in a V, ripples ring a hovering body, and spray and bubbles fly off. The robot stops kicking while the jet pushes.

### The robot and the companion

- The robot is redesigned: a small pilot in armour too big for it. A round helmet with a television face and a flat swept vane behind each ear, one flat breastplate with a centre ridge on a compact chest, a thin dark waist, a slate coat of hinged plates round the hips and bone blocks at the ends of thin dark limbs. One orange accent. The companion is the same helmet flying on its own, with a short crest faired into its crown and a nozzle under the dome.
- The robot has health by part: head, torso, each arm, each leg, each wing and each boot. A blow lands on the part nearest its contact. Damage shows as dents, scorch, sparks, a loose panel and a flickering screen, and it changes how you move: a hurt leg limps, two lost legs crawl, a hurt arm swings and throws weaker, hurt wings give less lift, hurt boots push less, a hurt head fills the view with static. The bottom left corner lists the frame integrity and each hurt part.
- Standing near the base mends every part. In the field U patches the worst part with what you carry, up to 80 percent. A destroyed robot wakes in a spare body at the base, and its pack waits in a crate where it fell.
- Hold Left Ctrl to crouch: you walk at under half speed and cannot sprint, and animals notice you at half the rate. Other players see you crouch.
- A blow from an animal shoves and staggers the robot by its momentum over your mass, and a hard one takes your footing. A block on the side of the blow cuts the push.

### Hunting, animals and plants

- Every living planet has its own life, chosen by physics: whether it lives and how far it climbed, its chemistry, and plants shaped by its climate and its star's light. Each planet has a food web of grazers, hunters and scavengers, and animals have bodies built from six plans, with legs, necks, tails, wings and fins that grow from inside the body.
- Herds graze, travel and rest around you, up to 48 at once within 150 m. Hunters stalk and run their prey down, the kill lies where it fell, and scavengers gather. Plant eaters are as heavy as on land: grazers run from hare to buffalo size, and big herds live within a walk of most open land. Animals keep off slopes their feet cannot hold. Every player sees the same herd in the same place.
- Animals notice you over time, not at a set distance, and by smell downwind. Crouching or standing still halves the rate, running and hard landings double it, and a hill hides you. Shy plant eaters run, big or cornered animals stand and call, and hungry hunters bigger than you come for you, never in your first minute on a planet.
- Animals attack with their body parts and can lose them. A hunter growls and rears before it charges. Animals brace the whole body in a wind-up, and a heavy attack leaves them off balance.
- Animals leave signs and make calls: footprints, droppings, cropped patches, nests, burrows and bones within 60 m, and calls that follow the body (low for big animals, muffled under water), with a chorus at dawn and dusk. Animals that glow light up at night.
- A field guide on K lists every species of the planet underfoot, plants first, with a count such as "12 of 31 seen". What you saw is saved with the world. Herds you have seen show on the compass and the minimap at the place you last saw them.
- You have a pack on I and a workshop on Tab in the same screen. The pack lists what you carry with its mass and wear, and carries 25 kg under Earth gravity, less on heavier worlds. The workshop proposes three tools for an intent (cut, pierce, crush, throw) from what you carry, with the one reason that matters. Six hotbar slots hold tools on 1 to 6.
- Swings and blows have weight. A clean blow goes home with a hitstop and a camera kick. A glancing blow slides off, a blunt one stops the tool dead, and a miss carries on and takes longer to bring back. Blows sound from where they land, a fast swing leaves a ribbon behind the tool, and tools wear.
- Blood is real. A cut through the hide or shell of an animal that has blood sprays along the blow, then flows as drops that fly under the planet's gravity and stain the ground, in the colour of the animal's oxygen carrier (iron red, copper blue, green, violet). A cut to a neck or a limb root pulses, a shallow cut clots, and an animal can bleed to death. Options, General, Blood turns the drops and stains off.
- A dead animal lies where it fell while it has meat. F cuts the part of the carcass you look at, and the part goes into your pack with a grade that follows how many blows the animal took. A spear can stay in an animal, and F pulls it out. Left click swings or throws, right click guards.
- H breaks a piece off the rock you look at, within 2.5 m: up to 2.5 kg, and no more than you can carry. The rock shrinks and is gone when its mass is used up. Rocks you took stay taken in the saved world.
- Grass grows on living planets that have ground cover: dense blades out to 15 m and thinning to 40 m, in the colour of the carpet under them. Farther out the same carpet shows on the ground. Trees, flowers, bulbs, mats and vines are grown from eight growth forms and every part is attached to its parent. Leaves, petals and pollen fall only from the plants around you, by wind, season and rain.
- The fixed birds, butterflies and fireflies are gone. The flying and swarming species of each planet take their place.

### Ground, sky, light and white balance

- The ground has one colour function, used by the near meshes, the far ones, the planet ball and the map, so the colours no longer change as you rise. The season tints the ground per pixel, not from the spot under the camera.
- The camera adapts to the light: the exposure follows the place, and a white balance pulls the picture toward the colour of the light over about a minute, so a red dwarf's light no longer turns the whole world salmon. Graphics, White balance turns it off.
- Surfaces take ambient light by direction, from the sky above and the ground below. Clouds shade the ground, grass and props. Water reflects 2 percent head on and everything at a grazing angle. Lamps fall off with the square of the distance.
- Graphics has a Lighting quality preset (Low, Medium, High, Ultra) and a switch for each costly effect: shadows, shadow detail and distance, ambient occlusion, bloom, cloud shadows, auto exposure, lamp shadows and sea reflections. Low keeps every light and drops only the costly effects.
- Dark patches on the ground are gone: a fading chunk no longer shadows the one that replaces it, shadows fade out instead of ending at a line, and ambient occlusion no longer haloes the chunk seams.
- The pixel look is the same on every screen: about 360 rows of square pixels, also on 4K. The Pixel size and Color depth options are removed.
- Stars and the galaxy band fade out with the brightness of the sky. The camera at the water surface splits the view along the waterline.

### The interface and the map

- A new look: translucent panels, text at 16 px or larger scaling with the window, sharp icons with no black box, no borders and a 32 px margin at the screen edge. Text and icons carry no shadow: they stay legible on a strip.
- The HUD shows only what means something where you are: on foot, swimming, gliding, in the ship landed or flying, in orbit, in open space, between stars, in warp and on the map. One line at the bottom says what the key does now, and why a course cannot go.
- Pins replace waypoints and favourites. B drops a pin at your feet, or removes the one within 10 m. You keep up to eight, they are saved with the world, and one is the active pin. Your friends see your active pin in your colour.
- Pins, the course, home, your ship and your friends show over the world at their real place with their name and distance, and slide to the screen edge with an arrow when out of view. A marker steps back when you are close to what it marks: under ten lengths of its target (60 m for home) it is a small icon without words, under four (24 m) it is gone, with a fade and no flicker at the limit. The course and the pin you follow keep their distance to the end. Near you, a friend is a name over their head. A pin you do not follow has no marker over the world past about two minutes of travel, 1 km on foot, 5 km gliding and 15 km in the ship, and stays on the compass, the minimap and the map. Markers dim in the middle of the view, never cover the compass, the minimap, the prompt or the hotbar, and when two meet the course wins and the other is drawn faint behind it. Options, General, Markers over the world has three steps: Off, Minimal (the course, your pin and friends) and All.
- The home marker is on the container, the house of the operator, on the compass, the minimap, the map and over the world. It was on open ground in the middle of the base.
- One Go command sends the ship to a place: pick it on the map, read what it is and how far, press Go. Enter and G do the same. When it cannot start, one sentence says why.
- The map card of a planet gives its chance of life, and what really lives there once you have landed. A map with no data says why. Right click a search result, a pin or a player for the same menu as on the map. The minimap shows the ground in the planet's real colours and no longer shows monuments.
- One formatter writes every distance, speed, time, mass, pull and air pressure: metres, km, AU, light-years, and gravity and air as a multiple of home.
- The map follows the real scale: a visited system reveals the stars within 10 light-years, and a system chart spaces orbits by the logarithm of the distance.
- The compass at the top shows icons with their distance on a small line under the bar, such as "340 m" or "1.2 km", and no names, which are on the map and on the markers over the world. Every mark stays at its true bearing: where two meet, the one that matters most is drawn in front and the other faint behind it, and a distance that would print over another is left out. Friends past a limit fold into the nearest one with a count. It shows the home mark only on the planet of your base.
- The map is two translucent panels inside the screen margins, so the blurred game shows around and through them. Ground nobody has seen is a dim tone of the panel, not black.
- Every fact of a place has an icon: a star (temperature, light, radius, mass, age), its system, a planet, a moon and a galaxy. Group headings, search results, pins and players have one too. A star's light and mass are given in units of the Sun.
- The side panel keeps the search, the name of the place and its Go button in view; the facts, the pins and the players scroll under them with a thin bar. The wheel scrolls the part of the panel under the pointer and zooms only over the map.
- The field guide shows a silhouette for each species, a question mark for one not seen yet, an icon for every fact, how an animal reacts to you (shy, stands its ground, hunts you) and what it yields. Its card scrolls when it is taller than the window.
- A key is shown after its label in a dimmer tone, without brackets, in every legend and on every button.
- The icon font has 132 glyphs, with their own pictures for weather, ship readouts, tools and life forms.
- The title screen is a slideshow of pictures of the game. Each one drifts slowly and the next fades in over it, and it opens on a different picture each time. The main menu sits on a translucent panel, so the picture shows at full light around it.
- F1 hides the whole interface in the world and shows it again: a clean view for a screenshot. Menus and the map still open. The key is listed in Options, Help.
- Options, General, Reduce motion stops the drift of the title pictures and changes them with a short fade.

### Multiplayer

- Protocol 3. A 0.10 game cannot join a 0.11 server, and a 0.11 game cannot join a 0.10 server. Friends must update before they join.
- Positions travel as a cell and millimetres, so players keep their millimetres at any distance from the origin.
- Friends see what you do, the same as you: crouch, held tools, swings and their ribbons, blows, stagger, wounds and blood, hits and sounds, animal attacks, carcasses, stuck spears and rocks taken. The server keeps carcasses, spears, wounds and taken rocks in `hunt3.bin` beside the world. Season, weather, clouds, herds and leaf fall come from the shared seed and clock.
- A friend's ship shows its gear lever, warp switch and dials, and the glow of its entry heat.
- The public server `space.facile.studio` starts a new world. A server that finds a 0.10 `world.toml` keeps it as `world.toml.0.10.bak`, starts a new world and keeps its key, so players still trust it.

### Saves

- Save format 3. Every world records the game that made it. Explored ground is saved as cube-sphere leaves in runs. A world made with 0.5 to 0.10 lists as "Made with 0.10" and does not open in 0.11: Play on it downloads the 0.10.1 release and opens the world with it. On macOS the game cannot do that yet.
- The upgrade code for worlds from 0.5 to 0.9 is deleted.
- The game accepts `--world <id>` to open one world at start.
- A saved place is never lost or wrong. On a server a player key has one seat: a second join under the same key closes the older session, and a connection that says nothing for 60 seconds loses its seat. The game sends its place to the server every 10 seconds. The server saves when a player leaves, and checks every stored place when the world loads. A stored height that is not a number or is out of range goes to the ground.
- Leaving in orbit brings you back in orbit. A ship beside a planet or a moon is saved against that body, so it comes back on the same orbit with the same velocity wherever the body has gone, also on a server whose clock kept running. Seated in the ship, your place is the place of the ship, in the air, in orbit or in open space. When a world starts, the ship is checked against the universe: seated inside a planet, under the deck of a gas giant or too close to a star, it is put on a circular orbit just above the air, or outside the warp lock of the star, and the banner says so. A ship left without you that cannot come back, or that is not on the body you stand on, is landed beside you. On foot you are never left in space or falling where there is no air to glide in: you start on the ground below.
- Loading always ends. A place saved in the air, in orbit or in open space loads. After a minute of loading a resumed body is stood on the ground under it, and after two minutes the game starts with what is loaded. The banner says why.

### Performance

- A world loads much faster: the turf cover of coarse and far ground is read on a small lattice, not cell by cell, and the spawn search reads each height once. Grass tiles are built off the main thread. The drainage grid of a planet is built once and shared. The cloud cover map is staged and built in tasks, and the shell coarsens from afar. The galaxy light bakes on half the cores, at most every few seconds. Animals sense every third frame, and herd and trail watches run in tasks. The shoreline for sound is read ten times a second, and the water shader skips shore foam, flecks and wake when nothing needs them. The spawn search samples the sea sparsely and skips open water. Body and lamp writes happen only when a value changes.

## [0.10.1] - 2026-11-19

Players and servers must both run 0.10.1: the multiplayer protocol changed.

### Added

- The map, on M anywhere or Tab in the ship: one level at a time, from the ground under you to the planet, its system, the galaxy and the universe. Double click a place to go in, Back or Backspace to come out, or click any level of the path at the top. The wheel and the trackpad zoom, a drag pans and drifts on, W A S D and the arrows pan. The game shows through behind it, blurred.
- A right click on the map opens a menu: set a course, go in, set a waypoint, save a favourite, rename or delete it, copy the coordinates.
- Planets on the map are flat and in colour, and only the ground someone has explored shows. On a server, ground one player uncovers is uncovered for everyone.
- Waypoints: B, or the map's right-click menu, sets one; again clears it. Your friends see your waypoint in your colour on their compass, minimap and map.
- Favourite places: save a place on the map with a name, then find it, rename it, delete it or set a course to it from the map.
- Your coordinates, altitude and the weather in the top-left corner, and the coordinates under the pointer on the map.
- North on the compass, on every planet.
- The robot on Kodya behind the main menu.
- Friends on the compass and the minimap, each in their own colour, and in the map's player list.
- A remote pilot sits in their cockpit, moving the stick, and their ship's glow and exhaust follow its throttle.
- Friends' wingsuit trails and their companions' trails, and their companions' faces.

### Changed

- The compass stays on screen, fades its marks toward its ends and hides what is behind you, like a real compass, and shows any number of marks, all on one line.
- When a course is set, one line always says what happens next: "G Engage autopilot", or exactly why the drive waits, such as "Inside 3 planet radii (now 1.4): fly out first". G engages from the cockpit without opening the map.
- The minimap range moved from M to N.
- The map's side panel card is easier to read: the place's name, what it is and what it turns around, then one fact per row under Surface, Climate, Body and Time, with the label on the left and the value on the right. Stars and galaxies get the same layout.
- The buttons for a place sit on its card, under its name: Set course, Engage autopilot, Clear course, Focus, Set waypoint and Save as favourite. When the drive cannot engage, the card says why. The line at the bottom left of the map now only shows a route in flight.
- The map's header shows the path once, with the Back button on its own row above it.
- The habitable zone reads "in the habitable zone", "too hot for water" or "too cold for water".
- The wingsuit rolls freely: hold A or D to roll over and fly on your back, S pulls toward your back and loops with the boosters, W pushes the nose down, Q and E turn gently. Let go and the wings come level by themselves.
- Below flying speed the wingsuit lowers its nose by itself, and spreading it again in mid-air carries on along your path.
- In flight the camera and the robot roll with you all the way round.
- The ship's frame reads "planet", "star system" or "between stars" instead of "orbit".
- A course ends by itself a few seconds after you arrive.
- A server takes as many players as its owner sets with `--max-players` or `ABSTRACTIONS_MAX_PLAYERS`, up to 1000 and 16 by default. Open to LAN reads `host_max_players` from the settings. The server list shows each server's real limit. Space, `space.facile.studio`, takes 64.

### Fixed

- Friends' ships no longer stutter or jump: their motion is drawn behind the measured network delay, with no freeze on landing, liftoff or far out in space.
- The minimap and the compass no longer disagree with where you are for a frame when the world shifts under you or the ship moves fast.
- You walk up stairs and small ledges without jumping.
- The ship's wings, winglets, fin and canopy are solid, and leaving the seat stops the motors.
- Unexplored ground on the map is plain black, without grid lines.
- Names on the map stay clear of the header and footer lines, and the home icon no longer covers the ship.
- The map no longer repeats the course hint while the Set course button shows.
- Wingsuit trails over the sea are no longer hidden by the water, and no clearer patch of sea follows a low camera.

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
