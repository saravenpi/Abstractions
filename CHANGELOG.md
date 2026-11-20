# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Every distance, speed, time, mass, pull and air pressure in the interface is written by one formatter: metres, kilometres with spaces, million km, AU, light-years, speeds in m/s, km/s or times light speed, and gravity and air as a multiple of home. Lists share one unit and a tooltip explains ly and AU. The map header has a Home control with its key, and the pin Go button shows its key.
- The HUD shows only what means something where you are. On foot it has the compass with its cardinal points, the minimap, the place and weather, and one prompt line. In the ship it adds altitude, speed, throttle, stabiliser and, near a world, the gear; in orbit the highest and lowest point; in open space only speed, throttle and stabiliser; in warp the course and the warp bar. Elements fade in and out.
- A course you set shows on the compass as a marker, on foot too, at any distance, with its name. Pins, the course, home, your ship and friends also show over the world at their real place with their distance, and slide to the edge of the screen with an arrow when out of view. They can be turned off in the general settings.
- One prompt line at the bottom says what the key does now: why a course cannot go, what a running course is doing, the course ready to go, the board key, or a first-time hint. At most four hints, each shown until you do the thing once: open the map, drop a pin, the minimap range, the stabiliser key.
- Panels are translucent, text is 16 px or larger and scales with the window height, icons are sharp with no black box behind them, and the screen edge is a 32 px margin everywhere. Hovering a row tints it instead of inverting it. Resting on an icon button shows its name.
- The minimap shows the ground only: the white dots of monuments are gone.
- Animals prepare an attack with the whole body. In the wind-up the hips drop on the legs by a share that grows with how much the attack commits (a charge most, a bite least), the weight moves back, a trample or a downward swipe rears, a charge or a lunge ducks the head end and a tail sweep turns the flank. The blow throws the weight forward, and in the recovery the animal is off balance for as long as it stays open. The feet stay planted and the wind-up time still grows with the size of the body.
- A swing comes to one of four things. A clean blow goes home: full hitstop and camera kick, blood only if it cut. A glancing blow, at a shallow angle or against a layer too hard for the edge, slides off along the surface: a short hitstop, a light kick, a high tick, chips if the layer is hard, a small flinch and little harm, and no wound. A blunt blow the body takes stops the tool dead and sets it back a little, more for a harder contact. A miss carries on through the whole follow-through and takes longer to bring back, with no hitstop; an animal near the tip becomes more aware of you, the closer it passed.
- The robot staggers. A blow from an animal shoves you along its direction by its momentum over your mass, and your body leans away along the push and recovers, longer for a harder blow and sooner when you are crouched or blocking on that side. A weak blow only hurts. A blow hard enough takes your footing: you go down and get up again. A block on the side of the blow cuts the push by the share the tool in your hand takes.
- Blows are heard. A hit sounds from where it lands: louder with the logarithm of the energy, a dull thud on flesh and hide, a ring on a plate, horn or shell that rises with the stiffness and falls with the mass of what was struck, and a short wet noise only when the blow cut. A swing whooshes, lower and louder for a heavy tool. Cutting a part off a carcass and a spear landing in the ground make their own sound. At most three sounds play for a hit and blows keep six voices free for everything else.
- A swing leaves a short translucent ribbon behind the edge of the tool, from its real path: an arc for a slash, a line for a thrust. It shows only while the tip is fast and fades in about a tenth of a second, on your tool and on other players'.
- Blood. A blow that cuts through the hide or shell of an animal that has blood opens a wound, and the blood sprays out at once along the blow and the surface, then keeps coming as drops that fly under the planet's gravity and drag and leave a stain where they land. A blunt blow that breaks nothing open bruises and bleeds nothing, and chips fly off a horn, plate or shell it did not cut. How hard it flows follows the size of the cut, the animal's blood pressure (taller animals on heavier worlds push harder) and the time since the cut: a cut into a neck, a limb root or a torso reaches a major vessel and pulses with the heart, a shallow cut seeps and clots. The blood lost lowers the animal's health and it can bleed to death. A spear left in the wound plugs it until you pull it out. The colour comes from what carries oxygen in the animal's body on that planet (iron red, copper blue, green, violet), so one planet's animals bleed in related colours; slimes, jellies, swarms and crystals have no blood. The stains on the ground use the same colour. Options, General, Blood turns the drops and stains off; the bleeding itself goes on.
- Swings follow through: the tool carries on past the end of the blow in proportion to its inertia and settles back, the chest and hips lead the arm, and a heavy tool takes longer to bring back to carrying. A hit kicks the camera along the blow, harder for more energy, and the animal shakes along the blow.
- The robot has health by part: head, torso, each arm, each leg, each wing and each boot. A blow lands on the part nearest its contact point, and a cut harms more than a blunt hit. The bottom left corner lists the frame integrity and each hurt part. Damage shows on the body as dents, scorch, sparks, a loose panel that swings and a dimmed, flickering screen. A hurt leg limps and slows you, two lost legs make you crawl, a hurt arm swings and throws weaker and less true, a broken arm drops a two-handed tool to one hand, a hurt torso carries less and recovers slower from a swing, a hurt head fills the view and the HUD with static and shortens the field guide's reach, hurt wings give less lift and pull to the weaker side, hurt boots push less. Standing near the base mends every part for free. In the field, U patches the worst part with what you carry: something hard for a plate, sinew or resin for a joint, up to 80 %. A destroyed robot wakes in a spare body at the base, and its pack waits in a crate where it fell. Health and the crate are saved with the world.
- A pack on I, on foot. It lists what the robot carries, each row with a silhouette, a name, what it is made of, its mass and, for a tool, a wear bar. Type / to search, F2 to sort by name or mass. The robot carries 25 kg under Earth gravity, less on heavier worlds, and walks slower once the pack is past 60 % full. The pack is saved with the world.
- A workshop in the same screen, on Tab. Pick an intent with Left and Right (cut, pierce, crush, throw) and see three proposals from what you carry: the lightest, the hardest hitting and the most durable. Each shows its mass, swing speed, depth and life against the tool you hold, and the one reason that matters, such as "edge too soft for shell". H and J swap the head or the shaft material and the numbers update. Enter makes it and uses up the materials. T treats the head material where the planet allows it, such as sun-drying in strong light.
- A hotbar of six slots at the bottom of the screen. The keys 1 to 6 hold a tool; the held slot is lit and each slot shows the wear left. Pressing the key of the held slot puts the tool away or draws it again.
- H breaks a piece off the rock you look at, within 2.5 m: up to 2.5 kg of its stone (one of those that lie on the planet) and no more than you can carry. The rock keeps the rest of its mass, shrinks as you quarry it and is gone when all its mass is used up. What you took is saved with the world, so a reloaded rock is as small as you left it. Aim at a real rock: nothing happens on bare ground. Other players do not see the rock change yet.
- Blows hurt animals by their energy. A hurt animal limps and cries out, and one that dies stays dead for a while; its largest parts go into your pack, graded by how many blows it took. Every blow wears the tool that struck it.
- Hold Left Ctrl to crouch (rebind it in the controls): you walk at under half speed, cannot sprint, and your eyes sink over a quarter of a second. Animals notice a crouched robot at half the rate, hear it less and lose it behind lower ground. Other players do not see your crouch yet.
- Clouds, sky and far planets follow real size. The cloud shell is a cube sphere within 50 m of the true sphere, and the weather grid is 256 by 256 per face. A planet is drawn by the world out to a radius and a half from its centre and by the sky beyond, on depth with no far limit. The galaxy is lit by one model: the band of your own galaxy and every galaxy in sight come from their real star density, with dust lanes dimming the light, baked into a map of the sky; the round smudges are gone.
- The map's card of a planet or a moon in a visited system gives its chance of life, such as "Chance of life 72 %", from the same odds that decide what lives there. Once you have landed it says what is really there: none, microbial mats, soft-bodied life, shelled life, animals with skeletons or ancient life.
- Right click a search result, a favourite or a player in the map's side panel for the same menu as on the map.
- Animals walk the ground of living planets: the herds of the planet's own species graze, travel and rest around you, up to 48 at once within 150 m, and fade in and out at the edge. Every player sees the same herd in the same place.
- Animals notice robots, yours and your friends': shy plant eaters look up, then run from you and come back to their herd once you are gone. Hunters stay bolder.
- Animals notice you over time instead of at a set distance: in a third of a second under 10 m, a second and a half at 10 to 30 m, four seconds at 30 to 60 m. Standing still or creeping halves the rate, running, boots and hard landings double it, and a hill between you hides you. Downwind they smell you from 80 m, hunters from 120 m. One lookout per herd sees farther, and when it runs the whole herd runs. Plant eaters flee anything over half their size; big or cornered animals stand and call instead, and hungry hunters bigger than you walk up to you, though never in your first minute on a planet. A badly hurt animal limps to its den and leaves blood every two metres.
- Hunts play out where you can see them: hunters stalk and run their prey down, the kill lies where it fell while it has meat, and scavengers gather round it.
- A field guide on K, on foot: every species of the planet underfoot, plants first, with a count such as "12 of 31 seen". A plant counts as seen once you stand on its planet, an animal once one comes within 40 m. Pick one with the arrows, the wheel or a click to read its body, diet, herd, hours, habitat, and what it eats and what eats it. Species you have not seen show as unknown. K or Esc closes it. The key can be changed in the options. For now the guide remembers what you saw until you leave the world.
- Herds you have seen show on the compass and the minimap with a small animal mark at the place you last saw them, labelled "Herd" until the field guide has their species, then by its name. A herd counts as seen once it is in view within 150 m with no hill between you; the mark does not follow the herd and fades out over three minutes. The three last seen show, one for herds that keep together.
- Animals that glow light up at night in their own colour.
- Animals leave signs and make calls. Within 60 m you find the footprints of herds that passed in the last four minutes, spaced wider where they ran, droppings and cropped patches where they grazed, nests and burrows near their homes, and bones where a kill was eaten. Herds call now and then, lower for bigger animals, call the alarm when they flee, and hunters cry as they give chase. Every player finds the same tracks and hears the same calls.
- Grass grows on living planets that have ground cover. Near you, blades stand dense out to 15 m and thin out to 40 m, sway in the wind and take the colour of the carpet under them. Farther out the ground shows the same carpet: two colours of the planet's ground-cover species in patches with a fine speckle. Blades stay clear of trunks and rocks and of the patches grazers cropped. A planet without ground cover has none.
- Hold the run key in the water to jet with the boots: underwater the robot dives, climbs and turns where you look, and at the surface it skims along the water. The boots push with the same power as in the air, through the planet's own liquid: about 8 m/s in water, slower in molten rock, nothing in a frozen sea.
- Wind from a wingsuit, boots or ship tears spray off the sea: low over the water, or fast and low skimming it, flecks fly up and fall back, thrown per metre you travel so they look the same at any frame rate.
- A fast pass low over the sea leaves a foam wake that spreads behind you in a wedge and fades in a few seconds.
- Hovering low over the sea rings it with ripples that run outward at the speed of waves under the planet's gravity.
- The water jet of the boots leaves a trail of bubbles that rise to the surface.
- The boot water pump thumps, faster as the boots spool up.
- Entering an atmosphere from space now burns. The glow follows the real heating of the nose, which grows with the root of the air density and the cube of your speed through it, so thick air glows early and hard, thin air faintly and late, and a slow entry or level flight not at all. Its colour follows the air of the planet: pink-orange to white over wet worlds, blue-white to violet over dry ones, pale pink to magenta over giants, teal over cold frozen worlds, yellow over molten ones. The cockpit glass glows with plasma streaming past, and from outside, in third person, a shock sheath with plasma streaks, embers and a short tail wraps the ship or the gliding robot. It fades as you slow down and blends into the cloud animation without a step.
- A route to the planet you are flying over takes you up to a circular orbit 200 km above it: engage the course to that planet while you are in its air, and the autopilot climbs, turns over above the air and lets go in orbit, in about a minute on a planet like the Earth.

