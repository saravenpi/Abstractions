# Hunting, combat and crafting: research and plan

Written 2026-11-20 from three research passes (combat animation, hunting, crafting), each reading primary sources. This plan refines roadmap 0.13 ("hunting, mining and crafting"); where they differ, this file wins. It is a plan only: no code starts until the owner approves it.

Tags: **[V]** verified by two or more independent sources, **[S]** one source, **[I]** our inference.

## The short answer

- **Hunting is four steps:** read the herd, approach unseen, land a clean hit, harvest by body part. Every sense an animal uses is visible to the player. Tissue quality comes from skill, not luck.
- **Combat stays kinematic, not physics-driven.** A swing is an arc of the weapon around the grip, written into the robot's existing pose layers, with IK hands on the grip. Physics only drives hit reactions. Physics-driven fighters cost studios years of tuning or motion capture plus training.
- **Weapons animate themselves.** Swing speed follows the weapon's moment of inertia about the grip, `v = C / I^0.27`, a measured sports-physics law. Any weapon the player builds swings plausibly with no hand-made clip.
- **Crafting answers an intent with physics.** Damage follows Dwarf Fortress's momentum-against-layer tests; tissues carry real material numbers; treatments are trade-offs; every build says why it is good or bad.
- **Creature attacks come from their bodies.** Horns charge, tails sweep, long necks lunge, claws swipe, mouths bite, heavy bodies trample, with wind-ups that grow with size.

## What the research found

### Combat feel and animation

- **Physics-driven animation is expensive.** Gibbon spent about 1.5 years tuning one scoring function [S] https://www.gamedeveloper.com/programming/deep-dive-physics-based-animation-in-gibbon-beyond-the-trees. Ubisoft La Forge's DReCon needs a 10-minute mocap database and about 30 hours of training per character type [V] https://theorangeduck.com/media/uploads/other_stuff/DReCon.pdf. Neither fits procedurally generated bodies and weapons [I].
- **Few keys, procedural blending works.** Overgrowth's combat used very few key poses blended procedurally [S] https://www.gamedeveloper.com/design/video-an-indie-approach-to-procedural-animation. Spore wrote moves relative to body parts ("move limb near the head, then to the enemy's nose"), and creatures without hands used the mouth [S] https://spectrum.ieee.org/engineering-spore. This maps onto our part trees [I].
- **Swing speed from inertia.** Maximum swing speed follows `V = C / I^0.27` for I above 0.03 kg·m²; with I held fixed, speed stays about the same as mass changes [V] (Cross and Bower via https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=DOI:%2210.1080/02640410500127876%22&resultType=core&format=json, confirmed by Schorah et al. https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=DOI:%2210.1080/14763141.2015.1027949%22&resultType=core&format=json). Bannerlord's crafted weapons work on the same idea: swing speed depends on how weight is spread around the pivot, damage on kinetic energy lost at the actual contact point, and part of it kicks back into the hand [S] https://www.gamebanshee.com/k43ha and https://www.gamebanshee.com/k4awc.
- **Directions.** Mount & Blade reads the swing direction from the mouse just before the click [V]; For Honor blocks by bracing on the attack's side, with exact-timing parries [S].
- **Hitstop.** Smash freezes attacker and victim for the same time, scaled by damage (Smash 4: `d·0.385 + 5` frames, capped at 30) [S] https://www.ssbwiki.com/Hitlag. The victim shakes, starting big and shrinking. Monster Hunter Wilds cut hitstop, then restored it after beta feedback [V, search summaries].
- **Creatures.** Rain World builds AI from attachable trackers (threat, prey, friends, den) with a relationship table per creature [S] https://unity.com/blog/exploring-procedural-design-rain-world. Monster Hunter designs a monster's body around its signature attack and cites limping away as part of the realism [S].

### Hunting

