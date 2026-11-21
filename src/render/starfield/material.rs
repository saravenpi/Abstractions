//! The material of the starfield: every speck is moved, stretched and tinted by its shader, so
//! one mesh serves the sky at rest, the sky a camera drifts through and the sky of a warp.

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

/// Uniform block read by `starfield.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct StarUniform {
    /// Visibility of the sky from 0 to 1 in x, the light-years the ship flies during one
    /// exposure in y, which every star streaks by from its own distance and bearing, and the
    /// share of their light single stars keep in z.
    pub params: Vec4,
    /// Unit vector of the direction of travel in render axes in xyz, and in w the speed the sky
    /// is seen through as a share of the speed of light.
    pub heading: Vec4,
    /// How far the camera has moved since the mesh was built, in light-years in galaxy axes.
    pub drift: Vec4,
    /// The dust of a warp: the laps the flow has run in x, its laps per second in y and the
    /// share of the dust that is in in z, which is nothing outside a galaxy.
    pub flow: Vec4,
    /// Unit vector away from the planet in render axes in xyz, and in w the luminance of the sky
    /// straight up, in linear light: what hides the stars, see `skylight`.
    pub dome: Vec4,
    /// The luminance of the sky at the horizon in x, in linear light.
    pub base: Vec4,
    /// Unit vector toward the star of the system in render axes in xyz, and in w the strength of
    /// its glare, which outshines the stars around its disc: see `glare`.
    pub sun: Vec4,
}

/// Material of the starfield.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct StarMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub stars: StarUniform,
}

impl Material for StarMaterial {
    /// Points at the shader embedded in the binary, which places every speck.
    fn vertex_shader() -> ShaderRef {
        "embedded://abstractions/render/starfield.wesl".into()
    }

    /// Points at the shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/render/starfield.wesl".into()
    }

    /// Starlight adds to the sky behind it.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }

    /// The stars stay out of the depth prepass.
    fn enable_prepass() -> bool {
        false
    }

    /// Draws both faces: the shader turns every speck along its streak, whatever its winding.
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
