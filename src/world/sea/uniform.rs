//! The values the water shader reads: sky colors, light, fog and waves, refreshed every frame.

use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

use super::waves::waves_for;
use crate::render::Palette;
use crate::wind::Wind;

/// Uniform block read by `water.wesl`. Every field is a `vec4` so the layout never needs padding.
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
    /// World x and z of the sea floor map center; the rest is unused.
    pub map: Vec4,
}

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
    /// Builds the uniform for a palette, the wind, a time in seconds and the camera fog.
    pub fn new(palette: &Palette, wind: &Wind, time: f32, fog: &FogLook) -> Self {
        let [a, b, c, d] = waves_for(wind).map(|wave| wave.packed());
        let sun_up = smooth(-0.05, 0.12, palette.sun_dir.y);
        Self {
            zenith: linear(palette.zenith),
            horizon: linear(palette.horizon),
            fog: fog.color,
            sun: palette.sun_dir.extend(sun_up),
            sun_color: linear(palette.sun_color).truncate().extend(smooth(
                -0.1,
                0.3,
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
            map: Vec4::ZERO,
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
}