- **Monster Hunter's loop** is hunt, take parts, craft gear; skill decides the result, and co-op rewards go to everyone equally [S].
- **Horizon's herds** keep grazers in the centre with lookouts and fighters on the edge; under attack the herd splits into fleeing and fighting groups; parts break off and drop as resources, colour-coded [S].
- **Splinter Cell Blacklist (GDC 2014):** detection builds up over time rather than firing instantly; stance changes visibility; sound distance follows paths around walls; spoken lines tell the player what was noticed. The goal is plausible, not intelligent [S].
- **Red Dead Redemption 2** ties perfect pelts to clean kills on pristine animals; players found the grades confusing and the rewards weak [S]. Lesson: show the grade before the shot [I].
- **Multiplayer:** lockstep needs bit-identical maths across machines, which floating point makes nearly impossible to guarantee [S]. Valheim reportedly lets the nearest player's machine run the creatures, so one weak machine desyncs everyone near it [community sources].

### Crafting

- **Dwarf Fortress damage:** an edged attack cuts a layer only if momentum beats `(rSY + (A+1)·rSF)·(10 + 2·Qa) / (S·Qw)`; when it fails, the hit turns blunt for that layer; small contact area stabs, large slashes; past a threshold, weapon weight matters little [V] https://dwarffortresswiki.org/index.php/Material_science and https://dwarffortresswiki.org/index.php/Weapon.
- **Real tissue numbers:**

| Material | Stiffness E (GPa) | Strength (MPa) | Toughness K_IC (MPa·m½) |
|---|---|---|---|
| Bovine femur | 17.7 | 99 | 6.35 |
| Antler, wet / dry | 7.5 / 17.1 | 108 / 188 | up to 10.3 |
| Horn keratin, wet / dry | 0.85 / 2.3 | 40 / 154 | 2.56 / 3.86 |
| Insect cuticle | 3.05 | | 4.12, lower when dried |
| Glass (obsidian stand-in) | | | 0.7 to 0.8 |

  Sources: https://en.wikipedia.org/wiki/Antler [S], https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=EXT_ID:20086133%20AND%20SRC:MED&resultType=core&format=json [V], https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=EXT_ID:22496286%20AND%20SRC:MED&resultType=core&format=json [V], https://en.wikipedia.org/wiki/Fracture_toughness [S], https://en.wikipedia.org/wiki/Obsidian [S].
