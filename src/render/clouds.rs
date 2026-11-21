//! The cloud layer: a few shells around the planet, stacked from the base of the cloud band to
//! its top, that draw the cloud field of that planet with its depth, and how deep the camera is
//! inside a real cloud of it. The slices, the whiteout and the fly-through puffs read one
//! volume, `weather::volume`.
//!
//! Cost: one draw of the shell mesh per slice. The base slice reads four octaves of the noise
//! and two more toward the star; the upper ones read three and no relief, so the layer costs
//! twelve noise lookups per pixel at most where one shell cost six.

mod cross;
mod inputs;
mod map;
mod maps;
mod material;
mod mesh;
mod near;
mod shell;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_cross;
#[cfg(test)]
mod tests_rebase;
#[cfg(test)]
mod tests_shell;
#[cfg(test)]
mod tests_stage;
#[cfg(test)]
mod tests_tolerance;
mod tune;

use bevy::asset::embedded_asset;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;
use bevy::shader::{Shader, load_shader_library};

pub use cross::{CloudCrossing, depth_at};
pub(super) use map::toward_second;
pub(super) use maps::CloudMaps;
pub use material::{CloudMaterial, CloudUniform};

use super::palette::Palette;
use super::vantage::{LocalSystem, Vantage};
use super::weather::LocalWeather;
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::space::Clock;
pub(super) use crate::weather::CELL;
use crate::weather::{SLICES, Weather, slice};

/// Marks the cloud shell.
#[derive(Component, Debug, Clone, Copy)]
pub struct CloudShell;

/// The slice of the cloud band a shell draws, counted from the base.
#[derive(Component, Debug, Clone, Copy)]
pub struct CloudSlice(pub u32);

/// A cover the cloud shell is drawn with in place of the broad cover of the weather, from 0 to 1,
/// for the probe to look at a clear, a broken and an overcast sky. Nothing sets it in the game.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct CloudCover(pub Option<f32>);

/// Spawns the hidden shells of the slices when the world starts loading: one mesh, scaled to the
/// radius of each slice, and a material each.
fn spawn_shell(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CloudMaterial>>,
) {
    let mesh = meshes.add(super::shell(false));
    for k in 0..SLICES {
        commands.spawn((
            (CloudShell, CloudSlice(k)),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(materials.add(CloudMaterial::default())),
            Transform::default(),
            Visibility::Hidden,
            NotShadowCaster,
            NoFrustumCulling,
            DespawnOnEnter(Screen::Title),
        ));
    }
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
    /// The cover the probe forces, when it does.
    forced: Res<'w, CloudCover>,
    /// The weather at the camera, whose rain and dust fade the far clouds sooner.
    sky: Option<Res<'w, LocalWeather>>,
    /// The cover maps, which the shell asks for the map of the planet below.
    maps: ResMut<'w, CloudMaps>,
}

/// The slice, transforms, visibility and material of a cloud shell.
type ShellParts = (
    &'static CloudSlice,
    &'static Mesh3d,
    &'static mut Transform,
    &'static mut GlobalTransform,
    &'static mut Visibility,
    &'static MeshMaterial3d<CloudMaterial>,
);

/// Returns how deep the camera of `inputs` is in real cloud, eased from `around` over the frame:
/// the volume of the slices at its height over the cloud the field holds under it.
fn crossing(inputs: &CloudInputs, around: &mut f32, mid: f32, cloudy: bool) -> f32 {
    let vantage = &*inputs.vantage;
    let band = (vantage.cloud_base, vantage.cloud_top);
    let column = |body: f32| depth_at(vantage.altitude, band, body);
    let (inside, here) = match cloudy {
        true => (column(1.0), column(inputs.density(mid))),
        false => (0.0, 0.0),
    };
    *around = cross::eased(*around, here, inside, inputs.time.delta_secs());
    *around
}

/// Measures how deep the camera is in real cloud, wraps the slices around the planet from the
/// base of the cloud band to its top, at radii that never change, and refreshes their shader
/// values. The one mesh of the shells is a cube sphere within [`shell::TOLERANCE`] metres of the
/// true sphere near it, and within a hundredth of the distance to it from afar, built over what
/// the camera can see of the highest slice off the main thread. The shells show once the cover
/// map of the planet is ready and their mesh is built. A sky without clouds has no shell and no
/// crossing, and past the hand-over the far body draws the same clouds from the same map.
pub(super) fn drift(
    mut inputs: CloudInputs,
    mut around: Local<f32>,
    mut crossing_now: ResMut<CloudCrossing>,
    mut shells: Query<ShellParts, With<CloudShell>>,
    mut assets: mesh::Shell,
) {
    let vantage = *inputs.vantage;
    let body = inputs.cloudy();
    let (base, top) = (vantage.cloud_base, vantage.cloud_top);
    let at = |share: f32| vantage.radius + base + share * (top - base);
    let mid = at(0.5);
    let depth = crossing(&inputs, &mut around, mid, body.is_some());
    crossing_now.set_if_neq(CloudCrossing { depth });
    let map = body.and_then(|id| inputs.maps.want(id));
    let wanted = map.is_some() && inputs.within(&vantage);
    let center = vantage
        .observer
        .place(vantage.observer.center, vantage.camera);
    let mut ready = None;
    for (k, mesh, mut transform, mut global, mut visibility, handle) in &mut shells {
        let sight = (vantage.radius, vantage.altitude, at(1.0));
        let shown =
            *ready.get_or_insert_with(|| wanted && assets.remesh(&mesh.0, sight, vantage.up));
        let radius = at(slice(k.0));
        let pose = Transform::from_translation(center).with_scale(Vec3::splat(radius));
        super::pin(pose, &mut transform, &mut global);
        visibility.set_if_neq([Visibility::Hidden, Visibility::Visible][usize::from(shown)]);
        if shown {
            assets.paint(&handle.0, inputs.uniform(mid, (k.0, radius), depth), &map);
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
    near::plugin(app);
    app.add_plugins(MaterialPlugin::<CloudMaterial>::default())
        .init_resource::<CloudCrossing>()
        .init_resource::<CloudMaps>()
        .init_resource::<CloudCover>()
        .add_systems(OnEnter(Screen::Loading), spawn_shell)
        .add_systems(OnEnter(Screen::Title), maps::clear)
        .add_systems(
            PostUpdate,
            (drift, maps::keep).chain().after(super::sun::track),
        );
}
