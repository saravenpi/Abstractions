# Spatial audio (0.11.0)

Owner request: "Make sure the sounds have actual locations and we don't just hear them played through: they should have an actual 3D position, fade with distance, and depend on the atmosphere too. Make it more realistic." Folded in later: the cockpit and the chase view sound different, the cockpit is quieter and sounds like a small room, and the air on the ship follows the real flow.

Code: `src/audio/spatial/` (the path), `src/audio/ship/` (the ship voices that use it), `src/ship/sound/` (what the ship tells the audio).

## Audit

Before this branch only the ship voices had a real path (delay line, Doppler, 1/r, air). Everything else either played flat or used a side-and-nearness pan with a law of its own.

| Sound | Emitted by | Before | Now |
| --- | --- | --- | --- |
| Blow, glance (spear, knife, bite, carving, blood) | `fauna/attack`, `fauna/bleed`, `avatar/strike`, `net/hunt/field` via `Impact` | side + `10 / (10 + r)`, no delay, no air | `Sound` at the contact point, reach 1 m, 1/r, delay r/c, ISO air, hill and hull occlusion. Within 2.2 m of the listener's body it is held: the body carries a share in thin air and vacuum |
| Swing | `avatar/strike/swish` via `Whoosh` | side + `10 / (10 + r)` | `Sound` at the tool tip, reach 1 m. Air noise, so never through the body |
| Animal calls | `fauna/signs/hear` | side + `h / (h + r)`, no delay | `Sound` at the herd, at ear height, reach `0.1 * 12 m * cbrt(mass)`. The fauna still ranks and culls with its own law |
| Own footsteps | `avatar/tread` | centred, air share, body share 0.5 | `Sound` held at the feet, 1.7 m below the ears, body share 0.5 |
| Other players' footsteps | `avatar/tread` | side, 1/r, fixed 343 m/s, cut at 60 m | `Sound` at their feet, reach 1.7 m, planet c, no cut: they end under the floor of hearing (-54 dB) |
| Companion chirps | `companion` via `CompanionChirp`, `audio/wire` | side + `6 / (6 + r)` from a continuous param | `Sound` from where it floats, reach 0.7 m |
| Thunder | `audio/sky` | flat, level times the altitude fade | `Sound` from a hashed bearing 1 km off, 300 m up, late by 3 s, dulled by 1 km of air |
| Sonic boom | `ship/sound/near` | flat | `Sound` arrived from the ship (placed and dulled, no added delay: the boom is the arrival) |
| Boot pump | `gear/fx/pump` | flat | `Sound` held at the body, reach 1.7 m, body share 0.6 |
| Ship voices (own, remote) | `ship/sound` into `AudioScene.hulls` | own delay line, `stereo` pan, fixed air law | same delay line; pan through the shared `Ear`; ISO air of the planet; front/back from `Ch::Front` |
| Cabin cues: relay, warp spool/engage/exit, landing, refused warp | `audio/wire` from `ShipEvent` | flat | from the panel while the ears are in the cabin (rings in the cabin); from the cockpit through the hull, reach 0.3 m, when they are outside |
| Wind, rain, dust, shore beds | `audio/ambience` | diffuse, altitude fade | diffuse beds (no point source), now also times the sea-level density of the planet; the shore keeps its side |
| Rush, flutter, boosters (wingsuit) | `audio/suit` | on the body | unchanged: on the body, none in vacuum |
| Clouds whoosh | `audio/ship` | diffuse | unchanged |
| Interface (move, confirm, cancel) | `ui`, `menus`, `computer`, `craft/ui`, `fauna/guide` | flat | flat |
| Music | `audio/music` | flat | flat |

## The model

