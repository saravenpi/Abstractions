//! The material of a body seen from afar: a glowing star, or a planet lit by its star as the ground
//! is, drawn from the baked texture of its real surface once that is ready.

use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use super::bake::shell_radius;
use super::giant::Giant;
use crate::render::SeasonUniform;
use crate::render::clouds::CELL;
use crate::render::daylight::PlaceUniform;
use crate::universe::{PlanetRecord, PlanetSpec, StarSpec};
use crate::world::globe::far::Ball;

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
#[derive(Clone, Copy, Debug, Default, PartialEq, ShaderType)]
pub struct BodyUniform {
    /// Linear colour in xyz and the glow gain in w: above 0 the body shines by itself.
    pub color: Vec4,
    /// Unit vector from the body toward its star, in render axes, and in w the share of the
    /// starlight the other bodies of its planet let through to it.
    pub light: Vec4,
    /// Linear colour the body gives off on top of the light it receives, night side included.
    pub ember: Vec4,
    /// Spin axis in render axes, and in w the number of bands of a gas giant, 0 for any other body.
    pub axis: Vec4,
    /// Linear colour of the dark belts of a gas giant, and in w the phase of its bands.
    pub tone: Vec4,
    /// Unit vector toward the storm of a gas giant in render axes, and in w the cosine of its radius.
    pub storm: Vec4,
    /// Linear colour of the storm of a gas giant.
    pub spot: Vec4,
    /// Inner and outer radius of the rings in planet radii and their density, 0 without rings.
    pub rings: Vec4,
    /// In z how much of the baked surface shows, from 0 for a dot in its mean colour to 1 for a
    /// disc, and 0 until the ball is baked; in w the cells of the cloud noise along the radius of
    /// the cloud shell.
    pub map: Vec4,
    /// Linear light the flat sea of the baked surface gives off by itself, and in w 1 once the
    /// surface texture is baked.
    pub glow: Vec4,
    /// Linear colour of the light of the star times its illuminance in lux, as the sun light of
    /// the ground has it in space, and in w 1 while the clouds show, 0 without clouds and
    /// until the cover map is ready.
    pub sun: Vec4,
    /// Linear colour of the sunlit clouds, and in w how far the cover is read from the first
    /// channel of the cover map toward the second, from 0 to 1.
    pub cloud: Vec4,
    /// First column of the rotation from render axes to the axes fixed to the body.
    pub fixed_x: Vec4,
    /// Second column of that rotation.
    pub fixed_y: Vec4,
    /// Third column of that rotation.
    pub fixed_z: Vec4,
    /// Cells of the noise the puffs of the clouds have drifted by, as `weather::drift` gives them.
    pub drift: Vec4,
}

/// Material of a star or a far planet.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct BodyMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub body: BodyUniform,
    /// The cover map of the body: the broad cover of its cloud field, as a cube texture. The
    /// cloud shell around the planet under the camera reads the same one.
    #[texture(1, dimension = "cube")]
    #[sampler(2)]
    pub cover: Option<Handle<Image>>,
    /// The baked surface of the body: six layers, one per face of its cube sphere, the linear
    /// colour in sRGB and the share of sea in alpha, with mipmaps. Nothing until it is baked.
    #[texture(3, dimension = "2d_array")]
    #[sampler(4)]
    pub surface: Option<Handle<Image>>,
    /// The star and the air the body is lit through, so its day and its night are its own and
    /// not those of the camera. All zero, a planet stays unlit and shows only its own glow.
    #[uniform(10)]
    pub place: PlaceUniform,
    /// The season of the planet, which turns the colours of its baked ground as it turns the ground of
    /// the world. All zero, the colours stay as they were baked.
    #[uniform(11)]
    pub season: SeasonUniform,
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
pub(super) fn shares(seed: u64) -> (f32, f32) {
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
            cover: None,
            surface: None,
            place: default(),
            season: default(),
        }
    }

    /// Builds the material of a planet or moon, which shows only its own glow until it is lit.
    ///
    /// A gas giant shows its bands and its storm instead of land and sea; ringed planets carry
    /// the size of their rings, whose shadow falls on them.
    pub fn planet(planet: &PlanetSpec) -> Self {
        let far = far(&planet.record, planet.seed);
        let giant = Giant::of(&planet.record, planet.seed);
        let rings = planet.rings.map_or(Vec4::ZERO, |rings| {
            let (inner, outer) = (rings.inner / planet.radius, rings.outer / planet.radius);
            Vec4::new(inner as f32, outer as f32, rings.density, 0.0)
        });
        Self {
            body: BodyUniform {
                color: giant.map_or(far.color, |giant| giant.first).extend(0.0),
                light: Vec4::ZERO,
                ember: far.ember.extend(0.0),
                axis: Vec4::new(0.0, 1.0, 0.0, giant.map_or(0.0, |giant| giant.bands)),
                tone: giant.map_or(Vec4::ZERO, |giant| giant.second.extend(giant.phase)),
                storm: giant.map_or(Vec4::ZERO, |giant| giant.spot.as_vec3().extend(giant.size)),
                spot: giant.map_or(Vec4::ZERO, |giant| giant.storm.extend(0.0)),
                rings,
                ..default()
            },
            cover: None,
            surface: None,
            place: default(),
            season: default(),
        }
    }
}

impl BodyMaterial {
    /// Turns this material into the one of the baked surface `ball` of `body` drawn from
    /// `surface`: the mean colour and the mean glow of the real surface for the dot, the light of
    /// its sea for the disc, and the cloud noise along the radius of its cloud shell.
    pub(super) fn baked(&mut self, body: &PlanetSpec, ball: &Ball, surface: Handle<Image>) {
        let cells = shell_radius(body).map_or(0.0, |shell| shell as f32 / CELL);
        let uniform = &mut self.body;
        uniform.color = ball.mean.extend(0.0);
        uniform.ember = (ball.glow * ball.sea).extend(0.0);
        uniform.map = Vec4::new(0.0, 0.0, 0.0, cells);
        uniform.glow = ball.glow.extend(1.0);
        self.surface = Some(surface);
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
