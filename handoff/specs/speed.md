# Speed: what the game does, what the research says

Written 2026-11-20 by `speed`. Every cue follows a real quantity. Nothing fakes a speed the body does not have.

## What existed before this work

| Body | Cue | Real quantity |
| --- | --- | --- |
| Glider, boots | View widens (`view/sway.rs` `rush`, `shot.widen`) and the boom stretches (`shot.pull`) | speed over the ground, saturating at 30 m/s |
| Glider | Camera lag spring and look-ahead toward the flight path (`sway.rs` `trail`, `ahead`) | velocity |
| Glider | Speed lines in a tube around the body (`gear/fx/streaks.rs`) | airspeed 15 to 55 m/s, air grip |
| Glider | Rush ring at the screen edge (`haze.rs`) | airspeed 55 to 110 m/s, air |
| Glider | Wingtip trails (`trails.rs`), boot exhaust | airspeed, load, air |
| Glider | Cloth flutter (`avatar/wing.rs`), pose buffet (`avatar/flight.rs`) | airspeed |
| Glider | Wind rush sound (`audio/suit.rs`) | airspeed 6 to 75 m/s, planet air |
| Boots | Dust and spray ring, sparks (`motes.rs`), water throw (`effects/water`) | push, clearance |
| Ship, air | Shake, jolt, whoosh in cloud (`ship/shake.rs`, `ship/fx.rs`) | dynamic pressure, cloud depth |
| Ship, air | Entry sheath (`render/heat`), rumble sound (`audio/ship.rs`) | heat, dynamic pressure |
| Ship, space | Widening, exhaust, HUD speed | speed |
| Warp | Aberration, star streaks, tremor (`render/warp`) | beta from the real warp speed on a log scale |
| Water | Kelvin wake (`world/sea/wake`) | Froude number, real wedge |
| World | Ambient motes, leaves, rain, dust (`effects/`) | world-anchored, drift with wind |

## What this work adds

| Cue | Driving quantity | Maximum |
| --- | --- | --- |
| "Speed effects" slider (`ViewPrefs.speed_fx`, 0 to 1, default 1, Controls tab) | none | 0 turns off widening, boom stretch, shake, speed lines, rush ring |
| Glider camera buffet (`gear/fx/buffet.rs`) | `q = 1/2 rho v^2`, 300 to 6000 Pa | 6 mm and 0.1 degree |
| Wingtip condensation (`trails.rs`, `speed::vortex`) | wetness: cloud depth times cover, or rain | trail 1.8 times brighter |
| Vapour cone (`ship/vapour.rs`) | Mach from `sqrt(gamma R T / M)` with T cooled by the dry adiabat, density, wetness | alpha 0.35, 0.85 to 1.25 Mach |
| Ambient particles from ahead (`effects/respawn.rs`) | air velocity against the camera = wind - body velocity | 90 percent of respawns on the upwind face |
| Wind pitch and loudness (`audio/suit.rs`) | pitch from speed, loudness from speed and thinning of the air at height | none |
| Buffet rumble (`audio/suit.rs`) | `FlightFx.buffet` from `q` | 0.9 of full scale before the limiter |

## What the research says, and what applies

Pages opened (full URLs). Many other pages returned 403 and are listed last.

- FOV with speed is the cheapest cue and games use it, even alone: https://80.lv/articles/see-how-you-can-fake-extreme-speeds-in-unreal-engine-5 (90 to 140 degrees in a demo; too much for us). Echo Point Nova widens FOV with speed, smooths the camera, blends grapple speed in over about a second, and has a switch for every camera flourish: https://gamedeveloper.com/design/deep-dive-the-movement-of-echo-point-nova . Applies: one setting, smoothed (rate 2 per second, already so), a few degrees. No comfort limit in degrees was found.
- Parallax and proximity: low viewpoints and near objects feel faster, ground texture detail and objects near the path matter more than speed itself (MotoGP example): https://www.shawnhargreaves.com/blog/vrrroom-whoosh.html . Applies: keep near detail (grass, props) and stream particles close to the path. Does not apply: fake roadside objects.
- Speed lines, radial blur, camera shake are listed as generic cues without numbers: https://80.lv/articles/how-to-make-game-characters-feel-lightning-fast . We keep world-anchored particles and no screen-space lines or blur.
- Noise gains for shake are only multipliers in Cinemachine, no physical units: https://docs.unity3d.com/Packages/com.unity.cinemachine@3.1/manual/CinemachineBasicMultiChannelPerlin.html . Applies: amplitude is ours, scaled by `q`.
- Wingtip vortex condensation: the core cools and condenses below the local dew point, most at high angle of attack, on humid days: https://en.wikipedia.org/wiki/Wingtip_vortices . Applies: trail brightness by wetness, none in dry air or vacuum.
- Vapour cone: low pressure behind the shock cools air under its dew point, humid air only, and it does not need Mach 1 exactly: https://en.wikipedia.org/wiki/Vapor_cone . The Prandtl-Glauert singularity is a wrong explanation, condensation is the cause: https://en.wikipedia.org/wiki/Prandtl%E2%80%93Glauert_singularity . Applies: cone from Mach, wetness and density, not from a singularity.
- Accessibility: no numbers. Set a sane default FOV, let the player adjust it, offer to disable any gap between controller and camera movement, avoid sudden unexpected movement: https://gameaccessibilityguidelines.com/full-list/ . Applies: the slider, defaults to full, smooth ramps.
- Criterion's vehicle feel talk is members only: https://gdcvault.com/play/1025383/Vehicle-Feel-Masterclass-Balancing-Arcade (not read).

Not found, so not claimed: comfort limits of FOV in degrees, buffet frequencies, per-object versus radial blur cost, Doppler numbers, HUD tape practice, haptics, re-entry plasma, studio write-ups for Burnout, Forza, Wipeout, Ace Combat, Elite, Star Citizen, No Man's Sky, Just Cause, Steep, Spider-Man.

Failed fetches (unverifiable): https://www.operationsports.com/why-is-speed-so-hard-to-capture-in-racing-games/ , https://eudl.eu/pdf/10.1007/978-3-319-55834-9_14 , https://forums.frontier.co.uk/threads/config-option-for-field-of-view-in-speed-mode.487663 , https://docs.unrealengine.com/4.27/InteractiveExperiences/UsingCameras/SpringArmComponents , https://www.autodesk.com/solutions/motion-blur , https://ro.uow.edu.au/test2021/1748 , https://www.audiokinetic.com/en/create-a-doppler-effect-with-wwise/ (403), https://www.oreilly.com/library/view/game-audio-development/9781787286450/dcb800a6-3ff9-42bf-bca4-869a5299dbef.xhtml , https://arxiv.org/pdf/1710.02599 , https://par.nsf.gov/servlets/purl/10470533 .

## Decisions

- Space has no wind and no parallax: no streaks there. The cues are the planet, the exhaust, the HUD and the warp optics, which already scale with the real warp speed.
- Doppler is not done: no other ship has an engine voice yet. A voice for remote ships comes first.
- Camera lag with acceleration is already the lag spring: the camera trails the velocity and the boom grows with speed.
- Not done: the widening relative to each body's own top speed. The existing curve saturates at 30 m/s on foot, 250 m/s in the ship, 50 km/s in warp, and is kept.
- Ground rush: blades stand within 40 m and below 30 m of height (`world/blades/tune.rs`), 6 tiles build at once, 2 land per frame. At 60 m/s the 40 m ring is crossed in 0.7 s, so tiles may pop in. Check low and fast. The ground colour carpet is not culled by this.
