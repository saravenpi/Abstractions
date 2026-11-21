# Lighting and LOD colour diagnosis (read from source, nothing run)
## Bug 2: colours differ with altitude
1. (certain <600 m) Season repaints the one shared colour lookup for the whole planet from the spot under the camera, faded by camera clearance: src/render/seasons.rs:60-130, src/render/season.rs:46 (NEAR = (150, 600)); paint() rewrites `colormap` Lut. Fix: per-fragment season tint in ground.wesl from a small uniform (autumn, snow) and the fragment's own position/latitude; remove Lut repainting.
2. (likely) Surface/colour depend on mesh coarseness: src/world/mesh/tris.rs:80-98, 100-116: slope read from the facet, biome dithered per triangle, tone() per-triangle noise. Far mesh level 2 with 16 cells per node -> slope ~0 -> rock/snow vanish. Fix: slope and height from the height field at the cell centre at a fixed analytic scale (Globe::glance does this for the map); tone only below some LOD.
3. (certain) Three colour functions: meshes use Lands::surface / Lands::rgb (tris.rs, lands/paint.rs); map picture src/map/raster/paint.rs:37-100 has own constants (ROCK_SLOPE, ROCK_HEIGHT, SNOW_HEIGHT, SAND_HEIGHT) + fake light shade() with NW LIGHT + height brightening. Fix: colour() calls the same paint path; drop shade() from albedo.
4. (likely) Turf carpet only on detail chunks: ground.wesl:25-32 mixes carpet() when uv_b exists (tris.rs:160). Fix: bake mean carpet colour into vertex colour of every LOD; shader adds only detail around the mean.
5. (likely) Ball uses different shading: src/render/bodies.wesl:104-123 own sunlit() (scatter, no specular) vs ground PBR via relit() (daylight.wesl:189); ball reads unseasoned colours (ball.rs:54), smooth normals. Fix: use sun_response from daylight.wesl in bodies.wesl.
6. (minor) colormap.rs 32 levels per channel.
## Bug 1: dark zones
1. (likely) SSAO on by default with DepthPrepass: src/render/camera.rs:36-40, src/settings/model.rs:116; skirts 3 m (tris.rs:51-56), LOD seams -> dark halos; daylight.wesl:188 multiplies by diffuse_occlusion. Fix: raise constant_object_thickness or disable above some clearance; prove by AO off screenshots.
2. (likely) Shadow/depth prepass does not dither visibility ranges (bevy_pbr-0.20.1 prepass.wesl:169): during 0.4 s LOD fade (chunks/fade.rs, dither.rs) coarse chunk shadows the fine one. Fix: NotShadowCaster on meshes carrying Way::Leave/Way::Hide; better: no shadow casting from chunks coarser than shadow range (110 m).
3. (likely) relit subtracts camera light regardless of shadows: daylight.wesl:189-197 `lit + (here - eye) * response` clamped at 0; second implementation of scene lights (Sky::light / Palette::seen). Fix: derive DirectionalLight lux from the same function (PlaceUniform::eye_sun) so delta is zero at camera; apply delta scaled by shadow term.
4. (guess) one-frame lag: update_palette/update_lights in Update, SunDir tracking and place_light in PostUpdate.
5. (certain, missing-shadow) 2 cascades 28 m and 110 m at 2048 (lighting.rs:57-65), no fade beyond last cascade; shade.rs fades 45-95 m clearance only. Fix: fade shadow by view distance in ground shader or third cascade.
6. (minor) moon casts shadows by day (lighting.rs:111-118).
## Sources opened
https://docs.rs/bevy/latest/bevy/light/struct.CascadeShadowConfigBuilder.html
https://learn.microsoft.com/en-us/windows/win32/dxtecharts/cascaded-shadow-maps
https://docs.rs/bevy/latest/bevy/pbr/struct.ScreenSpaceAmbientOcclusion.html
https://docs.rs/bevy/latest/bevy/camera/visibility/struct.VisibilityRange.html
https://outerra.blogspot.com/2012/11/maximizing-depth-buffer-range-and.html
Unverifiable (404 or off-topic): https://bevy.org/examples/3d-rendering/ssao/ https://www.reedbeta.com/blog/artifacts-in-linear-and-srgb-filtering/ https://www.gamedeveloper.com/programming/a-scalable-planet-renderer https://www.gdcvault.com/play/1017918/Terrain-in-Battlefield-3-A https://ebruneton.github.io/precomputed_atmospheric_scattering/
## Screenshot test
One seed, fixed sun, same ground point, autumn; straight down then tilted at 2 m, 10 km, 100 km, 1,000 km, 20,000 km: mean linear colour of a 64x64 patch within 3% luminance and 0.02 chroma across all five.

