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
