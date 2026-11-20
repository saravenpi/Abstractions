//! The cloud layer: a shell around the planet that draws the cloud field of that planet, and
//! how deep the camera is inside a real cloud of it.

mod cross;
mod map;
mod maps;
mod material;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_cross;

use bevy::asset::embedded_asset;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;
use bevy::shader::{Shader, load_shader_library};

pub use cross::{CloudCrossing, crossing_depth};
pub(super) use map::toward_second;
pub(super) use maps::CloudMaps;
pub use material::{CloudMaterial, CloudUniform};

use super::palette::Palette;
use super::sky::linear;
use super::vantage::{LocalSystem, Vantage};
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::space::Clock;
use crate::universe::BodyId;
pub(super) use crate::weather::CELL;
use crate::weather::{Field, Weather, density_at, drift as drifted};

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
    /// Where the camera is.
    vantage: Res<'w, Vantage>,
    /// The star system around it, for the planet below.
    local: Res<'w, LocalSystem>,
    /// The colours of the sky and the direction of the star.
    palette: Res<'w, Palette>,
    /// Universe time, which drifts and morphs the clouds.
    clock: Res<'w, Clock>,
    /// The settings, which can switch the clouds off.
    settings: Res<'w, Persistent<Settings>>,
    /// Frame time, which the cloud around the camera eases over.
    time: Res<'w, Time>,
    /// The weather at the camera, whose cloud is the broad cover of the field there.
    weather: Option<Res<'w, Weather>>,
    /// The cover maps, which the shell asks for the map of the planet below.
    maps: ResMut<'w, CloudMaps>,
}

impl CloudInputs<'_> {
    /// Returns the planet or moon under the camera when its sky has a deck of clouds and
    /// clouds are switched on.
    fn cloudy(&self) -> Option<BodyId> {
        let vantage = &*self.vantage;
        let near = self.local.0.as_ref()?;
        let body = near.body(u8::try_from(vantage.body).ok()?, vantage.moon)?;
        let on = self.settings.graphics.clouds && Field::of(body, false).deck;
        on.then_some(body.id)
    }

    /// Returns the rotation from render axes to the axes fixed to the planet under the camera,
    /// which the cloud field is laid in: the clouds turn with the ground below them.
    fn fixed(&self) -> Mat3 {
        let observer = &self.vantage.observer;
        Mat3::from_quat((observer.to_render * observer.spin).inverse().as_quat())
    }

    /// Returns how much cloud the field holds where the camera is, on a shell of `radius`
    /// metres: the puff the shader draws there, under the broad cover the weather reads.
    fn density(&self, radius: f32) -> f32 {
        let cover = self.weather.as_deref().map_or(0.0, |weather| weather.cloud);
        let dir = (self.fixed() * self.vantage.up).as_dvec3();
        density_at(dir, radius / CELL, self.clock.seconds, cover)
    }

    /// Returns the shader values for a shell of `radius` metres and a camera `depth` deep in cloud.
    fn uniform(&self, radius: f32, depth: f32) -> CloudUniform {
        let vantage = &*self.vantage;
        let distance = vantage.radius + vantage.altitude;
        let ground = vantage.radius * vantage.radius;
        let rim =
            (distance * distance - ground).max(0.0).sqrt() + (radius * radius - ground).sqrt();
        let (fixed, seconds) = (self.fixed(), self.clock.seconds);
        let air = self.local.air_below(vantage);
        CloudUniform {
            light: self.palette.sun_dir.extend(vantage.air),
            color: linear(self.palette.cloud)
                .truncate()
                .extend(toward_second(seconds)),
            haze: linear(self.palette.horizon).truncate().extend(depth),
            params: Vec4::new(radius / CELL, rim, 0.0, 0.0),
            space: air.clouds_from_space().extend(0.0),
            drift: drifted(seconds).extend(0.0),
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

/// Measures how deep the camera is in real cloud, wraps the shell around the planet halfway up
/// the cloud band, at a radius that never changes, and refreshes the shader values. The shell
/// shows once the cover map of the planet is ready. A sky without clouds has no shell and no
/// crossing, and past the hand-over the far body draws the same clouds from the same map.
pub(super) fn drift(
    mut inputs: CloudInputs,
    mut around: Local<f32>,
    mut crossing: ResMut<CloudCrossing>,
    mut shells: Query<ShellParts, With<CloudShell>>,
    mut materials: ResMut<Assets<CloudMaterial>>,
) {
    let vantage = *inputs.vantage;
    let body = inputs.cloudy();
    let radius = vantage.radius + 0.5 * (vantage.cloud_base + vantage.cloud_top);
    let band = match body {
        Some(_) => crossing_depth(vantage.altitude, vantage.cloud_base, vantage.cloud_top),
        None => 0.0,
    };
    let seconds = inputs.time.delta_secs();
    *around = cross::eased(*around, inputs.density(radius), band, seconds);
    let depth = band * *around;
    crossing.set_if_neq(CloudCrossing { depth });
    let map = body.and_then(|id| inputs.maps.want(id));
    let limit = inputs.local.0.as_ref().map(|near| near.handover(&vantage));
    let within = limit.is_none_or(|limit| vantage.observer.altitude(0.0) < limit);
    let shown = map.is_some() && within;
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
            material.cover.clone_from(&map);
        }
    }
}

/// Embeds the shaders, registers the material and the cover maps, and runs the layer once the
/// camera is tracked. The module both cloud shaders import is loaded wherever shaders are: a
/// world without a renderer, as in the tests, has none to load.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "clouds.wesl");
    if app.world().contains_resource::<Assets<Shader>>() {
        load_shader_library!(app, "cloud_field.wesl");
    }
    app.add_plugins(MaterialPlugin::<CloudMaterial>::default())
        .init_resource::<CloudCrossing>()
        .init_resource::<CloudMaps>()
        .add_systems(OnEnter(Screen::Loading), spawn_shell)
        .add_systems(OnEnter(Screen::Title), maps::clear)
        .add_systems(
            PostUpdate,
            (drift, maps::keep).chain().after(super::sun::track),
        );
}