## look-sky pass (2026-11-21): sky grain, clouds, fog, white balance

Found
- The sky grain was the cloud shell, not the stars. `clouds::drift` and `bodies::follow` rewrite their `Visibility` every frame, so the probe cannot hide them: despawn the shell instead. Far clouds had their alpha cut to a quarter in `clouds.wesl` and the screen-door `discard` turned that into flecks, densest at the horizon. Fixed: far clouds keep alpha and fade to the haze colour with a Koschmieder law (`fog::AIR_VISIBILITY`, 40 km over `Air::haze`); `LONE` (0.3, 0.8) in `cloud_field.wesl` and its twin `weather/puff.rs` cut lone tips, so cloud shadows follow the drawn clouds.
- Stars and the galaxy band keyed on `palette.stars` (`1 - air` on thin air). They now key on the luminance of the sky colour (`render/skylight.rs`, mirrored in `starfield.wesl`).
- Clouds were blue under the white balance because their colour was a fixed 35 percent white mix. They are now lit by the star colour against the Sun (`scatter::lit_cloud`, SUN_LIGHT is the neutral of the balance) and shaded by the sky ambient (`CloudUniform.ambient`).
- The camera fog was linear with a wall at 0.9 of the reach and its reach is the streamed ground (`fog_sight`, `Reach::sight`): on foot it exists to hide the chunk edge, it is not air. It is now exponential squared (`FOG_DENSITY` 2.4: 1 percent left at `FOG_END`), and the sea reads the same curve (`water.wesl`, `FOG_SQUARED`).
- White balance: `render/exposure/balance.rs`, CAT02 degree, 26 s time constant, snaps when switched on.

Not done
- Terrain `tone` jitter of plus or minus 8 percent per triangle (`src/world/mesh/style.rs:65`, used at `tris.rs:132`) reads as a quilt on flat ground. Belongs to `look-ground`: scale it by slope or distance.
- Fog colour is still the horizon colour, with no per-channel extinction. Hillaire-style aerial perspective (Bevy 0.20 ships `bevy_pbr::atmosphere` with LUTs, needs compute shaders and its own dome) was judged too large to adopt blind.
- Far terrain beyond the streamed ground does not exist on foot, so the horizon is still fog.
- Probe stages: `look-grain`, `look-cover`, `look-vista-light`, `look-vista-height`. On a locked home `clock_at` cannot reach a low star: fly toward the terminator.

## light2 pass (2026-11-21): aerial perspective

Changed
- The camera fog is `FogFalloff::Atmospheric` fed by `render/aerial.rs` (`Haze`): gas extinction `scatter::DEPTH * depth * share / (atmosphere / 12)` per channel, aerosols `AEROSOL` in a layer 0.15 scale heights thick, thickened by `WeatherLook::sight`. `fog::extinction` takes the mean density between the camera and the ground under it, so it is exact for the ground seen from any height on a flat approximation. The fog colour is the horizon of the sky at the ground under the camera, whatever the height of the camera.
- `fog_sight`, the fog `Height`, `OPEN`, `HORIZON_SHARE`, `FOG_DENSITY`, `FOG_END`, `AIR_VISIBILITY` and `THIN_AIR` are gone. The camera `far` is where 1 percent of red light is left (about 86 km in clear home air), capped at 100 radii.
- `world/chunks/sight.rs`: `Eye::of` sees as far as the clear air does, so every node swap in view fades. `veil` is kept for the flight tests only.
- `water.wesl` and `clouds.wesl` read Bevy's `fog` uniform for the colour; the clouds get their own extinction (`CloudUniform.fade`) to their height.

Not done
- The glow around a low star is on the dome only: the fog has no directional term yet, so far ridges under the star fade to the plain horizon colour.
- Props end at the detail radius (256 m at Medium) and now show it. Coarse terrain is what the horizon is made of and smears at speed.
- `WaterUniform.params.zw` and `FogLook.start/end` are no longer read by the shader.
- Probe: `look-isolate` hides by taking `Mesh3d` off. Never hide through `Visibility`.

## light2 pass (2026-11-21): clouds and sky colour

