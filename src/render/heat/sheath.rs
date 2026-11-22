//! The sheath: the glowing shock layer of an entry around a craft, seen from outside and through
//! the canopy alike. A bow shock wraps the nose, the stagnation point and the leading edges of
//! the wings burn hottest in the colour of their temperature, streamers peel off the wing tips
//! and the tail, and a long faint wake of ionised air trails behind. The plasma fades with the
//! flux; the hot skin of the nose and the edges cools after it.
//!
//! The shader integrates the light of the glowing gas along each view ray in closed form and
//! stops the ray at the depth of the scene, so the hull and the ground hide what lies behind
//! them and nothing is drawn in specks. Cost: one sphere of 512 triangles, one fragment per
//! pixel it covers with about fifteen line integrals of glowing tubes each, only while the craft
//! glows. The hot nose and leading edges are lines along the hull, never spots.

use bevy::asset::{AssetEventSystems, embedded_asset};
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, CompareFunction, Face, RenderPipelineDescriptor, ShaderType,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use super::state::{Heat, HeatSet};
use crate::screens::Screen;

/// Uniform block read by `sheath.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct SheathUniform {
    /// Centre of the craft in xyz and its radius in w.
    pub centre: Vec4,
    /// Unit vector of its flight in xyz.
    pub axis: Vec4,
    /// Unit vector out of its roof in xyz and the share of the full pressure of the air in w.
    pub roof: Vec4,
    /// Linear colour of the glowing gas in xyz and the strength of the plasma in w.
    pub glow: Vec4,
    /// Linear colour of the hot skin in xyz and the strength of its glow in w.
    pub skin: Vec4,
}

/// Material of the sheath.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct SheathMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub sheath: SheathUniform,
}

impl Material for SheathMaterial {
    /// Points at the shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/render/heat/sheath.wesl".into()
    }

    /// The sheath adds light to the view.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }

    /// The sheath stays out of the depth prepass, which its shader reads.
    fn enable_prepass() -> bool {
        false
    }

    /// The sheath casts no shadow.
    fn enable_shadows() -> bool {
        false
    }

    /// Draws the far side of the sphere, which every view ray through it meets once whether
    /// the camera is inside or outside, and skips the depth test: the far side lies behind the
    /// hull, and the shader stops each ray at the depth of the scene itself.
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = Some(Face::Front);
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = Some(false);
            depth.depth_compare = Some(CompareFunction::Always);
        }
        Ok(())
    }
}

/// Marks the one sheath, which is made once and only moved afterwards.
#[derive(Component, Debug, Clone, Copy)]
pub struct Sheath;

/// Spawns the hidden sheath when the world starts loading.
fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SheathMaterial>>,
) {
    commands.spawn((
        Sheath,
        Mesh3d(meshes.add(Sphere::new(1.0).mesh().uv(32, 16))),
        MeshMaterial3d(materials.add(SheathMaterial::default())),
        Transform::default(),
        Visibility::Hidden,
        NotShadowCaster,
        NoFrustumCulling,
        DespawnOnEnter(Screen::Title),
    ));
}

/// Moves the sheath to the craft and shows it while its plasma or its skin glows at all: it
/// fades in and out with them and never pops.
fn follow(
    heat: Res<Heat>,
    mut sheaths: Query<
        (
            &mut Transform,
            &mut GlobalTransform,
            &mut Visibility,
            &MeshMaterial3d<SheathMaterial>,
        ),
        With<Sheath>,
    >,
    mut materials: ResMut<Assets<SheathMaterial>>,
) {
    let (craft, glow) = (heat.craft, heat.glow);
    let shown = glow.heat > 0.0 || glow.shine > 0.0;
    let sight = [Visibility::Hidden, Visibility::Visible][usize::from(shown)];
    for (mut transform, mut global, mut visibility, handle) in &mut sheaths {
        visibility.set_if_neq(sight);
        if !shown {
            continue;
        }
        crate::render::pin(craft.bound(), &mut transform, &mut global);
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.sheath = craft.sheath(&glow);
        }
    }
}

/// Embeds the shader, registers the material and places the sheath after the heat is eased.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "sheath.wesl");
    app.add_plugins(MaterialPlugin::<SheathMaterial>::default())
        .add_systems(OnEnter(Screen::Loading), spawn)
        .add_systems(
            PostUpdate,
            follow
                .run_if(resource_changed::<Heat>)
                .after(HeatSet)
                .before(AssetEventSystems),
        );
}
