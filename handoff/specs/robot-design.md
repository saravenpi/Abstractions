# Robot and companion: design note (2026-11-20)

Written by the designer of the robot and the companion, before the build and corrected after the turntables. The ship designer reads this to keep the family (`ship-design.md` is its twin). Nothing here is copied from a game or an artist.

## The idea in one sentence

A small pilot in armour too big for it: a large round helmet with a television face under a brow wing, a compact chest on a thin dark waist, a slate coat of hinged plates round the hips, and heavy bone blocks at the ends of thin dark limbs.

Cute face, cool armour: the contrast is the design.

## Family rules (robot, companion, ship)

| Rule | Value |
| --- | --- |
| Plate over frame | Every bone plate sits on a dark frame part and leaves it visible at the waist, the upper limbs and the joints. |
| Joint disc | Every joint is a dark drum on the joint axis, with a bone cap where it shows: ears, shoulders, hips. A hanging plate starts at a disc and nowhere else. |
| Small chamfer | 6 mm on plates (the ship uses 3 cm at its scale). |
| Large chamfer | 15 mm on blocks: gauntlet, greave, boot, shoulder block, visor box, back plate (the ship uses 8 cm). |
| One tone per part | A chamfer has the colour of its face. Light on the edge draws the line, not paint. |
| Accent | One orange, `[0.95, 0.48, 0.16]`, the ship trim. Under 4 percent of the surface. |
| Markings | Two kinds: a triangle, and a stripe across a block near its end. Few. |
| Three layers | Bone is armour. Slate is what hangs or is soft: the coat plates, the collar, the bedroll, the cuffs. Dark is the frame. |
| Echoes | The brow wing of the robot is the cranked wing of the ship. The fin of the companion is the raked fin of the ship. |

## Palette (sRGB)

| Tone | Value | Where |
| --- | --- | --- |
| Frame | `[0.09, 0.10, 0.13]` (unchanged) | limbs, joints, waist, pelvis, visor box, soles, hands |
| Bone | `[0.93, 0.91, 0.85]` (was `[0.93, 0.93, 0.90]`) | helmet, chest, back plate, shoulder blocks, gauntlets, greaves, toe caps |
| Slate | `[0.47, 0.56, 0.64]` (was `[0.70, 0.78, 0.86]`, too close to the bone at 8 m) | tassets, collar, bedroll, helmet rim, cuffs, boot upper |
| Accent | `[0.95, 0.48, 0.16]` (unchanged) | brow wing tips, chest triangle, bedroll straps, one stripe per gauntlet, greave and tasset, the edge of the left shoulder lame |

## Proportions, standing 1.8 m (height above the sole, full width)

| Part | From | To | Width | Depth |
| --- | --- | --- | --- | --- |
| Boot | 0.00 | 0.13 | 0.17 | 0.26 |
| Greave | 0.12 | 0.43 | 0.10 at the knee, 0.16 at the ankle | 0.11 to 0.19 |
| Thigh frame | 0.47 | 0.83 | 0.09 | 0.09 |
| Tassets | 0.55 | 0.83 | 0.35 over both legs | 0.15 |
| Pelvis | 0.80 | 0.95 | 0.25 | 0.20 |
| Waist | 0.95 | 1.08 | 0.17 | 0.14 |
| Chest | 1.08 | 1.35 | 0.37 | 0.22, the back plate 0.02 more, the bedroll 0.04 |
| Shoulder blocks | 1.14 | 1.32 | 0.60 over both | 0.16 |
| Head | 1.41 | 1.73 | 0.35 (0.42 over the ear discs) | 0.30, the visor box 0.03 more |
| Brow wing | 1.66 | 1.74 | 0.46 tip to tip | 0.09 |
| Upper arm frame | 1.05 | 1.28 | 0.07 | 0.07 |
| Gauntlet | 0.80 | 1.01 | 0.10 | 0.15 |
| Hand | 0.75 | 0.85 | 0.05 | 0.07 |

The head is 0.32 of 1.8 m: more than a sixth, where a person has an eighth. The skeleton, the pivots and the segment lengths are unchanged. Widths are bounded by the player collider (0.70 m): the braced landing pose puts the wrists 0.35 m from the middle, so the gauntlets grow backward, not sideways.

## Parts and what each is for

Head
- Helmet: a twelve-sided dome, bone, slate under the rim. 0.35 wide, where it was 0.31.
- Visor box: a dark chamfered box proud of the helmet that holds the screen. The screen is unchanged, 0.23 by 0.12 m.
- Brow wing (the signature): two bone blades over the visor that meet in the middle and reach 6 cm past the helmet on each side, rolled up 14 degrees and swept back 11, with orange tips. It is the antenna, it shades the screen, and it gives the cute face a determined brow.
- Ear discs: the joint disc at each side, bone cap. Speakers and microphones.

