# Ship design: the blade with a crest

One sentence: a sleek one-seat dart that takes off vertically, flies in air, reaches orbit, warps and re-enters nose first; a spear nose with a sharp chine, a low faceted canopy that runs back into a dorsal spine, and that spine sweeping up into one tall raked fin, which is the signature of the silhouette; rows of overlapping plates at the wing roots and round the engine nacelles are the hint of layered armour.

Axes: nose along -Z, roof along +Y, right wing along +X, origin at the middle of the hull. Metres.

## Family rules (shared with the robot and the companion)

| Rule | Value |
| --- | --- |
| Plates over a frame | Light plates stand 3 cm proud of a dark frame. A gap of 4 to 6 cm between two plates shows the frame. No painted line: every panel break is a step. |
| Chamfers | Two sizes only: 3 cm on plates and small parts, 8 cm on blocks (pods, the seat shell, consoles). |
| Tones | Bone plates `srgb(0.86, 0.86, 0.82)`, a cool grey second tone, a charcoal frame `srgb(0.14, 0.15, 0.18)`. |
| Accent | The robot's `ACCENT`, `srgb(0.95, 0.48, 0.16)`. On the ship it covers under 3 percent of the surface. |
| Markings | Geometry or colour blocks, few: one stripe, one slanted bar, a row of three studs, a two-bar tally. No text on the hull. |
| Lames | Three plates in a row, each overlapping the next by a few centimetres like the plates of a shoulder guard. |

## Size

| Thing | Value |
| --- | --- |
| Length | 8.15 m: nose tip at z = -4.45, nozzle exits at z = 3.7 |
| Span | 6.4 m |
| Height on its gear | 3.25 m at the fin tip; the belly is 0.6 m off the ground |
| Fuselage at the seat | 1.7 m wide at the chine, 1.1 m high |
| Canopy roof over the head | y = 1.28, the pilot's eye at y = 0.9: 19 cm over a 19 cm head, room for a bigger helmet |
| The robot | 1.8 m standing: its head is level with the canopy sill; seated it takes 0.72 m of width and 1.3 m of length |

## Parts and what each is for

| Part | Purpose | Shape |
| --- | --- | --- |
| Fuselage | Lifting body and heat shield carrier | Eight-corner section with a low sharp chine, a flat belly and sides that lean in to a narrow back |
| Nose | Speed, and the entry shock | A flat spear, chine out to the tip, drooped 0.17 m |
| Canopy | The pilot's view and the way in | Faceted trapezoid glass, raked windscreen, on two slim bows and a sill; hinged at the top of its rear hoop, it lifts 57 degrees |
| Spine | Carries the canopy line to the fin; holds the canopy hinge and the antenna | Opaque fairing behind the canopy, tapering back |
| Fin | Yaw stability; the crest | One blade, 2 m above the middle of the hull, swept 56 degrees, tip raked past the nozzles |
| Wings | Lift in air | Cranked arrow: a long strake from the chine, a swept panel, a trailing edge swept forward; the rear quarter is the flap |
| Tip blades | Roll stability; they stop a body walking off the wing | Swept blades through the wing tips |
| Nacelles | The two main engines, 220 kN each | Round pods sunk in the rear flanks, three lames round each, a real bell with a lip and a lit chamber |
| Lift nozzles | Vertical take-off, 92 kN each | Four bosses under the belly plate |
| Attitude ports | Fourteen cold-gas ports | Dark sockets in the skin |
| Heat shield | Entry | A charcoal slab under the belly |
| Gear | Three legs | A sleeve, an oleo piston, a drag brace and a long skid foot. Folded, the foot closes its own bay: the foot is the door |
| Wing root lames | Armour hint; they cover the wing joint | Three overlapping plates on each wing root, on a dark plate of the frame that shows round them |
| Rung and kick steps | How the pilot climbs in | A rung under the left chine, 0.73 m off the ground, and two dark kick steps in the flank above it, under the sill |
| Service panel | Honest access to the avionics | A flush panel on the left flank of the nose, with the two-bar tally |

## Palette

| Tone | Where |
| --- | --- |
| Bone | The back, the upper wings, the fin, the nacelles |
| Cool grey | Every second lame, the flanks under the chine line, the flaps, the spine collar |
| Charcoal frame | Belly, heat shield, bells, gear, canopy frame, the gaps between plates |
| Accent | The stripe on the fin, one slanted bar on each wing root, the tally of the service panel, the seat harness, the grips of the controls |

## Boarding

The canopy opens: hinged across the top of its rear hoop, it lifts at the front while the player stands at the boarding spot beside the cockpit on the left, in front of the wing. The pilot steps on the rung under the chine, climbs the two kick steps and steps over the sill into the seat. The old flank hatch is gone. The canopy opens for a player within 7 m of the landed ship and shuts once the player sits.

## Cockpit

- The tub is 0.76 m wide and 1.3 m long under the canopy. The floor is 0.4 m under the seat pan.
- Seat: a dark shell with a headrest and low side wings that stop 0.42 m up, under the shoulder blocks of the robot; three stacked back pads like lames; a pan pad 0.27 m wide between two thigh bolsters, so the plates that hang outside the thighs hang beside the pad; an accent harness.
- Dashboard: under the rim of the hull so the nose stays in view. One panel faces the eye, 0.62 m wide and 0.34 m tall, under a thin hood whose rear edge is behind and over the top of the panel, so the whole of every display shows from the eye (a test casts the rays). Every display keeps a margin of 2 pixels on its four edges, and a value too long for it loses the space between its thousands instead of being cut. Three displays, each in a bone bezel: the drive on the left (throttle, warp, gear as bars), the flight in the centre (speed, height, the attitude ball with the course and the nearby body marked on it), the status on the right (six lamps with legends). The top third of the panel, with the speed and the height, shows when the pilot looks level; the rest when the pilot looks down.
- Text: Undefined Medium is drawn on a grid of 100 font units, five columns by seven rows, with a side bearing of half a cell. A display pixel is 4 mm, about one pixel of the 360-row game image from the seat, so each display paints the font one grid cell to one pixel: the exact glyphs of the game font at their native size (the 68 glyphs of `cockpit/font.rs` were compared with the outlines of `assets/fonts/undefined-medium.otf`, cell by cell). The images are sampled linearly, because nearest sampling at one to one drops a stroke wherever a display pixel is a little smaller than a screen pixel. A UI camera rendering into a texture would add a render pass per ship and resample the same glyphs at this size.
- Each side console is high at the front, where the gear lever and the warp switch are on the right, and low beside the seat. A pedestal stands in from each console beside the knee and carries a short side stick on the right and the throttle on the left, hinged 0.23 m from the middle (the stick inside its pedestal, 6 cm under the top) and held at a height of 0.36 m: 0.22 m under the shoulders and in line with them, so the elbows stay inside the tub (the widest point of the seated robot is 0.344 m from the middle, the limit is 0.36 m and the wall is at 0.39 m; checked with the Python port of the rig in `~/.cache/abstractions-targets/lm-check`). The hand moves 4 cm either way on the stick and 10 cm on the throttle. Two pedals. The hands are below the view when the pilot looks level and show when the pilot looks down or to a side.
- Walls: ribs, a cable run along each side under the sill, a heel plate on the floor.
- Light: the displays glow; a cool panel light and a warm cabin light, both without shadows.

## Budgets

Near about 3,200 triangles outside and 1,300 in the cabin, far 911, no new material. The cabin draws nine meshes fewer than before: the six lamps and the black panel face are painted into the three displays.
