//! The material of a body seen from afar: a glowing star or a planet lit by its star.

use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use crate::universe::{PlanetRecord, PlanetSpec, StarSpec};

/// Brightness of a star against plain white, enough for bloom to catch it.
const STAR_GAIN: f32 = 9.0;

/// How far the seed of a planet moves its share of sea seen from afar, up or down.
const SEA_SWING: f32 = 0.06;

/// How far the seed of a planet moves its brightness seen from afar, up or down.
const SHADE_SWING: f32 = 0.1;

/// Light a glowing sea sends out by itself, against the glow of its class.
const EMBER: f32 = 0.3;

/// Least share of a glowing world that shines from afar: the cracks glow where no pool shows.
const EMBER_FLOOR: f32 = 0.15;

/// Uniform block read by `bodies.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct BodyUniform {
    /// Linear colour in xyz and the glow gain in w: above 0 the body shines by itself.
    pub color: Vec4,
    /// Unit vector from the body toward its star, in render axes.
    pub light: Vec4,
    /// Linear colour the body gives off on top of the light it receives, night side included.
    pub ember: Vec4,
}

/// Material of a star or a far planet.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct BodyMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub body: BodyUniform,
}

/// A planet seen from afar, in linear colours.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Far {
    /// Colour of the surface under full light.
    pub color: Vec3,
    /// Light the surface gives off by itself, which the night side keeps.
    pub ember: Vec3,
}

/// Returns two shares from 0 to 1 drawn from the seed of a planet.
fn shares(seed: u64) -> (f32, f32) {
    let mixed = (seed ^ (seed >> 29)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let unit = |bits: u64| (bits & 0xFFF) as f32 / 4096.0;
    (unit(mixed >> 52), unit(mixed >> 40))
}

/// Returns how a planet shows from afar: the land and the sea of its class mixed by the share of
/// its surface under liquid, and the glow of a molten sea. The seed moves the mix and the
/// brightness a little, so two planets of one class are not twins.
pub fn far(record: &PlanetRecord, seed: u64) -> Far {
    let look = record.class.look();
    let (wet, shade) = shares(seed);
    let sea = (record.ocean as f32 + SEA_SWING * (2.0 * wet - 1.0)).clamp(0.0, 1.0);
    let bright = 1.0 + SHADE_SWING * (2.0 * shade - 1.0);
    Far {
        color: look.land.lerp(look.sea, sea) * bright,
        ember: look.sea * (look.glow * EMBER * sea.max(EMBER_FLOOR)),
    }
}

impl BodyMaterial {
    /// Builds the material of a star, glowing in its own colour.
    pub fn star(star: &StarSpec) -> Self {
        Self {
            body: BodyUniform {
                color: star.color.extend(STAR_GAIN),
                ..default()
            },
        }
    }

    /// Builds the material of a planet, which shows only its own glow until it is lit.
    pub fn planet(planet: &PlanetSpec) -> Self {
        let far = far(&planet.record, planet.seed);
        Self {
            body: BodyUniform {
                color: far.color.extend(0.0),
                light: Vec4::ZERO,
                ember: far.ember.extend(0.0),
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
