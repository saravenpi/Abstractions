//! The material of the cloud shell around the planet under the camera.

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

/// Uniform block read by `clouds.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, PartialEq, ShaderType)]
pub struct CloudUniform {
    /// Unit vector toward the star in xyz and the share of air above the camera in w.
    pub light: Vec4,
    /// Linear cloud color in xyz, and in w how far the cover is read from the first channel
    /// of the cover map toward the second, from 0 to 1.
    pub color: Vec4,
    /// Linear horizon color in xyz and the whiteout from 0 to 1 in w.
    pub haze: Vec4,
    /// Noise cells per radius of the middle of the band, where every slice reads the field, in x;
    /// in z 1 when the probe forces the cover, and that cover in w.
    pub params: Vec4,
    /// Linear colour of the sunlit clouds seen from space.
    pub space: Vec4,
    /// Linear colour of the sky light on the shaded side of a cloud in xyz.
    pub ambient: Vec4,
    /// The turn of the puffs round the spin axis, as `weather::drift` gives it: the cosine and the
    /// sine of the angle of the broad octave, then of the finer ones.
    pub drift: Vec4,
    /// First column of the rotation from render axes to planet-fixed axes.
    pub fixed_x: Vec4,
    /// Second column of that rotation.
    pub fixed_y: Vec4,
    /// Third column of that rotation.
    pub fixed_z: Vec4,
    /// Share of red, green and blue light the air takes per metre between the camera and the
    /// shell, in xyz: the aerial perspective that fades a far cloud into the horizon.
    pub fade: Vec4,
    /// Planet-fixed point of the shell under the camera, in noise cells, in xyz, before the turn
    /// of the puffs, and the metres of one noise cell in w: inside the cloud the whiteout is read
    /// from the field along each view ray from there.
    pub eye: Vec4,
    /// Height in the cloud band of the slice this shell draws, from 0 at the base to 1 at the
    /// top, in x, and the octaves of the noise it reads in y.
    pub slice: Vec4,
}

/// Material of the cloud shell.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct CloudMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub clouds: CloudUniform,
    /// The cover map of the planet: the broad cover of its cloud field, as a cube texture.
    #[texture(1, dimension = "cube")]
    #[sampler(2)]
    pub cover: Option<Handle<Image>>,
}

impl Material for CloudMaterial {
    /// Points at the shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/render/clouds.wesl".into()
    }

    /// The shell stays out of the depth prepass: the shader cuts holes the prepass would fill.
    fn enable_prepass() -> bool {
        false
    }

    /// Draws both faces so the layer shows from below and from above.
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
