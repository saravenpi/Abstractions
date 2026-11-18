//! The cloud layer: a shell of noise around the planet, and how deep the camera is inside it.

use bevy::asset::embedded_asset;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use super::DayClock;
use super::celestial::smoothstep;
use super::dome::cloud_cover;
use super::palette::Palette;
use super::sky::linear;
use super::vantage::{LocalSystem, Vantage};
use super::weather::LocalWeather;
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};

/// Width in metres of one cell of the cloud noise.
const CELL: f32 = 450.0;

/// Distance in planet radii past which the shell is no longer drawn.
const HIDE_RADII: f32 = 20.0;

/// How deep the camera is in the cloud layer: 0 outside it, 1 in the middle of the band.
///
/// The ship and the audio read it for shake and sound. It stays 0 while clouds are switched off
/// and on a planet without clouds.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct CloudCrossing {
    /// Depth in the band, from 0 to 1.
    pub depth: f32,
}

/// Returns how deep `altitude` is in the band from `base` to `top`: 0 below the base and above
/// the top, 1 in the middle, smooth in between.
pub fn crossing_depth(altitude: f32, base: f32, top: f32) -> f32 {
    if top <= base {
        return 0.0;
    }
    let along = (altitude - base) / (top - base);
    smoothstep(0.0, 1.0, 1.0 - (2.0 * along - 1.0).abs())
}

/// Uniform block read by `clouds.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct CloudUniform {
    /// Unit vector toward the star in xyz and the share of air above the camera in w.
    pub light: Vec4,
    /// Linear cloud color in xyz and the cover from 0 to 1 in w.
    pub color: Vec4,
    /// Linear horizon color in xyz and the whiteout from 0 to 1 in w.
    pub haze: Vec4,
    /// Time in seconds, noise cells per shell radius and the distance in metres to the far rim of the shell.
    pub params: Vec4,
    /// Linear colour of the sunlit clouds seen from space.
    pub space: Vec4,
    /// First column of the rotation from render axes to planet-fixed axes.
    pub fixed_x: Vec4,
    /// Second column of that rotation.
    pub fixed_y: Vec4,
    /// Third column of that rotation.
    pub fixed_z: Vec4,
}

/// Material of the cloud shell.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct CloudMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub clouds: CloudUniform,
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

/// Marks the cloud shell.
#[derive(Component, Debug, Clone, Copy)]
pub struct CloudShell;

/// Spawns the hidden shell when the world starts loading.
fn spawn_shell(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CloudMaterial>>,
) {
    commands.spawn((
        CloudShell,
        Mesh3d(meshes.add(super::shell(false))),
        MeshMaterial3d(materials.add(CloudMaterial::default())),
        Transform::default(),
        Visibility::Hidden,
        NotShadowCaster,
        NoFrustumCulling,
        DespawnOnEnter(Screen::Title),
    ));
}

/// Everything the cloud layer is computed from.
#[derive(SystemParam)]
pub(super) struct CloudInputs<'w> {
    vantage: Res<'w, Vantage>,
    local: Res<'w, LocalSystem>,
    palette: Res<'w, Palette>,
    clock: Res<'w, DayClock>,
    settings: Res<'w, Persistent<Settings>>,
    time: Res<'w, Time>,
    weather: Res<'w, LocalWeather>,
}

impl CloudInputs<'_> {
    /// Returns the shader values for a shell of `radius` metres and a camera `depth` deep in the band.
    ///
    /// The noise is read in planet-fixed axes, so the clouds turn with the ground below them,
    /// and the cover of the day is scaled by how cloudy the weather makes the planet.
    fn uniform(&self, radius: f32, depth: f32) -> CloudUniform {
        let vantage = &*self.vantage;
        let observer = &vantage.observer;
        let distance = vantage.radius + vantage.altitude;
        let ground = vantage.radius * vantage.radius;
        let rim =
            (distance * distance - ground).max(0.0).sqrt() + (radius * radius - ground).sqrt();
        let fixed = Mat3::from_quat((observer.to_render * observer.spin).inverse().as_quat());
        let air = self.local.air_below(vantage);
        let cover = self
            .weather
            .look(&air)
            .cover(&air, cloud_cover(self.clock.phase));
        CloudUniform {
            light: self.palette.sun_dir.extend(vantage.air),
            color: linear(self.palette.cloud).truncate().extend(cover),
            haze: linear(self.palette.horizon).truncate().extend(depth),
            params: Vec4::new(self.time.elapsed_secs(), radius / CELL, rim, 0.0),
            space: air.clouds_from_space().extend(0.0),
            fixed_x: fixed.x_axis.extend(0.0),
            fixed_y: fixed.y_axis.extend(0.0),
            fixed_z: fixed.z_axis.extend(0.0),
        }
    }
}

/// The transforms, visibility and material of the cloud shell.
type ShellParts = (
    &'static mut Transform,
    &'static mut GlobalTransform,
    &'static mut Visibility,
    &'static MeshMaterial3d<CloudMaterial>,
);

/// Measures how deep the camera is in the cloud layer, wraps the shell around the planet halfway
/// up the band, whose bottom rain lowers, and refreshes the shader values. A sky without clouds
/// has no shell and no crossing.
pub(super) fn drift(
    inputs: CloudInputs,
    mut crossing: ResMut<CloudCrossing>,
    mut shells: Query<ShellParts, With<CloudShell>>,
    mut materials: ResMut<Assets<CloudMaterial>>,
) {
    let vantage = &*inputs.vantage;
    let look = inputs.weather.look(&inputs.local.air_below(vantage));
    let base = look.base(vantage.cloud_base);
    let on = inputs.settings.graphics.clouds && look.cover > 0.0 && vantage.cloud_top > base;
    let depth = match on {
        true => crossing_depth(vantage.altitude, base, vantage.cloud_top),
        false => 0.0,
    };
    crossing.set_if_neq(CloudCrossing { depth });
    let radius = vantage.radius + 0.5 * (base + vantage.cloud_top);
    let shown = on && vantage.radius + vantage.altitude < HIDE_RADII * vantage.radius;
    let center = vantage
        .observer
        .place(vantage.observer.center, vantage.camera);
    let pose = Transform::from_translation(center).with_scale(Vec3::splat(radius));
    for (mut transform, mut global, mut visibility, handle) in &mut shells {
        super::pin(pose, &mut transform, &mut global);
        visibility.set_if_neq(match shown {
            true => Visibility::Visible,
            false => Visibility::Hidden,
        });
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.clouds = inputs.uniform(radius, depth);
        }
    }
}

/// Embeds the shader, registers the material and runs the layer once the camera is tracked.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "clouds.wesl");
    app.add_plugins(MaterialPlugin::<CloudMaterial>::default())
        .init_resource::<CloudCrossing>()
        .add_systems(OnEnter(Screen::Loading), spawn_shell)
        .add_systems(PostUpdate, drift.after(super::sun::track));
}
