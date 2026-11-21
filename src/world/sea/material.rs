//! The water material: an alpha blended shader with vertex waves, foam, and reflections of the
//! sky and of the actors.

use bevy::asset::embedded_asset;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use super::order::{FIRST, surface};
use super::uniform::WaterUniform;
use crate::render::PlaceUniform;

/// Material of the sea surface.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub water: WaterUniform,
    /// Sea floor depth map around the focus: depth in red and green, pond flag in blue.
    #[texture(1)]
    #[sampler(2)]
    pub floor: Handle<Image>,
    /// The light of the planet under the camera, so that sea seen from far is lit as its place is.
    #[uniform(3)]
    pub place: PlaceUniform,
    /// The actors the mirror camera drew under the water, lined up with the game view, or one
    /// see-through pixel while it does not draw. The shader reads it texel by texel, with no
    /// sampler: Metal allows 16 samplers in a fragment shader and the view takes most of them.
    #[texture(4)]
    pub mirror: Handle<Image>,
}

impl Material for WaterMaterial {
    /// Points at the vertex shader embedded in the binary.
    fn vertex_shader() -> ShaderRef {
        "embedded://abstractions/world/sea/water.wesl".into()
    }

    /// Points at the fragment shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/world/sea/water.wesl".into()
    }

    /// The sea blends with what lies behind it, so it draws in the transparent pass.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    /// Puts the sea before every other see-through thing.
    fn depth_bias(&self) -> f32 {
        FIRST
    }

    /// The sea stays out of the depth prepass so the shader can read the depth of what lies below.
    fn enable_prepass() -> bool {
        false
    }

    /// The sea never takes part in shadow rendering.
    fn enable_shadows() -> bool {
        false
    }

    /// Draws both faces and writes depth, see [`surface`].
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        surface(descriptor);
        Ok(())
    }
}

/// Embeds the shader and registers the material.
pub fn plugin(app: &mut App) {
    embedded_asset!(app, "water.wesl");
    app.add_plugins(MaterialPlugin::<WaterMaterial>::default());
}
