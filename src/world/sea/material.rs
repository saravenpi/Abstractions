//! The water material: an alpha blended shader with vertex waves, foam and sky reflections.

use bevy::asset::embedded_asset;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use super::uniform::WaterUniform;

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

    /// The sea stays out of the depth prepass so the shader can read the depth of what lies below.
    fn enable_prepass() -> bool {
        false
    }

    /// The sea never takes part in shadow rendering.
    fn enable_shadows() -> bool {
        false
    }

    /// Draws both faces so the surface shows from below.
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
pub fn plugin(app: &mut App) {
    embedded_asset!(app, "water.wesl");
    app.add_plugins(MaterialPlugin::<WaterMaterial>::default());
}
