# Robot and companion: design note (2026-11-20)

Written before the build by the designer of the robot and the companion. The ship designer reads this to keep the family; nothing here is copied from a game or an artist.

## The idea in one sentence

A small pilot in armour too big for it: a large round helmet with a television face and a swept topknot fin, a compact chest on a thin waist, heavy gauntlets and greaves at the ends of thin dark limbs, and hinged plates hanging from the shoulders and the hips that draw a coat line.

Cute face, cool armour: the contrast is the design.

## Family rules (robot, companion, ship)

| Rule | Value |
| --- | --- |
| Plate over frame | Every light plate sits on a dark frame part and leaves it visible at the joints. |
| Joint disc | Every joint is a dark drum on the joint axis. A hanging plate starts at a disc and nowhere else. |
| Small chamfer | 6 mm on plates and markings (ship: scale by 8, about 5 cm). |
| Large chamfer | 15 mm on blocks: gauntlet, greave, boot, visor box, pack (ship: about 12 cm). |
| Accent | One orange, `[0.95, 0.48, 0.16]`, the ship trim. Under 4 percent of the surface. |
| Markings | Three kinds only: a triangle, a stripe across a block near its end, a row of three studs. |
| Tones | Bone plate, slate secondary, dark frame. Flat areas, hard breaks. |

## Palette (sRGB)

| Tone | Value | Where |
| --- | --- | --- |
| Frame | `[0.09, 0.10, 0.13]` (unchanged) | limbs, joints, waist, visor box, soles, hands |
| Plate | `[0.93, 0.91, 0.85]` bone (was `[0.93, 0.93, 0.90]`) | helmet, chest, gauntlets, greaves, hanging plates |
| Secondary | `[0.47, 0.56, 0.64]` slate (was `[0.70, 0.78, 0.86]`, too close to the plate at 8 m) | pack, collar, calf, helmet rim, boot upper |
| Accent | `[0.95, 0.48, 0.16]` (unchanged) | ear discs, fin tip, one stripe per gauntlet and tasset, chest triangle |

## Proportions, standing 1.8 m (height above the sole, full width)

| Part | From | To | Width | Depth |
| --- | --- | --- | --- | --- |
| Boot | 0.00 | 0.13 | 0.15 | 0.26 |
| Shin greave | 0.13 | 0.43 | 0.10 at the knee, 0.14 at the ankle | 0.16 |
| Thigh frame | 0.47 | 0.83 | 0.09 | 0.09 |
| Pelvis | 0.80 | 0.95 | 0.25 (0.35 over the tassets) | 0.19 |
| Waist | 0.95 | 1.08 | 0.17 | 0.14 |
| Chest | 1.08 | 1.35 | 0.34 (0.53 over the shoulder discs) | 0.21, pack 0.05 more |
| Head | 1.43 | 1.72 | 0.31 (0.37 over the ear discs) | 0.30, visor box 0.03 more |
| Fin tip | | 1.79 | 0.03 | overhangs the back of the head by 0.08 |
| Upper arm frame | 1.05 | 1.28 | 0.07 | 0.07 |
| Gauntlet | 0.82 | 1.00 | 0.10 | 0.12 |
| Hand | 0.75 | 0.85 | 0.05 | 0.07 |

The head is 0.29 of 1.8 m: one sixth, where a person has one eighth. The skeleton, pivots and segment lengths are unchanged.

## Parts and what each is for

Head
- Helmet shell: twelve-sided dome, bone, slate under the rim.
- Visor box: a dark chamfered box proud of the helmet that holds the screen. The face stays 0.23 by 0.12 m.
- Brow hood: a bone plate over the visor box, 13 mm proud of the screen. It shades the screen and gives the helmet a peak.
- Ear discs: the joint disc at each side, orange centre. They are the speakers and microphones.
- Topknot fin (the signature): a 28 mm blade from the crown, swept back and up to overhang the back of the head, orange tip. It is the antenna.

Torso
- Waist: a slim dark frame, two bone belly lames.
- Chest shell: chamfered loft, bone, slate under the ribs.
- Chest plate: a proud bone plate with the orange triangle on the left.
- Collar: a slate ring round the neck housing.
- Shoulder housings: dark drums sunk into the chest.
- Pack: a slate block on the back with two dark vents and the dark wing roots on its sides. The wingsuit unfolds from it.

Pelvis
- Dark core with a ball at the waist, a bone belt plate, a front plate and a rear plate, none lower than the core, so the seat pan stays clear.
- Hip housings: dark drums.

Arms
- Upper arm: thin dark tube, elbow drum.
- Shoulder plate: a bone plate hinged at the shoulder disc, on the outside of the arm. The left one has two lames and is the one deliberate asymmetry.
- Gauntlet: a chamfered block that widens to the wrist, bone with an orange stripe.
- Hand: dark palm, finger block and thumb. The grip stays 45 mm under the wrist.

Legs
- Thigh: thin dark tube, a slim bone plate on the front, knee drum.
- Tasset: a long bone plate hinged at the hip disc, on the outside of the thigh, with three studs.
- Greave: a chamfered block that widens to the ankle, bone front and slate calf, a knee cap, an orange stripe.
- Boot: dark sole, bone toe cap, slate upper, dark heel block over the nozzle.

## The coat hint and why nothing clips

The coat line is four plates: two shoulder plates and two tassets. Each is part of the limb segment it hangs on and starts at that limb's joint disc, so it turns about the joint with the limb and can never open a gap or cut the limb. Each lies only on the outer side of its limb, beyond the widest part of the trunk at that height. The tassets span the thigh's own depth, so seated they do not reach under the thigh. No plate hangs from the pelvis below the pelvis core.

## Companion (0.34 wide, 0.32 high, 0.29 deep)

The companion is the robot's helmet flying on its own: the same dome, the same dark visor box and brow hood, the same ear discs with an orange centre, and the same fin swept back, here carrying the glowing antenna tip. Under it a dark nozzle bell holds the thruster glow. Three bone lames hang round the lower back as a neck guard, its coat hint. The hands are two small gauntlets with the orange stripe. It stays round and friendly: no point faces the player.

## Budgets

| Model | Today | Near | Far |
| --- | --- | --- | --- |
| Robot segments | about 800 | at most 3,500 | at most 800 |
| Companion body and hands | about 280 | at most 1,000 | at most 280 |

The far level is the same design with fewer sides, no chamfers and no markings, swapped by `VisibilityRange`. One material and one draw per part, as today.