- **Treatments are trade-offs:** fire-hardening wood makes it slightly harder but cuts strength by 30% and work of fracture by 36% [V] https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=EXT_ID:27194289%20AND%20SRC:MED&resultType=core&format=json; heat-treated stone loses toughness and gets easier to knap [V].
- **Systems:** Breath of the Wild's chemistry rests on three rules (elements change materials, elements change each other, materials never change each other) [S] https://www.thumbsticks.com/gdc-17-breath-of-the-wild-science-lies. Veloren keys a modular weapon by main part, material and hands [S] https://docs.veloren.net/veloren_common/comp/inventory/item/modular/index.html. Minecraft stores only what differs from an item's defaults [S] https://minecraft.wiki/w/Data_component_format.
- **Choice:** decision time grows with `log₂(n+1)` choices (Hick's law) [S] https://en.wikipedia.org/wiki/Hick%27s_law, which supports three proposals, not a list of fifty [I].

## The design for Abstractions

### Hunting

- **Awareness meter (0 to 1)** per animal: fills in about 0.3 s under 10 m, 1.5 s at 10 to 30 m, 4 s at 30 to 60 m; crouching halves the rate, moving doubles it; it drains at about 0.1 per second. Shown through head turns, ears and calls, and a subtle marker, not a big bar.
- **Scent and wind:** a downwind cone about 40° wide, 80 m for grazers, 120 m for hunters. The wind is visible (particles, the glider ribbon).
- **Herd roles:** one lookout per herd with 1.5× sense range; its alarm sends the whole herd running.
- **Fight or flight:** a creature fights if `own mass × health / threat mass > 0.5`, or when cornered or guarding young; grazers flee anything over half their size; hunters attack when bigger and hungry; scavengers follow the wounded.
- **Wounded:** below 40% health an animal limps at 0.6× speed, leaves blood every 2 m (reusing the 4-minute footprint fade) and heads for its den.
- **Tissue grade:** species base × kill quality; one weak-point hit is clean, two or three is good, more or blunt is poor. The companion's scan shows the grade before the shot and feeds the field guide.
- **Harvest by part:** horns, plates, glands and hide each give their own tissue; breaking a part in the fight guarantees it; breakable parts are marked. The right tool gives the right yield (roadmap rule).
- **Scarcity:** at most about 30% of a herd per recovery window; past it, survivors leave the area instead of dying out. Herd pressure shows on the map.
- **Predators hunt the robot** with the same visible meter, and never spawn on top of a fresh landing.

### Combat

- **Weapon model:** each part is a point mass on the grip axis; compute mass, moment of inertia about the grip `I`, balance point and reach. Tip speed `v = C / (I + I_hand)^0.27` with `I_hand ≈ 0.02 kg·m²`; thrust speed ∝ `1/√mass`; wind-up ≈ `reach / v × 0.5`.
- **Gravity:** hold torque `τ = g · Σ m·r` against a grip budget (15 N·m one hand, 30 N·m two); above it the build says "too heavy to swing on this world" and the wind-up stretches by `τ / budget`.
- **Swings:** mouse movement over the last 120 ms picks one of four directions; that sets the plane of an arc pivoted at the grip and swept by hips, chest and shoulder in the pose layers; IK hands hold the grip. Thrust for pierce builds, overhead for heavy heads, slash otherwise. Block is a held pose on the chosen side; parry window 150 ms (100 ms for very light weapons).
- **Damage:** energy `½ I ω²` at the contact radius × a sweet-spot factor, tested against the target's layers DF-style (cut, else blunt). Kickback stun `0.1 s × (1 − energy transferred)`, reduced by a counterweight.
- **Hitstop:** `frames = clamp(3 + 0.4·√E, 3, 12)` at 60 fps (50 to 200 ms); attacker and victim freeze together, the victim shakes from 3 cm to 0; client-side only, the server keeps simulating. Camera shake only when the local player is involved, with an intensity setting.
- **Creature attacks from parts:**

| Body feature | Attack |
|---|---|
| Horns or antlers and room to run | Charge |
| Tail | Sweep behind |
| Neck longer than 0.6 × body | Lunge bite |
| Clawed forelimbs | Swipe |
| Mouth | Bite |
| Mass over 500 kg | Trample |

- **Telegraphs:** wind-up `0.25 s + 2·√(L/g)` (0.38 s at 0.1 m, 0.68 s at 1 m, 1.1 s at 4 m under 22 m/s²); recovery 1.5 × wind-up. Same Froude scaling as the gaits.
- **Hit reactions:** a short physics impulse on the hit part blended back into the procedural pose; parts can break off (Horizon-style) and drop their tissue.
- **Robot death:** as in the roadmap: you wake in a spare body at the base, your gear waits where you fell.

### Crafting

- **0.13.0 slice:** intents cut, pierce, crush (block in 0.13.1); parts grip (0.10 to 0.15 m), shaft (0.2 to 1.4 m), head (edge, point or mass, 0.05 to 2 kg); one binding (sinew or resin, shear 5 to 20 MPa); one treatment (sun-drying: stiffness ×2.5, toughness ×0.8).
- **Materials:** tissue classes bone, antler, keratin, cuticle, hide, plus obsidian and flint from rocks, with the table values scaled per species and per individual grade.
- **Solver:** enumerate at most 200 combinations of carried materials, score them against a reference target (a shell-like cuticle), return three proposals: lightest, hardest-hitting, most durable. Show each as differences against the held item.
- **Explanations:** five templates: edge too soft for the target, weakest joint, too heavy on this world, too brittle (K_IC below 1), no edge material carried.
- **Wear:** a hit counter per joint; the build breaks at its weakest joint; repair replaces that piece.
- **Inventory:** by mass, not slots (cap about 25 kg, slower above 60% × `9.81 / g`), a list with silhouette, name, mass and wear; tissues stack by species, tissue and grade. A six-slot hotbar at the bottom.
- **Look:** the silhouette follows the parts; head mass share above 0.6 → chop or smash; pierce → thrust; else slash.
- **Storage:** an item is a recipe (part template ids, u8-quantised parameters, material refs as species seed + tissue index or mineral id, treatment bits, wear per joint), about 60 bytes; stats are recomputed on load.

### Multiplayer

- **Kill ledger:** herds stay a function of (seed, herd, time), minus a server-held set of `(herd, member, time of death)`. A member returns at `death + recovery`, so the ledger clears itself.
- **Authority:** the server confirms hits, deaths and crafted items; clients predict swings and hitstop locally. The client sends an intent plus a proposal index; the server runs the same deterministic solver and owns the result.
- **Loot:** everyone who damaged the animal in the 30 s before its death gets a full personal harvest. No split, no race.
- **Determinism:** hunt maths on the server, or in fixed point, never relying on float-identical clients.

## Plan of work

Starts after 0.11 ships, or in parallel on its own branch `hunt` from `ecosystems`, two agents at a time, each with its own worktree and cache. Every agent that changes something visible checks it in probe screenshots (see `AGENTS.md`).

### Wave H1: foundations (in parallel)

| Agent | Owns | Delivers |
|---|---|---|
| `matter` | `src/craft/matter/` (new), `src/life/tissue.rs` extensions | Material numbers per tissue class and mineral from the life model and geology; grade per individual; treatments as trade-offs; tests against the table above |
| `weapon` | `src/craft/build/` (new) | Parts, sockets, bindings; mass, inertia, balance, reach, hold torque; swing and thrust speeds; damage against layers (DF-style); wear and breakage; tests for the worked example (0.3 kg × 0.6 m shaft + 0.8 kg head: I ≈ 0.32 kg·m², ω ≈ 24 rad/s, ≈ 93 J) |

### Wave H2: the loop (in parallel, after H1)

| Agent | Owns | Delivers |
|---|---|---|
| `senses` | `src/fauna/senses/` (new), herd roles in `src/life/herd` | Awareness meter, sight, hearing, scent with wind, lookouts, fight or flight, wounded behaviour and blood signs |
| `swing` | `src/avatar/strike/` (new), `src/gear` hooks | Directional swings, thrusts, blocks, parries from weapon stats; IK hands; hitstop and shake; local prediction |

### Wave H3: creatures fight back, and the workshop (in parallel)

| Agent | Owns | Delivers |
|---|---|---|
| `attacks` | `src/fauna/attack/` (new) | Attacks from body parts, telegraphs by size, hit reactions, part breaking, robot damage and death |
| `craft` | `src/craft/ui/`, inventory, hotbar | Intent panel with three proposals and explanations, inventory by mass, hotbar, harvesting by part with tool rules, the one treatment |

### Wave H4: multiplayer and polish

| Agent | Owns | Delivers |
|---|---|---|
| `net` | `crates/net`, `src/net` | Protocol additions: hits, kill ledger, items as recipes, personal loot |
| `feel` | tuning | Sound layering for hits, camera, telegraph readability pass, frame budget check |

### Checks before each merge

- The agent's own area tests, clippy and filet; the lead runs the full gate after merging.
- Probe screenshots of anything visible; the owner plays each wave in a dev or hot build.
- Cost budgets: senses under 20 µs per animal per frame, a swing and its hit test under 0.1 ms, the solver under 5 ms per request.

## Owner decisions (2026-11-20)

1. **Start now,** on branch `hunt` from `ecosystems`, beside the ecosystems fixes.
2. **First weapons:** melee (cut, pierce, crush) **and a thrown spear** in the first release. The spear joins wave H2 (`swing` owns throwing; `weapon` adds a "throw" intent stat: balance point near the middle, flight drag, penetration on arrival).
3. **Predators hunt the robot** in the first release, with the same visible awareness rules, never right after a landing (wave H3, `attacks`).
4. **Hitstop on by default,** with a setting to reduce or turn it off (wave H2, `swing`; the option goes in the gameplay settings).