### Changed

- A new player starts in third person on foot, and in the cockpit in the ship. The game keeps the last view you chose for each.
- The robot stops kicking while its boots jet it through the water: its legs stay straight behind it, and the kick comes back as the jet winds down.
- Leaves, petals and pollen fall when the world says so. In calm summer air a tree loses almost nothing. Gusts tear leaves off in small bursts, harder the stronger the wind and the thicker the air, and rain and storms knock more down. Deciduous trees shed heavily in autumn and are bare in winter, evergreens drop a slow trickle all year, and dry plants shed more. Petals fall only while a plant flowers, pollen only on dry days with some wind. Leaves sink at the speed their size, the air and the planet's gravity give and drift downwind.
- The wake of a fast or low source is a V: foam arms at the wedge angle of its speed, size and the planet's gravity (19.47 degrees when slow, narrower when fast), with a turbulent band between them. The sea surface itself carries the wake as moving waves, arms, transverse waves and ripple rings, and the foam rides them.
- The ship scales with the planets. The warp locks only within 1.2 planet radii of a body, the speed hold flies up to twice the local orbital speed near a big planet, and a body carries your warp bubble inside half of its Hill sphere instead of a fixed number of radii.
- Gravity is a real 9.81 m/s² on an Earth-like planet, and the robot is retuned to it: a standing jump rises as high as before, the wingsuit, the dive and the boot jet fly at the same speeds, and the ship's thrust keeps its push over its weight.
- Air follows physics. Every planet has its own gases: nitrogen with carbon dioxide where liquid water locks carbon into rock, oxygen only where life makes it, carbon dioxide on dead rocky worlds, hydrogen and helium on gas giants, nitrogen and methane on cold worlds, sulphur dioxide over molten or volcanic ground. The gases, the temperature and the gravity set how fast the air thins with height: by a factor e over one scale height, 8.4 km on a planet like the Earth, with the top of the atmosphere 12 scale heights up. The ship's drag, the wingsuit, the wind, the sounds of the air and the sky all thin the same way now, and cold or heavy air is thicker than its pressure alone says. Clouds start where rising air condenses and end at the top of the weather layer, about 1 to 11 km on a planet like the Earth.
- The minimap shows the ground of a planet in its real colours, from the same palette as the map, with its relief shading.
- Falling leaves, petals and pollen come only from the plants around you. A tree in leaf drops leaves of its own colour and size, more in autumn; a plant in flower drops petals and, by day, pollen; a bare tree and a lifeless planet drop nothing.
- Animals move when they call: the head lifts, the jaw drops on every note, and the chest or the bell swells. Bigger animals call slower. Every player sees the same animal call at the same moment.
- Animals have more to say. A hunter growls and rears up, head low, just before it charges, and hunters and horned animals do the same when you come too close. A caught animal cries in distress, and young call back to their parents. Herds call more around sunrise and sunset, in a dawn and a dusk chorus.

