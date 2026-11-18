//! The values the water shader reads: sky colors, light, fog and waves, refreshed every frame.

use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

use super::blend;
use super::liquid::Liquid;
use super::waves::waves_for;
use crate::render::Palette;
use crate::universe::PlanetRecord;
use crate::wind::Wind;
use crate::world::lands::Lands;

/// Uniform block read by `water.wesl`. Every field is a `vec4` so the layout never needs padding.
///
/// Wave directions are in sea coordinates, the chart of the seat, not in render axes.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct WaterUniform {
    /// Linear sky color straight up.
    pub zenith: Vec4,
    /// Linear sky color at the horizon.
    pub horizon: Vec4,
    /// Linear fog color in xyz.
    pub fog: Vec4,
    /// Sun direction in xyz and its visibility in w.
    pub sun: Vec4,
    /// Linear sun color in xyz and the daylight factor in w.
    pub sun_color: Vec4,
    /// Moon direction in xyz and its visibility in w.
    pub moon: Vec4,
    /// First wave: direction x, direction z, wave number, amplitude.
    pub wave_a: Vec4,
    /// Second wave, same layout.
    pub wave_b: Vec4,
    /// Third wave, same layout.
    pub wave_c: Vec4,
    /// Fourth wave, same layout.
    pub wave_d: Vec4,
    /// Time in seconds, wind strength, fog start and fog end.
    pub params: Vec4,
    /// Sea coordinates of the floor map center in xy and of the mesh center in zw.
    pub map: Vec4,
    /// Render position of the planet center in xyz and the sea level radius in w.
    pub globe: Vec4,
    /// Render direction of sea coordinate x at the seat in xyz, and in w the distance from the
    /// camera at which the near sea has faded out.
    pub frame: Vec4,
    /// Linear colour of the liquid over a shallow floor in xyz, and in w the light it gives off by itself.
    pub shallow: Vec4,
    /// Linear colour of the liquid over a deep floor in xyz, and in w the share of the sky it mirrors.
    pub deep: Vec4,
    /// Linear colour of the foam in xyz, and in w the least opacity of the surface.
    pub foam: Vec4,
    /// Linear colour of the flat sea of the coarse planet mesh before it is lit, which the near sea turns
    /// into from afar.
    pub far: Vec4,
    /// The hand-over, see [`blend::packed`].
    pub blend: Vec4,
}

/// Sines of the height of the star between which the water goes from its night look to its day
/// look. `water.wesl` holds the same pair under the same name, for the water far from the camera.
pub const DAY_EDGE: (f32, f32) = (-0.1, 0.3);

/// Sines of the height of the star between which its glitter on the water comes on, the same
/// pair as in `water.wesl`.
pub const SUN_UP_EDGE: (f32, f32) = (-0.05, 0.12);

/// Distance fog of the camera, in the units the shader needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FogLook {
    /// Linear fog color.
    pub color: Vec4,
    /// Distance where the fog starts.
    pub start: f32,
    /// Distance where the fog hides everything.
    pub end: f32,
}

/// Converts an sRGB triple to a linear color vector.
fn linear(v: Vec3) -> Vec4 {
    Color::srgb(v.x, v.y, v.z).to_linear().to_vec4()
}

impl FogLook {
    /// Reads the camera fog, or falls back to a horizon colored fog without a camera.
    pub fn from_fog(fog: Option<&DistanceFog>, horizon: Vec3) -> Self {
        let Some(fog) = fog else {
            return Self {
                color: linear(horizon),
                start: 120.0,
                end: 420.0,
            };
        };
        let (start, end) = match fog.falloff {
            FogFalloff::Linear { start, end } => (start, end),
            FogFalloff::Exponential { density } => (0.0, 3.0 / density.max(1e-4)),
            FogFalloff::ExponentialSquared { density } => (0.0, 1.8 / density.max(1e-4)),
            _ => (120.0, 420.0),
        };
        Self {
            color: fog.color.to_linear().to_vec4(),
            start,
            end,
        }
    }
}

