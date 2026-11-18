//! The rings of a planet: a flat annulus in the plane of its equator, dithered to its density so
//! it needs no transparent sorting, shaded by the planet in `rings.wesl`.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::math::DVec3;
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use crate::universe::{PlanetSpec, Rings};

/// Segments around the annulus: from the deck of a ringed giant the inner edge stays round.
const SEGMENTS: u32 = 256;

/// Uniform block read by `rings.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct RingUniform {
    /// Linear colour of the ring material in xyz and its density in w.
    pub color: Vec4,
    /// Unit vector from the planet toward its star in the axes of the rings, which are the
    /// axes of the star system.
    pub light: Vec4,
    /// Inner radius of the rings in x and radius of the planet in y, both over the outer radius.
    pub shape: Vec4,
}

/// Material of the rings of a planet.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct RingMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub ring: RingUniform,
}

impl RingMaterial {
    /// Builds the material of the rings of `planet`, unlit until its star is known.
    pub fn of(planet: &PlanetSpec, rings: &Rings) -> Self {
        let shape = Vec2::new(
            (rings.inner / rings.outer) as f32,
            (planet.radius / rings.outer) as f32,
        );
        Self {
            ring: RingUniform {
                color: rings.color.extend(rings.density),
                light: Vec4::ZERO,
                shape: shape.extend(0.0).extend(0.0),
            },
        }
    }

    /// Lights the rings of a planet `pos` metres from the star.
    pub fn light(&mut self, pos: DVec3) {
        self.ring.light = (-pos).normalize_or(DVec3::X).as_vec3().extend(0.0);
    }
}

/// Returns a flat annulus in the XZ plane from radius `inner` to 1, facing +Y, whose UV is its
/// own (x, z) so the shader knows the radius and the side of every point.
pub fn annulus(inner: f32) -> Mesh {
    let count = 2 * (SEGMENTS as usize + 1);
    let (mut positions, mut uvs) = (Vec::with_capacity(count), Vec::with_capacity(count));
    for step in 0..=SEGMENTS {
        let (sin, cos) = (TAU * step as f32 / SEGMENTS as f32).sin_cos();
        for radius in [inner, 1.0] {
            positions.push([cos * radius, 0.0, sin * radius]);
            uvs.push([cos * radius, sin * radius]);
        }
    }
    let indices = (0..SEGMENTS)
        .flat_map(|step| {
            let k = 2 * step;
            [k, k + 2, k + 1, k + 1, k + 2, k + 3]
        })
        .collect();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh.enable_raytracing = false;
    mesh
}

impl Material for RingMaterial {
    /// Points at the shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/render/rings.wesl".into()
    }

    /// The rings stay out of the depth prepass, like the bodies.
    fn enable_prepass() -> bool {
        false
    }

    /// Draws both faces: the rings are seen from above and from below their plane.
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
