//! The material of the foam: an alpha blended shader that moves and fades every quad itself.

use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

/// Uniform block read by `foam.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct FoamUniform {
    /// Linear colour of the foam in xyz and its greatest opacity in w.
    pub color: Vec4,
    /// The foam clock in seconds in x, the clock the quads were laid by.
    pub clock: Vec4,
}

/// Material of the foam mesh.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct FoamMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub foam: FoamUniform,
}

impl Material for FoamMaterial {
    /// Points at the vertex shader embedded in the binary.
    fn vertex_shader() -> ShaderRef {
        "embedded://abstractions/effects/water/foam.wesl".into()
    }

    /// Points at the fragment shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/effects/water/foam.wesl".into()
    }

    /// Foam blends over the sea, which writes its depth first.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    /// The foam stays out of the depth prepass.
    fn enable_prepass() -> bool {
        false
    }

    /// The foam never takes part in shadow rendering.
    fn enable_shadows() -> bool {
        false
    }
}