Torso
- Waist: a slim dark frame with two bone belly lames.
- Chest shell: a chamfered loft, bone, slate under the ribs, with two breast plates that meet in a V and the orange triangle on the left breast.
- Collar: a slate ring that holds the neck.
- Shoulder housings: dark drums sunk into the chest.
- Back plate: bone, with a slate inset, two dark vents and the dark wing roots on its sides. The wingsuit unfolds from it. It is thin: seated, it stops 2 mm in front of the backrest.
- Bedroll: a slate roll across the top of the back plate under two orange straps. It is the traveller's mark and gives the side view a pack.

Pelvis
- A dark core with a ball at the waist, a bone belt plate, a front plate and a rear plate, none lower than the core, so the seat pan stays clear.
- Hip housings: dark drums.

Arms
- Upper arm: a thin dark tube and the elbow drum.
- Shoulder block: a bone block over the joint and the top of the arm, hinged at the shoulder, with the shoulder disc on its outer face. The left one has a short slate lame under it, edged in orange: the one deliberate asymmetry.
- Gauntlet: a chamfered block that deepens toward the back of the arm, bone, orange stripe, slate cuff over the wrist.
- Hand: a dark palm with a bone knuckle plate, a finger block and a thumb. The grip stays 45 mm under the wrist.

Legs
- Thigh: a thin dark tube and the knee drum.
- Tassets: three slate plates hinged at the hip: a long one over the front of the thigh, a long one on its outside with the hip disc and an orange stripe, a short one behind. Together the two legs wear a split coat.
- Greave: a chamfered block that widens to the ankle, bone, a knee cap under the knee, an orange stripe, a slate cuff. The ankle drum shows on both sides.
- Boot: dark sole, bone toe cap, slate upper, dark heel block over the nozzle.

## The coat and why nothing clips

The coat is eight plates: three tassets on each thigh, the shoulder blocks and the left lame. Each is part of the limb segment it hangs on and starts at that limb's joint, so it turns about the joint with the limb: it cannot open a gap and cannot cut its own limb. Each stays outside the widest part of the trunk at its height: the tassets start under the pelvis plates, the shoulder blocks start outside the chest. The rear tasset is short so the calf does not reach it in a deep crouch, and thin so it stays over the seat pan. No plate hangs from the pelvis below the pelvis core.

Known contacts:
- An arm raised straight overhead passes through the tip of the brow wing, as it passed through the ear of the old head.
- Seated, the lower edge of the outer tasset sinks up to 4 cm into the pan pad of the seat, and the shoulder blocks (0.60 m over both, from 0.44 to 0.62 m above the floor of the pan, reaching back to z = -1.03 in hull axes) cut about 4 cm into the side wings of the seat shell drawn in `ship-design.md` (x from 0.20 to 0.25, front at z = -1.065). The seat wings should stop under 0.44 m or stand 0.31 m out.

## Limits the game sets

Three boxes bound the armour. They were checked by hand against the poses; the tests check them again.

| Limit | Where it binds | Margin left |
| --- | --- | --- |
| Player collider, 0.35 m radius | The gauntlet in the braced landing; the knee of a running leg; the brow wing in a banked run | 3 mm, 5 mm, 13 mm |
| Backrest of the seat | The vents of the back plate and the rear plate of the pelvis | 2 mm |
| Tub of the cockpit, 0.72 m wide | The left gauntlet on the throttle | 6 mm |

## Companion (0.34 wide, 0.32 high, 0.29 deep)

The companion is the robot's helmet flying on its own: the same dome, the same dark visor box, the same ear discs, here with an orange centre. Its own signature is a fin swept back over the crown that carries the glowing antenna tip. A small bone peak sits over the visor. Under the dome a dark nozzle bell holds the thruster glow. Three slate lames hang round the lower back as a neck guard: its coat. The hands are two small gauntlets with the orange stripe and a dark cuff. It stays round and friendly: nothing points at the player.

## Budgets

| Model | Before | Near | Far |
| --- | --- | --- | --- |
| Robot, fifteen segments | 808 | 3,492 | 756 |
| Companion, body and two hands | 224 | 752 | 212 |

The far level is the same design with half the sides, no chamfers and none of the small parts. `VisibilityRange` swaps it in at 24 m for the robot and 12 m for the companion, where the two levels cover the same game pixels. One material and one draw per part, as before.