### Changed

- Falling leaves, petals and pollen come only from the plants around you. A tree in leaf drops leaves of its own colour and size, more in autumn; a plant in flower drops petals and, by day, pollen; a bare tree and a lifeless planet drop nothing.
- The minimap shows the ground of a planet in its real colours, from the same palette as the map, with its relief shading.
- Calls follow the body: animals that breathe air moan, hoot, whistle, growl or hiss, shelled ones rasp and click, soft ones gurgle, and calls from under water sound muffled. Long, thin bodies and long necks call lower.
- The map's side panel has more room: Search, Place, Favourites and Players each sit under a heading, with space between them and between the groups of the card. The result count sits beside the Search heading, the save button beside the Favourites heading, and the distance only in the result row. Rows are taller, and in a short window the favourites and players lose their last rows before anything else is cut.
- The buttons on the card are shorter, so they take two rows: Set course or Engage and Clear, Focus, Set waypoint and Save place.
- The compass shows the home mark only while you are on the planet of your base, on foot or flying in its air. In space and on other planets it is gone.
- The wingsuit answers the pitch keys faster: a pull bends the flight path about twice as fast at speed, and let go the wings fly on where you pointed them instead of snapping back to the glide.
- A banked pull turns no faster than 40 degrees a second, and slower at speed: rolled on edge, the wingsuit drifts round instead of spinning.
- Folding the wings in the air keeps the body prone and turns it into a skydiver arch, belly down, arms and legs spread, head up. A fast fall from a jump arches too, and the body turns upright and reaches for the ground just before it lands.
- A body falling belly down has twice the drag of one diving head first, so it falls slower.
- The home planet, its star and its galaxy carry generated names like every other place.
- Landings match the impact. A soft one bends the knees as before. A hard drop crouches deeper, puts one hand and then both on the ground and gets up slower. A fast touchdown with the gear rolls over the shoulder, forward, backward or over the side, one turn over the distance the roll covers. A shallow fast one skids on the feet, leaning back, or on the belly with the arms out after a glide. A crash at 60 m/s tumbles with the limbs flailing, then the body gets up.
- Plant eaters are about a hundred times heavier than before, as on land the biggest animals eat plants: a planet's grazers now run from hare to buffalo size, and their herds are easy to spot.
- Big game is common: herds of plant eaters at least 0.8 m at the hip, in herds of 5 to 30, live within a walk of most open land on planets with animals with skeletons. Population follows body mass (big animals are rarer, but their herds roam far), grazers keep to open meadows and browsers to woods, and a niche too heavy for a shell or a swarm is filled by an animal with a skeleton.
- Four-legged animals stand on longer legs, as mammals do: a 75 kg grazer stands about 0.8 m at the hip, a 500 kg one about 1.5 m.
- Legs, necks, tails, wings and fins grow from inside the body instead of touching its outline through a thin joint, and thighs widen into a haunch where they join it.
- Animals no longer walk up cliffs. Each body holds a slope as steep as its feet grip, about 39 degrees on pads, more on many clawed legs, less for slitherers; herds walk around ground steeper than that, and an animal led at a cliff follows its foot or stops.
- Animals let you nearer: a calm grazer looks up at about 20 m and runs once you walk within about 11 m. Small animals let you come closer, hunters closer still, and an animal lets a robot that stands still come nearer than one that walks.
- Flying species circle in flocks 8 to 30 m over the ground, and swarms hover low over the plants.
- The pixel look is the same on every screen: the game always shows about 360 rows of pixels, and every pixel is the same square of screen pixels, also on high-density and 4K screens. Before, a bigger window showed more detail and some rows and columns of pixels came out wider than others.

