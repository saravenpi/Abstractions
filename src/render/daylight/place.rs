//! The light at any point of one planet: what its star, its air and its own shadow give the
//! ground there, whoever looks at it. `light_at` in `daylight.wesl` is the same rule on the GPU.

use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

use super::{NIGHT_EDGE, daylight};
use crate::render::air::Air;
use crate::render::atmosphere::{SHADOW_EDGE, air, horizon_dip};
use crate::render::celestial::smoothstep;
use crate::render::palette::{NIGHT_AMBIENT_COLOR, Palette};
use crate::render::scatter::peak;
use crate::render::sky::linear;

/// The star and the air one planet is lit through.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sky {
    /// Linear colour of the star.
    pub star: Vec3,
    /// Home atmospheres of air the light crosses straight down at sea level.
    pub depth: f32,
    /// Share of a home sky the air makes at sea level, 0 without air.
    pub share: f32,
    /// Sea level radius of the planet in metres.
    pub radius: f32,
    /// Height above sea level in metres where the air ends.
    pub atmosphere: f32,
    /// Ambient brightness the stars add where the night is complete.
    pub starlight: f32,
    /// Ambient colour of the full day, in sRGB.
    pub day: Vec3,
}

/// The light one point of a planet receives, in the units the lights of the scene carry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light {
    /// Linear colour of the direct light of the star times its illuminance in lux.
    pub sun: Vec3,
    /// Linear colour of the ambient light times its brightness.
    pub sky: Vec3,
}

/// Uniform block of the light of the planet under the camera, read through `daylight.wesl`.
/// Every field is a `vec4` so the layout never needs padding. All zero, it changes nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, ShaderType)]
pub struct PlaceUniform {
    /// Render position of the planet centre in xyz and its sea level radius in metres in w.
    pub centre: Vec4,
    /// Unit vector toward the star in render axes in xyz, and the depth of the air in w.
    pub star: Vec4,
    /// Linear colour of the star in xyz, and in w the share of a home sky the air makes.
    pub colour: Vec4,
    /// Height in metres where the air ends in x, the ambient brightness of the stars at night
    /// in y, and in z 1 when a planet is lit this way and 0 when the block is to be ignored.
    pub air: Vec4,
    /// Ambient colour of the full day in sRGB.
    pub day: Vec4,
    /// Direct light this rule gives at the camera, which the lights of the scene carry.
    pub eye_sun: Vec4,
    /// Ambient light this rule gives at the camera, which the ambient light of the scene carries.
    pub eye_sky: Vec4,
}

impl PlaceUniform {
    /// Fields of the block, each one texel where a shader reads it from a texture.
    pub const FIELDS: usize = 7;

    /// Bytes of the block as texels of four 32 bit floats.
    pub const BYTES: usize = 16 * Self::FIELDS;

    /// Returns the fields in the order of the block, which `place_from` in `daylight.wesl`
    /// reads texel by texel.
    pub fn fields(&self) -> [Vec4; Self::FIELDS] {
        [
            self.centre,
            self.star,
            self.colour,
            self.air,
            self.day,
            self.eye_sun,
            self.eye_sky,
        ]
    }
}

/// Returns the share of starlight that reaches a point `altitude` metres above the sea level of
/// a planet of `radius` metres, where the star stands at the sine `height` over level ground:
/// [`crate::render::atmosphere::shadow`] for a point given by those two numbers.
pub fn shadow_at(height: f32, altitude: f32, radius: f32) -> f32 {
    if height >= 0.0 {
        return 1.0;
    }
    let miss = (1.0 + altitude / radius.max(1.0)) * (1.0 - height * height).max(0.0).sqrt();
    smoothstep(SHADOW_EDGE.0, SHADOW_EDGE.1, miss)
}

impl Sky {
    /// Returns the sky of a planet of `radius` metres whose air `air` ends `atmosphere` metres
    /// up, under a star of linear colour `star`, on a night the stars light by `starlight`.
    pub fn of(star: Vec3, air: &Air, (radius, atmosphere): (f32, f32), starlight: f32) -> Self {
        Self {
            star,
            depth: air.depth(),
            share: air.share(),
            radius,
            atmosphere,
            starlight,
            day: Palette::through(1.0, star, air).ambient,
        }
    }

    /// Returns the light at a point `altitude` metres above sea level where the star stands at
    /// the sine `height` over level ground.
    ///
    /// It is what the palette gives a camera standing at that point: the daylight of the place
    /// through the air left above it, the open star where there is no air, the shadow of the
    /// planet, and by night the ambient light of the night sky and the stars.
    pub fn light(&self, height: f32, altitude: f32) -> Light {
        let seen = height + horizon_dip(altitude, self.radius);
        let day = daylight(seen, self.star, self.depth);
        let air = air(altitude, self.atmosphere) * self.share;
        let open = Palette::SPACE_LUX * shadow_at(height, altitude, self.radius) * (1.0 - air);
        let lux = open + Palette::SPACE_LUX * day.through * day.risen * air;
        let night = smoothstep(NIGHT_EDGE.0, NIGHT_EDGE.1, -seen) * air;
        let swing = Palette::DAY_AMBIENT - Palette::NIGHT_AMBIENT;
        let thin = Palette::SPACE_AMBIENT + (1.0 - Palette::SPACE_AMBIENT) * air;
        let level = (Palette::NIGHT_AMBIENT + swing * day.sky) * thin + self.starlight * night;
        let colour = NIGHT_AMBIENT_COLOR.lerp(self.day, day.sky);
        Light {
            sun: linear(peak(self.star).lerp(day.tint, air)).truncate() * lux,
            sky: linear(colour).truncate() * level,
        }
    }

    /// Returns the uniform block for this sky around a planet whose centre is at `centre` in
    /// render space, lit from the unit vector `toward`, seen by a camera `altitude` metres up
    /// for which the star stands at the sine `height` over level ground.
    pub fn uniform(
        &self,
        centre: Vec3,
        toward: Vec3,
        (height, altitude): (f32, f32),
    ) -> PlaceUniform {
        let eye = self.light(height, altitude);
        PlaceUniform {
            centre: centre.extend(self.radius),
            star: toward.extend(self.depth),
            colour: self.star.extend(self.share),
            air: Vec4::new(self.atmosphere, self.starlight, 1.0, 0.0),
            day: self.day.extend(0.0),
            eye_sun: eye.sun.extend(0.0),
            eye_sky: eye.sky.extend(0.0),
        }
    }
}
