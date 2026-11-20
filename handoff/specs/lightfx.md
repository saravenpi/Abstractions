# Light effects: what applies here (agent `lightfx`, 2026-11-20)

Nothing here was compiled or run. Costs are estimates from the code, not measurements.

## Findings that change the code

1. Fixed exposure cannot hold a day and a night. Unity lists sun 110,000 to 120,000 lux, overcast 1,000 to 2,000, moonlight "below 1 lux", starry night 0.002 lux. Unreal and Unity both clamp the exposure (Min/Max EV100, Limit Min/Max) and adapt slower toward the dark than toward the light.
2. Bevy 0.20 `AutoExposure` meters the rendered image with a 64-bin histogram (`range` -8..8, `filter` 0.10..0.90). The sky, fog and clouds of this game are unlit colours that never take `view.exposure`, so an image histogram would brighten the ground and not the sky. Chosen instead: meter the place block (the light the ground shader already uses), write `Exposure`, and lift the sky colours by the same gain in `update_palette`.
3. Bevy `AmbientLight` is flat (docs: brightness times colour, cd/m2). Hemisphere ambient, ground bounce and twilight are not in Bevy; they live in `relit`.
4. Bevy `Bloom`: `prefilter` threshold is "physically inaccurate"; to glow more, raise the mesh `emissive`, not the global intensity. At 360 rows the mip chain is tiny, so bloom is cheap.
5. `VolumetricFog` needs a `FogVolume` bounding the region, `VolumetricLight` on a shadowed light, `step_count` default 64. Starfield benchmark: volumetric High to Low 9 percent, shadows Low to Ultra 18 percent, GTAO 4 percent.
6. Cloud shadows in games are a scrolling noise cookie in world XZ multiplied into the sun term, "negligible cost". Here the noise is the real puff field, so the cookie is computed from it.
7. Point lights: Bevy `intensity` is lumens, 800 lm is about 60 W incandescent; `range` is a hard cut, so it must follow the power. Candela is lumens over 4 pi (arithmetic, not from a page).
8. Directional light default 10,000 lux; the docs advise one or two shadow casters. The game's 12,000 lux noon is in that scale.

## What was built

| Effect | Physical basis | Setting | Expected cost |
| --- | --- | --- | --- |
| Metered exposure (`render/exposure.rs`) | EV100 from lux: sun + moon + pi x ambient; 0.35 stop per stop of light, +1 / -0.5 stop limits, 1.5 s to close, 5 s to open | Auto exposure | one system a frame, a few flops |
| Sky gain | the same gain applied to zenith, horizon, cloud in linear light | follows exposure | none |
| Bloom threshold 2.5, softness 0.5 | sun on white ground is about 3.4, grey ground about 1 (12,000 lux x albedo / pi x 0.001) | Bloom | cheaper than before: fewer pixels pass |
| Hemisphere ambient (`relit`) | mix of sky above and ground bounce below by the cosine of the normal to up; bounce = land colour x (sun x sin h / pi + sky), capped at 1.5 x sky | always on | about 20 ALU per fragment |
| Twilight glow | arch peaks at star height -0.02, tint from the optical depth, 35 percent of ambient, on the side of the star | always on | 10 ALU |
| Cloud shadows (`render/cloudshade.rs`) | sun ray to the cloud shell, real `density_at`, beam `exp(-2 d)`, off below 3 degrees | Cloud shadows | 1,024 texels each second, about 0.15 ms once a second; 8 texture loads a fragment |
| Water | Schlick with F0 0.02 (index 1.33) and exponent 5; was 0.05 and 4 | Sea reflections (sky only, or sky and sun) | none |
| Shadow detail and reach | maps 1024 or 2048; reach 55, 110, 220 m; 2 cascades, 3 at 220 m; the shader fades the last fifth | Shadows, Shadow detail, Shadow distance | the biggest: one pass per cascade |
| Lamp reach (`lighting::reach`) | `sqrt(lumens / (4 pi x 5.5 lux))`: the 850 lm robot lamp keeps 3.5 m | always on | fewer clusters at low power |
| Lamp shadows | cube map for the own robot lamp only | Lamp shadows (Ultra) | six passes while on |

## Presets

| | Low | Medium | High (default) | Ultra |
| --- | --- | --- | --- | --- |
| Shadows | off | on | on | on |
| Shadow detail / distance | Low / 55 m | Low / 110 m | High / 110 m | High / 220 m |
| Ambient occlusion | off | off | on | on |
| Bloom | off | on | on | on |
| Cloud shadows | off | off | on | on |
| Auto exposure | off | on | on | on |
| Lamp shadows | off | off | off | on |
| Sea reflections | sky only | sky and sun | sky and sun | sky and sun |