impl WaterUniform {
    /// Builds the uniform for a palette, the wind in sea coordinates, a time in seconds and the
    /// camera fog, drawing the water of home. The map, globe and frame values are left for the caller.
    pub fn new(palette: &Palette, wind: &Wind, time: f32, fog: &FogLook) -> Self {
        let [a, b, c, d] = waves_for(wind).map(|wave| wave.packed());
        let sun_up = smooth(SUN_UP_EDGE.0, SUN_UP_EDGE.1, palette.sun_dir.y);
        let water = Self {
            zenith: linear(palette.zenith),
            horizon: linear(palette.horizon),
            fog: fog.color,
            sun: palette.sun_dir.extend(sun_up),
            sun_color: linear(palette.sun_color).truncate().extend(smooth(
                DAY_EDGE.0,
                DAY_EDGE.1,
                palette.sun_dir.y,
            )),
            moon: palette
                .moon_dir
                .extend((palette.moon_lux / Palette::MOON_LUX).clamp(0.0, 1.0)),
            wave_a: a,
            wave_b: b,
            wave_c: c,
            wave_d: d,
            params: Vec4::new(time, wind.strength, fog.start, fog.end),
            blend: blend::packed(),
            ..Self::default()
        };
        water
            .of(&Liquid::WATER)
            .over(Lands::of(&PlanetRecord::TEMPERATE))
    }

    /// Returns these values over the coarse sea of `lands` in place of the one they had.
    pub fn over(self, lands: Lands) -> Self {
        Self {
            far: lands.sea_shown().extend(1.0),
            ..self
        }
    }

    /// Returns these values drawing `liquid` in place of the liquid they had.
    pub fn of(self, liquid: &Liquid) -> Self {
        Self {
            shallow: liquid.shallow.extend(liquid.glow),
            deep: liquid.deep.extend(liquid.mirror),
            foam: liquid.foam.extend(liquid.cover),
            ..self
        }
    }
}

/// Hermite interpolation of `x` between `a` and `b`, clamped to 0..1.
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Noon lights the sun and a night shows the moon only.
    #[test]
    fn sun_and_moon_visibility_follow_the_day() {
        let fog = FogLook::from_fog(None, Vec3::ONE);
        let noon = WaterUniform::new(&Palette::at(0.25), &Wind::default(), 0.0, &fog);
        let night = WaterUniform::new(&Palette::at(0.75), &Wind::default(), 0.0, &fog);
        assert!(noon.sun.w > 0.99 && noon.moon.w < 0.01);
        assert!(night.sun.w < 0.01 && night.moon.w > 0.9);
    }

    /// A fresh uniform draws the water of home, and another liquid only changes the liquid values.
    #[test]
    fn the_uniform_carries_the_liquid() {
        let fog = FogLook::from_fog(None, Vec3::ONE);
        let water = WaterUniform::new(&Palette::at(0.25), &Wind::default(), 0.0, &fog);
        assert_eq!(water.shallow, Vec4::new(0.03, 0.55, 0.48, 0.0));
        assert_eq!(water.deep, Vec4::new(0.002, 0.04, 0.16, 1.0));
        assert_eq!(water.foam, Vec4::new(0.95, 0.98, 1.0, 0.0));
        let lava = Liquid {
            glow: 4.0,
            cover: 1.0,
            ..Liquid::WATER
        };
        let hot = water.of(&lava);
        assert_eq!((hot.shallow.w, hot.foam.w), (4.0, 1.0));
        assert_eq!(
            (hot.sun, hot.wave_a, hot.zenith),
            (water.sun, water.wave_a, water.zenith)
        );
        assert_eq!((hot.far, hot.blend), (water.far, blend::packed()));
        let home = Lands::of(&PlanetRecord::TEMPERATE);
        assert_eq!(water.far, home.sea_shown().extend(1.0));
    }
}
