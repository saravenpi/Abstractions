//! The material of a body seen from afar: a glowing star or a planet lit by its star.

use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use crate::universe::{PlanetSpec, StarSpec};

/// Brightness of a star against plain white, enough for bloom to catch it.
const STAR_GAIN: f32 = 9.0;

/// Linear colour of the sea every planet is mostly covered by.
const SEA: Vec3 = Vec3::new(0.05, 0.22, 0.42);

/// Share of its own hue a planet shows through the sea colour.
const OWN_HUE: f32 = 0.4;

/// Uniform block read by `bodies.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct BodyUniform {
    /// Linear colour in xyz and the glow gain in w: above 0 the body shines by itself.
    pub color: Vec4,
    /// Unit vector from the body toward its star, in render axes.
    pub light: Vec4,
}

/// Material of a star or a far planet.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct BodyMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub body: BodyUniform,
}

/// Returns the linear colour of a planet seen from afar: sea blue shifted toward a hue of its seed.
pub fn tint(seed: u64) -> Vec3 {
    let mixed = (seed ^ (seed >> 29)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let hue = (mixed >> 40) as f32 / (1u64 << 24) as f32 * 360.0;
    let own = Color::hsl(hue, 0.5, 0.5).to_linear().to_vec3();
    SEA.lerp(own, OWN_HUE)
}

impl BodyMaterial {
    /// Builds the material of a star, glowing in its own colour.
    pub fn star(star: &StarSpec) -> Self {
        Self {
            body: BodyUniform {
                color: star.color.extend(STAR_GAIN),
                light: Vec4::ZERO,
            },
        }
    }

    /// Builds the material of a planet, which shows no light until it is lit.
    pub fn planet(planet: &PlanetSpec) -> Self {
        Self {
            body: BodyUniform {
                color: tint(planet.seed).extend(0.0),
                light: Vec4::ZERO,
            },
        }
    }
}

impl Material for BodyMaterial {
    /// Points at the shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/render/bodies.wesl".into()
    }

    /// Bodies stay out of the depth prepass, like the sky behind them.
    fn enable_prepass() -> bool {
        false
    }
}