Found from the `look-isolate` shots
- The sky flecks are the clouds at light cover: forced cover 0 gives a clean sky. The noise cell was 450 m under a cloud band whose middle is 7.5 km up, so a puff was a few pixels overhead and under one at the horizon. Overcast was a quilt because a full cover started puffs at a noise value of 0.4, which left two fifths of the sky as gaps.
- The clouds read blue because the sky was too red, not because the clouds were wrong: `Palette::through` multiplied the scattered light by the colour of the star a second time, so under a red star the sky and the sky fill were redder than the star light, the white balance adapted to that, and anything lit by the star alone looked blue against it.

Changed
- `weather/puff.rs`: `CELL` 1800 m, `WIND` kept at 5 m/s, `THRESHOLD` (0.72, 0.25). The ground shadows and the whiteout read the same field, so cloud shadows are four times wider.
- `cloud_field.wesl`: `fbm_seen` fades each octave of the noise to its mean once its cells are under two to four pixels, so no cloud detail can be smaller than that at any distance. The Rust twin keeps every octave: the two agree near the camera.
- `clouds.wesl`: the cut-out edge is 1.5 pixels wide whatever the distance (`EDGE_PIXELS`), dithered inside that band only.
- `palette.rs`: the star colour enters the sky once.

Not done
- Cloud shading is still one relief term from the noise toward the star. No separate lit top and shaded base by height.
- The isolate stage could not say why I07 to I16 showed no cloud: the cover of the weather moves during the run. It now forces a reference cover of 0.3 for every shot and logs the weather cover.

## light2 pass (2026-11-21): the haze is back, as haze

The owner missed the distance fog: at 54 km of clear-day sight the haze was invisible at the scale of play.
- `AEROSOL` now gives 23 km on an ordinary dry day (the standard rural air of the aerosol models), about 10 km in the damp air under a whole cover of cloud (`WeatherLook.damp`, `DAMP_FOG`), 3 km in heavy rain, 800 m in a dust storm. Dusty air (`Air::dust`, the tint that stains the sky) doubles the aerosols. The aerosols thin with the square root of the pressure, the gas with the pressure.
- The ground shader works the haze out per point (`aerial` in `daylight.wesl`): the mean of the gas and aerosol layers between the height of the camera and the height of the point, so valleys and the sea fill and peaks stand clear. It reads the layers from `fog.directional_light_color.rgb` (gas, aerosol, scale height) and `fog.directional_light_exponent` (cloud whiteout), which Bevy leaves unread while the alpha is 0. Standard materials, the grass and the sea keep Bevy's uniform fog at the mean between the camera and the ground under it.
- The haze takes the glow of the dome around the star (`star_glow`, the two lobes of `sky.wesl`), so ridges under a low star fade warm.

## light3 pass (2026-11-21): golden light, fill, sunset band, cheaper shadows

Found from the rel-haze57 shots
- Sunset and dusk skies and clouds were green: the white balance adapted (CAT02, degree near 1) to the light at the camera, which a low star reddens. Dividing the beige horizon by that orange light gives teal-green; adapting to the blue night ambient gives grey-green dusks.
- Shadowed faces were black: day sky 350 against a 12,000 lux sun is 11 to 1 on level ground.
- The whole horizon turned orange at sunset, the anti-sun side too.
- Cloud undersides took the saturated sky colour and read blue.

Changed
- `exposure/balance.rs`: the white is `PlaceUniform::noon()`, the light of the place with the star overhead, at every hour. Noon looks as before; sunrise and sunset stay warm; night stays blue.
- `Palette::DAY_AMBIENT` 350 to 650 (6 to 1), `REFERENCE_LUX` 13,000 to 14,000.
- `palette.rs` `HORIZON_WARMTH` 0.45 (was a power of 0.7) and `sunset_band` in `sky.wesl` and in `star_glow` of `daylight.wesl` (same function, test `tests_sunset`): a warm band on the horizon under a low star, gone above a sine of 0.35.
- `clouds.wesl`: undersides `mix(white, sky, 0.45)` at 0.62 of the lit side.
- Cloud shadows opt-in (`Lighting::cloud_shade`, old key `cloud_shadows` dropped once), outside the presets.
- Coarse nodes and the far planet are `NotShadowCaster` and `Shadowless` (`world/chunks/dither.rs`).

