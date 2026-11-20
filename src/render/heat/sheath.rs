//! The sheath: the glowing shock around the nose of a craft in an entry, with plasma streaming
//! back along the hull, embers peeling off and a short tail, seen from outside the craft.

use bevy::asset::embedded_asset;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use super::state::{Heat, HeatSet};
use super::tune::{SHEATH_FROM, SHEATH_REACH};
use crate::render::Vantage;
use crate::screens::Screen;

/// Share of the radius of the craft the camera must be farther than for the sheath to show: from
/// inside the craft, the glass of the cockpit glows instead.
const OUTSIDE: f32 = 1.5;

/// Uniform block read by `sheath.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct SheathUniform {
    /// Centre of the craft in xyz and its radius in w.
    pub centre: Vec4,
    /// Unit vector of its flight in xyz.
    pub axis: Vec4,
    /// Linear colour of the glow in xyz and its strength in w.
    pub glow: Vec4,
    /// Share of the full pressure of the air in x.
    pub load: Vec4,
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

    /// The sheath stays out of the depth prepass.
    fn enable_prepass() -> bool {
        false
    }

    /// The sheath casts no shadow.
    fn enable_shadows() -> bool {
        false
    }

    /// Draws both faces, so the sheath shows whichever side the camera is on.
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

/// Moves the sheath to the craft and shows it while the craft glows and the camera is outside it.
fn follow(
    heat: Res<Heat>,
    vantage: Res<Vantage>,
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
    let craft = heat.craft;
    let away = vantage.camera.distance(craft.centre) > craft.radius * OUTSIDE;
    let shown = heat.glow.heat > SHEATH_FROM && away;
    let pose = Transform::from_translation(craft.centre)
        .with_scale(Vec3::splat(craft.radius * SHEATH_REACH));
    for (mut transform, mut global, mut visibility, handle) in &mut sheaths {
        visibility.set_if_neq(match shown {
            true => Visibility::Visible,
            false => Visibility::Hidden,
        });
        if !shown {
            continue;
        }
        crate::render::pin(pose, &mut transform, &mut global);
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.sheath = SheathUniform {
                centre: craft.centre.extend(craft.radius),
                axis: craft.heading.extend(0.0),
                glow: heat.glow.color.extend(heat.glow.heat),
                load: Vec4::new(heat.glow.pressure, 0.0, 0.0, 0.0),
            };
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
            follow.run_if(resource_changed::<Heat>).after(HeatSet),
        );
}