- **Listener.** The ears turn with the camera, so a sound comes from where it is on screen. They sit in the robot's head (0.8 m over the body centre, 1.7 m over the soles) whenever it stands in the world, first or third person. Seated in the ship, or with no body, they are the camera: in the cabin with the seat view, outside with the chase view. `audio::spatial::Listener`, written in `Sensing`.
- **Distance.** Pressure is `reach / max(r, reach)`, so 1/r past the reach of the source and 6 dB per doubling. No fixed cut: a sound under -54 dB of its reach level (`FLOOR`) takes no voice. Reaches were picked so that nothing near is more than 0.8 dB louder than the old laws.
- **Air.** Density at the listener is the sea-level density times `exp(-h / H)`. Pressure at the source follows density (a monopole), capped at 2x home. A vacuum carries nothing through the air.
- **Absorption.** ISO 9613-1 (classical, O2 and N2 relaxation, humidity from the planet's ocean share, temperature at height). Past the standard: the classical term follows the gas's molar mass and ratio of heats, and CO2 relaxes at `38 kHz * p`, about 240 Hz on Mars, as Perseverance measured. Each frame the corner of 3 dB loss is found at 30 m and 1 km and a power law is fitted through them; a one-pole low-pass at that corner is the air. At home: 15.7 kHz at 10 m, 5.3 kHz at 50 m, 700 Hz at 1 km. At 0.1 bar every corner is lower. A world with no water has no vapour and carries highs further: 2.2 kHz left at 1 km.
- **Speed of sound.** From the planet's gas and its temperature at the listener's height (`speed::sound_at`). Delay is `r / c` at emission. A vacuum has no delay because nothing travels.
- **Ears.** Equal-power pan, 1.0 in each ear straight ahead. At 90 degrees the near ear gets 1.31 and the far ear 0.54 (7.7 dB). The far ear hears the sound later by Woodworth's `a / c (theta + sin theta)`, with a = 8.75 cm: 0.66 ms in home air. A head-shadow low-pass on the far ear closes to 1.8 kHz fully to the side. A sound from behind loses 4.4 dB above 3 kHz.
- **Water.** Ears more than 0.3 m under the surface: c = 1480 m/s, corner 1.2 kHz (bone conduction), 30% of the bearing. Source and ears both under water: cylindrical spreading past 10 m. Across the surface: 20 dB down (real is 30 dB), corner 650 Hz.
- **Occlusion.** Seven terrain heights read along the line from the ears to a source, skipping 1.5 m at each end. A hill between them takes 6 dB and leaves 700 Hz. Cost is 7 `Terrain::height` calls per placed one-shot, a few per second.
- **Moving sources.** Ships keep their delay line, which gives Doppler. One-shots get their delay and filter once, when they start.

## Seat and chase view

| | Seat (first person in the ship) | Chase view (third person in the ship) | On foot by a parked ship |
| --- | --- | --- | --- |
| Ears | the camera in the seat, in the cabin | the chase camera, outside | the robot's head, outside |
| Own engines and nozzles | through the canopy low-pass (260 Hz shut, 6 kHz open), with the airframe rumble, pump shafts and mechanisms through the frame | from their place on the hull through the path: distance, bearing, air, Doppler, full bandwidth | same as chase, from where you stand |
| Wind on the airframe | through the hull, low-passed at 900 Hz | rushing past the camera at the ship's airspeed, 60% of it, panned by the sideslip | none |
| Other world sounds | through the hull: 10 dB down, corner 500 Hz, ring in the cabin | open | open |
| Cabin cues and the companion | from the panel, at their level, dry plus the cabin's reflections | from the cockpit through the hull, about 32 dB down at 12 m | same |
| Ship bus level | 4 dB under the chase view (`SEAT = 0.63`); the cabin gives about 1 dB back as reflections | reference | reference |
| Vacuum | the frame still carries the engines | silent apart from the music and the interface. No structure-borne hum is added: the camera touches nothing | silent apart from what your body carries |

Switching the view or boarding fades over about 0.3 s (`VIEW_FADE`, a 0.1 s time constant). The ship's inside share and the cabin muffling of the beds both use it.

**Cabin acoustics** (`audio::spatial::room`). For a cockpit of about 1.5 m³: four early reflections at 2.1, 3.3, 4.6 and 5.7 ms, alternating ears; one damped feedback comb at 7.1 ms with a 0.15 s decay to -60 dB, walls damped above 3.5 kHz; a +3.5 dB rise at 350 Hz for the cabin modes; 35% wet. It costs about 20 operations per sample and is silent outside. It is fed by the ship bus while the ears are in the cabin and by every one-shot whose spot has `room = 1`. Nothing is added to the open world in the chase view.

**Air on the airframe** (`audio/ship/airframe.rs`, `src/ship/sound/flow.rs`). The level follows the square root of dynamic pressure (10 dB per decade of q). The centre is 220 Hz plus 5 Hz per m/s. Angle of attack or sideslip from 8 to 25 degrees tears the flow: the band widens (Q down 60%), gets up to 80% louder, and the buffet rumbles. Roll and yaw rate from 0.5 to 2.5 rad/s add up to 50%. A 2 to 5 g pull buffets. Every gain eases with a 0.1 s time constant. The base level is 3 dB under 0.11.0 (`WIND.1` from 0.12 to 0.085). Angles and rates come from the real hull velocity and spin of the ship this machine flies. Remote ships carry no flow data, so their airframe follows airspeed only.

How other games do it:

- MSFS 2024 keeps separate mixer buses for the cockpit view and the outside view, and tags interior sounds `ViewPoint="Inside"` (https://docs.flightsimulator.com/msfs2024/retail/sound/master-mixer-hierarchy/ and https://docs.flightsimulator.com/msfs2024/retail/content-configuration/models/modelbehaviors/usingbehaviours/interactions/adding-sound-to-interactions/). The mixer page fetched without its body text, so these details come from search summaries and are not verified.
- MSFS players report a delay of several seconds when the sound set changes between the cockpit and an external camera, and wing views picking up the exterior set (https://forums.flightsimulator.com/t/sound-change-delay-when-switching-cockpit-external-views/772701, https://forums.flightsimulator.com/t/engine-sound-in-exterior-interior-views/378886). That is why this branch crossfades in 0.3 s.
- DCS players ask for separate interior and exterior levels (https://forum.dcs.world/topic/183566-world-sound-loudness-in-external-view).
- Elite Dangerous: Matthew Florianz explains the cockpit sound as the ship rendering the world through speakers, and says it changes when the canopy is lost (https://community.elitedangerous.com/en/node/274). That link now redirects to https://www.elitedangerous.com/news/galnet, so the quote comes from a search summary and is not verified. See also https://forums.frontier.co.uk/threads/meet-the-team-10-jose-castro-sound-designer.8236/page-3.
- Star Citizen: Wwise with distance blending (https://robertsspaceindustries.com/comm-link/transmission/14853-Star-Citizen-Audio-Update). Propagation through rooms and portals inside ships (https://robertsspaceindustries.com/spectrum/community/SC/forum/3/thread/star-citizen-comms-dive-3-september-16-2025/8275427).
- Mars attenuation: Maurice et al., Nature 605, 2022, https://doi.org/10.1038/s41586-022-04679-0.

## Cost

- **Audio thread:** about 25 operations per sample per live one-shot voice (16 blips and 8 treads at most; idle voices cost nothing). Plus 20 for the cabin room. Ships unchanged plus about 10 for the pan.
- **Game thread:** a few hundred operations per placed one-shot (the spot, plus 7 terrain heights for occlusion). The air fit runs once a frame: two 24-step searches of the ISO law.

## For the `creature-sound` merge

`creature-sound` (not merged) adds `side`, `near` and `metres` to `Call` and pans its throats with `stereo(call.side, 1.0)` and `tract::air(metres)`. After this branch:

- `Call` carries no place. Drop `side`, `near` and `metres` from it and from `Call::pack`; keep `one`, `formant` and `print`.
- `Throats::play(call, spot: &Spot)`: give each `Throat` an `audio::spatial::Ear` (`Ear::new(1.0)`, `ear.place(spot, rate)`) in place of `sides`. Render with `ear.frame(sample, sample)`, and start the throat `spot.delay` seconds late.
- Drop the air low-pass from the tract: the ear's way filter is the air now. Keep the muffle for a call from under water.
- Set the plan's `gain` from `LEVEL * mood.gain * muffle`, with no `near`: the spot carries the distance.
- In `Pool::play`, `self.buddy` is gone (the companion is placed by its spot). `BUDDY_UNDER` can scale the blip gain of a `Chirp` while throats are busy.
- `fauna/signs/hear.rs` already sends `Sound { cue: Call(..), from: Origin::At(herd point, reach) }`. Keep that, and add `one` to the `Call` it builds.
- The render takes here are numbered 30 to 39 so they do not collide with creature-sound's 21 to 29.
