//! The sky material: an unlit dome shader with a gradient that fades with height, the glow of the
//! sun, the moon, the horizon rim and the band of the galaxy.

use bevy::asset::embedded_asset;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use super::celestial::smoothstep;
use super::fog::UNDERWATER;
use super::palette::Palette;

/// Uniform block read by `sky.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct SkyUniform {
    /// Linear sky color straight up.
    pub zenith: Vec4,
    /// Linear sky color at the horizon.
    pub horizon: Vec4,
    /// Sun direction in xyz and the strength of its glow in the air in w.
    pub sun: Vec4,
    /// Moon direction in xyz and its visibility in w.
    pub moon: Vec4,
    /// Linear cloud color in xyz and how much of the view the cloud whiteout takes in w.
    pub cloud: Vec4,
    /// Local up in xyz and the sine of the angle the horizon drops by in w.
    pub up: Vec4,
    /// Linear color of the bright rim along the horizon, already scaled by its strength.
    pub rim: Vec4,
    /// Normal of the galaxy disc in xyz and the strength of its band in w.
    pub disc: Vec4,
    /// Direction of the galaxy center in xyz.
    pub core: Vec4,
    /// Linear color of the star light the air lets through, which colors the glow around the star.
    pub glow: Vec4,
    /// Unit vector of the direction of travel in xyz and, in w, the speed the sky is seen
    /// through in a warp as a share of the speed of light: the band crowds ahead with the stars.
    pub warp: Vec4,
}

/// What the dome is drawn from besides the palette, with directions in render axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkyLook {
    /// Unit vector away from the planet center.
    pub up: Vec3,
    /// Sine of the angle the horizon drops by at this height.
    pub dip: f32,
    /// Sine of the height of the star above the horizon.
    pub sun_height: f32,
    /// Share of the atmosphere left above the camera.
    pub air: f32,
    /// How deep the camera is in the cloud layer, from 0 to 1.
    pub whiteout: f32,
    /// sRGB color of the rim along the horizon, already scaled by its strength.
    pub rim: Vec3,
    /// Normal of the galaxy disc.
    pub disc: Vec3,
    /// Direction of the galaxy center.
    pub core: Vec3,
    /// Strength of the galaxy band.
    pub band: f32,
    /// Whether the camera is below the sea surface.
    pub submerged: bool,
    /// Unit vector of the direction of travel.
    pub heading: Vec3,
    /// Speed the sky is seen through in a warp, as a share of the speed of light.
    pub beta: f32,
}

/// Converts an sRGB triple to a linear color vector.
pub fn linear(v: Vec3) -> Vec4 {
    Color::srgb(v.x, v.y, v.z).to_linear().to_vec4()
}

impl SkyUniform {
    /// Builds the uniform for a palette and what the camera's place adds to it.
    ///
    /// Underwater the dome is one flat color without sun, moon, rim or band.
    pub fn new(palette: &Palette, look: &SkyLook) -> Self {
        let dry = if look.submerged { 0.0 } else { 1.0 };
        let (zenith, horizon) = match look.submerged {
            true => (UNDERWATER, UNDERWATER),
            false => (palette.zenith, palette.horizon),
        };
        let glow = smoothstep(-0.15, 0.05, look.sun_height) * look.air * dry;
        let moon = smoothstep(-0.15, 0.05, -look.sun_height) * look.air * dry;
        Self {
            zenith: linear(zenith),
            horizon: linear(horizon),
            sun: palette.sun_dir.extend(glow),
            moon: palette.moon_dir.extend(moon),
            cloud: linear(palette.cloud).truncate().extend(look.whiteout * dry),
            up: look.up.extend(look.dip),
            rim: linear(look.rim) * dry,
            disc: look.disc.extend(look.band * palette.stars * dry),
            core: look.core.extend(0.0),
            glow: linear(palette.sun_color),
            warp: look.heading.extend(look.beta),
        }
    }
}

/// Material of the sky dome.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SkyMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub sky: SkyUniform,
}

impl Material for SkyMaterial {
    /// Points at the shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/render/sky.wesl".into()
    }

    /// The dome stays out of the depth prepass.
    fn enable_prepass() -> bool {
        false
    }

    /// The dome never takes part in shadow rendering.
    fn enable_shadows() -> bool {
        false
    }

    /// Draws both faces so the dome shows from the inside whatever its winding.
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

/// Embeds the shader and registers the material.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "sky.wesl");
    app.add_plugins(MaterialPlugin::<SkyMaterial>::default());
}
