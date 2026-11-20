//! The material of the foam: an alpha blended shader that moves and fades every quad itself.

use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use super::foam::{Foam, FoamMesh};
use super::tune::FOAM_OPACITY;
use crate::space::Anchor;
use crate::world::wake::{FoamWakes, Wakes};
use crate::world::{Liquid, Terrain};

/// Uniform block read by `foam.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct FoamUniform {
    /// Linear colour of the foam in xyz and its greatest opacity in w.
    pub color: Vec4,
    /// The foam clock in seconds in x, the clock the quads were laid by.
    pub clock: Vec4,
    /// The live wakes of the sea, in the space of the foam mesh.
    pub wake: FoamWakes,
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

/// Sets the clock and the colour of the foam and shows its mesh while some quad still lives.
pub fn tick(
    (time, terrain, anchor): (Res<Time>, Res<Terrain>, Res<Anchor>),
    (foam, wakes): (Res<Foam>, Res<Wakes>),
    mut materials: ResMut<Assets<FoamMaterial>>,
    mut meshes: Query<(&mut Visibility, &Transform), With<FoamMesh>>,
) {
    let now = time.elapsed_secs();
    let shown = [Visibility::Hidden, Visibility::Inherited][usize::from(now < foam.until)];
    let mut origin = Vec3::ZERO;
    for (mut visibility, at) in &mut meshes {
        visibility.set_if_neq(shown);
        origin = at.translation;
    }
    if now >= foam.until {
        return;
    }
    if let Some(mut material) = materials.get_mut(&foam.material) {
        let color = Liquid::of(terrain.globe()).foam;
        material.foam = FoamUniform {
            color: color.extend(FOAM_OPACITY),
            clock: Vec4::new(now, 0.0, 0.0, 0.0),
            wake: FoamWakes::pack(&wakes, now, |at| anchor.to_render(at) - origin),
        };
    }
}
