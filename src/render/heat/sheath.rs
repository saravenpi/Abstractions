//! The sheath: the glowing shock layer of an entry around a craft, seen from outside and through
//! the canopy alike. A bow shock wraps the nose, the stagnation point and the leading edges of
//! the wings burn hottest in the colour of their temperature, streamers peel off the wing tips
//! and the tail, and a long faint wake of ionised air trails behind. The shock also lights the
//! hull from the nose.
//!
//! The shader integrates the light of the glowing gas along each view ray in closed form and
//! stops the ray at the depth of the scene, so the hull and the ground hide what lies behind
//! them and nothing is drawn in specks. Cost: one sphere of 512 triangles, one fragment per
//! pixel it covers with about forty Gaussian line integrals each, and one point light without
//! shadows, all only while the craft glows.

use bevy::asset::embedded_asset;
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
use super::tune::{HULL_LUMENS, SHEATH_FROM};
use crate::render::reach;
use crate::screens::Screen;

/// Uniform block read by `sheath.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct SheathUniform {
    /// Centre of the craft in xyz and its radius in w.
    pub centre: Vec4,
    /// Unit vector of its flight in xyz.
    pub axis: Vec4,
    /// Unit vector out of its roof in xyz.
    pub roof: Vec4,
    /// Linear colour of the glowing gas in xyz and the strength of the glow in w.
    pub glow: Vec4,
    /// Linear colour of the hot skin in xyz and the share of the full pressure of the air in w.
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

/// Marks the light the shock casts on the hull from the nose.
#[derive(Component, Debug, Clone, Copy)]
pub struct HullLight;

/// Spawns the hidden sheath and its dark light when the world starts loading.
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
    let light = PointLight {
        intensity: 0.0,
        range: reach(HULL_LUMENS),
        radius: 0.5,
        shadow_maps_enabled: false,
        ..default()
    };
    let hidden = (Transform::default(), Visibility::Hidden);
    commands.spawn((HullLight, light, hidden, DespawnOnEnter(Screen::Title)));
}

/// The sheath and the hull light, each with its pose and whether it shows.
type Posed = (
    &'static mut Transform,
    &'static mut GlobalTransform,
    &'static mut Visibility,
);

/// Filter of the light ahead of the nose, apart from the sheath.
type HullOnly = (With<HullLight>, Without<Sheath>);

/// Moves the sheath to the craft and shows it while the craft glows, and lights the hull from
/// just ahead of the nose with the colour and the strength of the glow.
fn follow(
    heat: Res<Heat>,
    mut sheaths: Query<(Posed, &MeshMaterial3d<SheathMaterial>), With<Sheath>>,
    mut lights: Query<(Posed, &mut PointLight), HullOnly>,
    mut materials: ResMut<Assets<SheathMaterial>>,
) {
    let (craft, glow) = (heat.craft, heat.glow);
    let shown = glow.heat > SHEATH_FROM;
    let sight = [Visibility::Hidden, Visibility::Visible][usize::from(shown)];
    for ((mut transform, mut global, mut visibility), handle) in &mut sheaths {
        visibility.set_if_neq(sight);
        if !shown {
            continue;
        }
        crate::render::pin(craft.bound(), &mut transform, &mut global);
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.sheath = craft.sheath(&glow);
        }
    }
    let nose = Transform::from_translation(craft.centre + craft.heading * (craft.radius * 1.2));
    let tint = glow.color.lerp(glow.skin, 0.5);
    for ((mut transform, mut global, mut visibility), mut light) in &mut lights {
        visibility.set_if_neq(sight);
        crate::render::pin(nose, &mut transform, &mut global);
        light.intensity = HULL_LUMENS * glow.heat * f32::from(u8::from(shown));
        light.color = Color::linear_rgb(tint.x, tint.y, tint.z);
    }
}

/// Embeds the shader, registers the material and places the sheath after the heat is eased.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "sheath.wesl");
    app.add_plugins(MaterialPlugin::<SheathMaterial>::default())
        .add_systems(OnEnter(Screen::Loading), spawn)
        .add_systems(
            PostUpdate,
            follow.run_if(resource_changed::<Heat>).after(HeatSet),
        );
}