Not done
- The fog knobs still ride in `fog.directional_light_color` and its exponent (`fog_for`).
- The low sun stays bright (54 percent of noon at 3 degrees): `transmit` has no aerosol term.
- Tonemapper: TonyMcMapface desaturates mid tones; an A/B against AcesFitted or AgX is worth one build.
- Shadows end at 110 m; ridges cast no long shadows at golden hour.

## clouds-fix pass (2026-11-21): puff size, the limb, the drift and the shadows

Found
- Far puffs shrank: `fbm_seen` faded every octave to its mean, the first one too. The mean (0.469) is under the cut of a broken sky (0.58 at cover 0.3), so past about 30 km on the ground (fp 0.3 cells a pixel) the cover fell to nothing, and over cover 0.6 it went to a solid sheet.
- The limb jumped: `LIMB` lifted the noise by 0.3 at grazing views and `thick` raised the density to 1/facing, so the cover went from 0 at 40 km to 1 at 80 km on a sky of cover 0.3.
- The drift was a translation of the 3D noise along fixed axes: east motion where the shell is normal to Y, boiling in place where it is normal to X, apart from the broad cover that turns round Y, and it started over every 131,072 s.
- The shadows were a 32 by 32 grid of 250 m texels centred on the camera, worked out once a second from the weather cover, not the cover the shell reads.

Changed
- `weather/puff` (folder): the first octave is always read; the finer ones fade at 2 to 4 pixels a cell and the spread is stretched back (`sqrt(full / kept)`). The cover holds within 2 percent at any distance (numpy twin of the field, 200,000 points).
- Grazing cover (`graze`, Rust twin in `weather/puff/graze.rs`, test only): a sight crosses `1 + TALL cot` puffs; the noise is lifted by the spreads that give `1 - clear^crossed`, through a logistic-with-cubic fit of the noise tail (`TAIL`). `SPREAD` 0.1065 and the tail were measured over a million points.
- Drift: `weather::drift(seconds, cells)` is the cosine and sine of the turn round the spin axis of the broad octave and of the finer ones (`SHEAR` 0.25 faster), at `WIND` 4 m/s, the speed of the broad cover. Angles in f64 round the circle. The shaders call `spot_at`.
- Shadows: `cloud_shade` in `daylight.wesl` intersects the sun ray with the shell per pixel and calls `overhead`, the drawn field seen from overhead. The light block holds five texels. `CloudShade::eye` reads the same rule on the CPU each frame.
- Probe: `PROBE_STAGES=skyfly` (`sky-3km`, `sky-15km`, `sky-orbit`), eleven shots one second of the game apart.

Cost
- Shell: 4 + 2 octaves of noise per pixel as before, fewer far away (faded octaves are skipped), plus about 20 ALU ops for the grazing cover.
- Shadows on: one ray and up to 4 octaves per ground and grass pixel, against a texture fetch before; off: nothing. CPU: the 1,024 field samples per second are gone.

Not done
- A puff whose first octave is under a pixel (orbit past about 250 km, the last degrees above the horizon) is point sampled: its cover is right on average, it reads as a stipple. The far ball dithers the same way.
- The shell is still one surface at mid band. Puffs have no height of their own except through the grazing cover.

Sources
- Castaño, Computing alpha mipmaps: http://www.ludicon.com/castano/blog/articles/computing-alpha-mipmaps/ (alpha-tested detail vanishes with distance unless the coverage is kept per level; the stretch of the spread does that for the noise).
- Schneider, The real-time volumetric cloudscapes of Horizon Zero Dawn, SIGGRAPH 2015: https://advances.realtimerendering.com/s2015/The%20Real-time%20Volumetric%20Cloudscapes%20of%20Horizon%20-%20Zero%20Dawn%20-%20ARTR.pdf and https://www.guerrilla-games.com/read/the-real-time-volumetric-cloudscapes-of-horizon-zero-dawn (the PDF could not be read here: what it says about wind offsets is from memory).
- Hillaire, Physically based sky, atmosphere and cloud rendering in Frostbite, SIGGRAPH 2016: https://media.contentapi.ea.com/content/dam/eacom/frostbite/files/s2016_pbs_frostbite_sky_clouds.pdf (found, not read).
- Lague, Coding Adventure: Clouds: https://www.youtube.com/watch?v=4QOcCGI6xOU (found, not watched).
- Unverifiable: https://bgolus.medium.com/anti-aliased-alpha-test-the-esoteric-alpha-to-coverage-8b177335ae4f (403).
