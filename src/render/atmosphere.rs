//! The air of a planet: how it thins with height, the shadow the planet casts and the halo seen from space.

use bevy::asset::embedded_asset;
use bevy::math::DVec3;
use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use super::air::Air;
use super::bodies::Body;
use super::celestial::smoothstep;
use super::vantage::Scene;

/// Radius of the halo shell in planet radii.
pub const HALO_SCALE: f32 = 1.08;

/// Distances from the axis of the shadow of a planet, in planet radii, between which a point
/// behind it goes from its shadow into the light of the star.
pub const SHADOW_EDGE: (f32, f32) = (0.97, 1.03);

/// Scale heights in an atmosphere, as `crate::universe::TOP_SCALES` and `daylight.wesl` have it.
const TOP_SCALES: f32 = crate::universe::TOP_SCALES as f32;

/// Returns the share of the atmosphere left above `altitude` metres: 1 at the ground, 0 from
/// `height` metres up. A planet without air gives 0 everywhere.
///
/// The air thins as its pressure does, by a factor e over each scale height, a twelfth of
/// `height`: the same density the ship and the gear fly in.
pub fn air(altitude: f32, height: f32) -> f32 {
    if height <= 0.0 || altitude >= height {
        return 0.0;
    }
    (-TOP_SCALES * altitude.max(0.0) / height).exp()
}

/// Returns the sine of the angle the horizon drops by, seen `altitude` metres above a globe. It is
/// `sqrt(1 - (R / (R + h))^2)` written as `sqrt(h (2 R + h)) / (R + h)`, which keeps a height of a few
/// metres on a planet of any size.
pub fn horizon_dip(altitude: f32, radius: f32) -> f32 {
    let high = altitude.max(0.0);
    (high * (2.0 * radius + high)).sqrt() / (radius + high).max(1.0)
}

/// Returns the share of starlight that reaches a point `from` metres from the centre of a planet
/// of `radius` metres: 0 in its shadow, 1 in the open. `sun` is the unit vector toward the star.
pub fn shadow(from: DVec3, sun: DVec3, radius: f64) -> f32 {
    let along = from.dot(sun);
    if along >= 0.0 {
        return 1.0;
    }
    let miss = (from - sun * along).length() / radius;
    smoothstep(SHADOW_EDGE.0, SHADOW_EDGE.1, miss as f32)
}

/// Uniform block read by `atmosphere.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct HaloUniform {
    /// Linear glow colour in xyz and its strength in w.
    pub color: Vec4,
    /// Unit vector toward the star in xyz, and in w the facing of the shell where the planet limb shows.
    pub light: Vec4,
}

/// Material of the glowing shell around a planet with air.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct HaloMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub halo: HaloUniform,
}

impl Material for HaloMaterial {
    /// Points at the shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/render/atmosphere.wesl".into()
    }

    /// The glow adds to what lies behind it.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }

    /// The shell stays out of the depth prepass.
    fn enable_prepass() -> bool {
        false
    }
}

/// Lights every halo from its star, in the colour and strength of the air of its planet; the
/// halo of the planet below fades in above its air.
fn light_halos(
    scene: Scene,
    halos: Query<(&Body, &MeshMaterial3d<HaloMaterial>)>,
    mut materials: ResMut<Assets<HaloMaterial>>,
) {
    let vantage = &scene.vantage;
    let limb = (1.0 - 1.0 / (HALO_SCALE * HALO_SCALE)).sqrt();
    let local = scene.local.0.as_ref();
    let star = local.map_or(Vec3::ONE, |near| near.system.star.color);
    for (body, handle) in &halos {
        let Body::Halo(id) = *body else {
            continue;
        };
        let Some((planet, pos)) = scene.body(id) else {
            continue;
        };
        let glow = Air::of(&planet.record).halo(star);
        let leaving = smoothstep(1.0, 1.5, vantage.altitude / vantage.atmosphere.max(1.0));
        let strength = if vantage.is(id) { leaving } else { 1.0 };
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.halo = HaloUniform {
                color: glow.truncate().extend(glow.w * strength),
                light: vantage.sunward(pos).extend(limb),
            };
        }
    }
}

/// Embeds the shader and the daylight module, registers the material and lights the halos once
/// the camera is tracked.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "atmosphere.wesl");
    app.add_plugins((
        super::daylight::plugin,
        MaterialPlugin::<HaloMaterial>::default(),
    ))
    .add_systems(PostUpdate, light_halos.after(super::sun::track));
}
