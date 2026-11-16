//! The sky material: an unlit dome shader with a gradient, sun, moon, stars and clouds.

use bevy::asset::embedded_asset;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use super::camera::UNDERWATER;
use super::palette::Palette;

/// Uniform block read by `sky.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct SkyUniform {
    /// Linear sky color straight up.
    pub zenith: Vec4,
    /// Linear sky color at the horizon.
    pub horizon: Vec4,
    /// Sun direction in xyz.
    pub sun: Vec4,
    /// Moon direction in xyz.
    pub moon: Vec4,
    /// Linear cloud color in xyz.
    pub cloud: Vec4,
    /// Time in seconds, star visibility, cloud cover and sun and moon visibility.
    pub params: Vec4,
}

/// Converts an sRGB triple to a linear color vector.
fn linear(v: Vec3) -> Vec4 {
    Color::srgb(v.x, v.y, v.z).to_linear().to_vec4()
}

impl SkyUniform {
    /// Builds the uniform for a palette, a time, a cloud cover and whether the camera is underwater.
    pub fn new(palette: &Palette, time: f32, cover: f32, submerged: bool) -> Self {
        let (zenith, horizon, stars, cover, discs) = match submerged {
            true => (UNDERWATER, UNDERWATER, 0.0, 0.0, 0.0),
            false => (palette.zenith, palette.horizon, palette.stars, cover, 1.0),
        };
        Self {
            zenith: linear(zenith),
            horizon: linear(horizon),
            sun: palette.sun_dir.extend(0.0),
            moon: palette.moon_dir.extend(0.0),
            cloud: linear(palette.cloud),
            params: Vec4::new(time, stars, cover, discs),
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