Render distance, clouds in the sky, vsync and field of view are not in the presets. Every light stays on in every preset. Nothing is detected.

## Left out, with the reason

- Light shafts. `VolumetricFog` marches 64 steps (default) per pixel inside a `FogVolume`, and one volume cannot span a planet at render-space precision. A real shaft needs the cloud cover or the canopy as its density; neither is a 3D field. Not added, no setting.
- Cloud shadows on the sea. The sea material binds the place block as a uniform (`@binding(3)`); the patch lives in the ground texture. Needs a texture binding in `src/world/sea` (agent `perf-sky`).
- Real horizon air mass. `HORIZON_MASS` is 12.5, a real horizon is about 38; it is in `scatter.rs` and `daylight.wesl` with a test, so it is a separate change.
- Sun disc and glare: `sky.rs` and `sky.wesl` are not mine.
- Caves, under water and the cockpit: the place block knows only the sky. A cave or cockpit light would be a metered term in `scene_lux`.
- Resolution: `REFERENCE_ROWS` is 360, fixed, not a setting (`render/tune.rs`). It is the cheapest lever on a weak GPU, and the owner has not asked to expose it.

## For the ship agents (`ship-look`, `cockpit`)

- Units: `PointLight.intensity` in lumens, `range` from `render::reach(lumens)` (floor 5.5 lux). A 4,000 lm lamp reaches 7.6 m.
- Colour temperature: tungsten 2700 K is about (1.0, 0.65, 0.35) in linear RGB, LED 4000 K about (1.0, 0.82, 0.64), cool white 6500 K about (0.9, 0.95, 1.0). Emissive faces take the same colour times 2 to 4 for the bloom (above 2.5 after exposure).
- Shadows: `shadow_maps_enabled: false`; only the own robot lamp casts shadows, on Ultra.
- Engine fire: HDR emissive of 6 or more, so it clears the bloom threshold in daylight.

## Unverified

- Everything compiles only on the lead's machine; shaders are loaded only by the game.
- The 0.35 follow, the +1 / -0.5 limits and the 1.5 and 5 s constants are choices, not measurements.
- Whether the ground at night reads too bright against the lifted sky: look at it.
- `Exposure` is inserted on the game camera by command; its first write is one frame late.

## Sources opened (by the research pass; I did not reopen them)

https://docs.rs/bevy/latest/bevy/camera/struct.Exposure.html
https://docs.rs/bevy/latest/bevy/post_process/auto_exposure/struct.AutoExposure.html
https://docs.rs/bevy/latest/bevy/post_process/bloom/struct.Bloom.html
https://docs.rs/bevy/latest/bevy/light/struct.VolumetricFog.html
https://bevy.org/examples/3d-rendering/volumetric-fog/
https://docs.rs/bevy/latest/bevy/light/struct.DirectionalLight.html
https://docs.rs/bevy/latest/bevy/light/struct.PointLight.html
https://docs.rs/bevy/latest/bevy/light/struct.AmbientLight.html
https://docs.rs/bevy/latest/bevy/pbr/struct.ScreenSpaceAmbientOcclusion.html
https://dev.epicgames.com/documentation/en-us/unreal-engine/auto-exposure-in-unreal-engine
https://docs.unity3d.com/Packages/com.unity.render-pipelines.high-definition@14.0/manual/Override-Exposure.html
https://docs.unity3d.com/Packages/com.unity.render-pipelines.high-definition@14.0/manual/Physical-Light-Units.html
https://www.realtimerendering.com/blog/?p=5694
https://learnopengl.com/PBR/Theory
https://mirzabeig.substack.com/p/unity-tutorial-fake-cloud-shadows
https://gamersnexus.net/game-benchmarks-graphics-guides/starfield-graphics-optimization-guide-benchmarks-settings

Unverifiable (404, 403, too large, off-topic): https://iquilezles.org/articles/fog/ , the Frostbite PBR course notes PDF at https://seblagarde.wordpress.com/wp-content/uploads/2015/07/course_notes_moving_frostbite_to_pbr_v32.pdf , https://seblagarde.wordpress.com/2014/04/14/dont-forget-the-photometric-units/ , https://docs.unrealengine.com/5.3/en-US/auto-exposure-in-unreal-engine/ , https://bevy.org/examples/3d-rendering/auto-exposure/ , https://docs.rs/bevy/latest/bevy/light/struct.AtmosphereSettings.html . F0 0.02 for water is not on any opened page: it is derived from the index 1.33 in the test.