### Removed

- The fixed birds, butterflies and fireflies. The flying and swarming species of each planet take their place, and dead planets have none.
- The Pixel size and Color depth options. The pixel size now follows the screen, and the colours keep their 24 levels.

### Fixed

- Dark patches on the ground. A chunk that fades out no longer shadows the one that replaces it. The ground light is corrected for shadows, so it is exactly the sun light at the camera and never relights a shadow. Shadows end in a fade over the last fifth of the shadow range instead of at a line. Ambient occlusion fades out between 30 and 90 m and is taken off a camera more than 90 m above the ground, so chunk seams no longer halo. The moon casts a shadow only when its light is strong enough to show one. The lights now update in the same step as the light of the place, with no frame of lag.
- The autopilot aligns the ship before it warps and never turns it in warp: the drive waits until the nose is on the course and has stopped turning, the course aims at where a moving planet will be on arrival, and a heading that would miss the destination drops the drive out to align again.
- The windows of the ship are clear glass instead of blue: nearly colourless and see-through, with a faint sheen at a slant, so the sky, clouds and water show their own colours through it.
- The minimap in space follows your ship every frame, including in warp and when leaving a system, instead of jumping five times a second.
- In the system view of the map only the home planet is labelled home; the star of the home system no longer is. The view of stars still marks the home star.
- The map no longer shows blue seas on every world: the liquid of a planet picture is the liquid the world draws, so a lava planet has a sea of molten rock, a dry planet has none, and the minimap, which shares the picture, follows.
- The camera at the water surface splits the view along the waterline, with a thin dark line where the surface crosses the lens: only what is seen from under the water is fogged, and the line follows the waves. The camera never rests on the surface itself.
- In third person the head no longer vanishes when a wall pushes the camera close to the robot. It only hides when the camera is inside the head.
- No flash of water at eye level in front of the base door as the boot screen fades: the warm-up copy of the sea was drawn 30 m wide in front of the camera.
- No more floating parts on the plants of living planets. The top clump of a tree sits on its trunk, thorns and resin stay on a leaning trunk or stalk, fruit hangs from its flower, and the flowers of trees, bulbs, mats and vines sit on their leaves and stems. The spines of a rosette grow on its blade tips. On a slope, mats, side shoots, bulbs and kelp reach down into the ground instead of hovering over its low side, and the caps of a leaning stalk follow it.

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
