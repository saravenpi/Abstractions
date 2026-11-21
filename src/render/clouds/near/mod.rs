//! Flying through a cloud: the puffs of the cloud field around the camera drawn as soft, lit
//! discs at their places in the band, so they stream past the ship with its speed and their
//! parallax, thicken as the camera goes deeper, and tear open on the way out with the sky and
//! the ground showing through the gaps. The whiteout of the fog follows the same volume at the
//! camera, see `CloudCrossing`.
//!
//! Cost: about 500 samples of the field and one mesh of 256 discs of 6 triangles rewritten per
//! frame, one draw, only while the camera is within [`REACH`] metres of the cloud band.

mod discs;
mod fill;

#[cfg(test)]
mod tests;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::prelude::*;

use discs::{Discs, mesh};
use fill::{Layer, Puff, REACH, lattice};

use super::CloudCover;
use crate::render::GameCamera;
use crate::render::exposure::{Metered, lifted};
use crate::render::palette::Palette;
use crate::render::sky::linear;
use crate::render::vantage::Vantage;
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::space::Clock;
use crate::weather::{CELL, Weather};

/// Marks the mesh of the near cloud.
#[derive(Component, Debug, Clone, Copy)]
pub struct NearCloud;

/// Spawns the hidden near cloud when the world starts loading.
fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    };
    commands.spawn((
        NearCloud,
        Mesh3d(meshes.add(mesh())),
        MeshMaterial3d(materials.add(material)),
        Transform::default(),
        Visibility::Hidden,
        NotShadowCaster,
        NoFrustumCulling,
        DespawnOnEnter(Screen::Title),
    ));
}

/// Returns the cloud layer under the camera, or nothing when clouds are off, the sky has no
/// band, or the camera is farther than [`REACH`] from it.
fn layer(vantage: &Vantage, cover: f32, seconds: f64) -> Option<Layer> {
    let (base, top) = (vantage.cloud_base, vantage.cloud_top);
    let reach = REACH as f32;
    let near = vantage.altitude > base - reach && vantage.altitude < top + reach;
    (top > base && near && cover > 0.0).then(|| Layer {
        band: (base, top),
        ground: f64::from(vantage.radius),
        cells: (vantage.radius + 0.5 * (base + top)) / CELL,
        seconds,
        cover,
    })
}

/// What the near cloud is drawn from.
type Sky<'w> = (
    Res<'w, Vantage>,
    Res<'w, Palette>,
    Res<'w, Metered>,
    Res<'w, Clock>,
);

/// The cover of the sky: the one the probe forces, else the weather at the camera, and the
/// settings that can switch the clouds off.
type Cover<'w> = (
    Res<'w, CloudCover>,
    Option<Res<'w, Weather>>,
    Res<'w, Persistent<Settings>>,
);

/// The near cloud entity: its pose, whether it shows and its mesh.
type Near = (
    &'static mut Transform,
    &'static mut GlobalTransform,
    &'static mut Visibility,
    &'static Mesh3d,
);

/// Lays the puffs of the field around the camera into the mesh of the near cloud and shows it,
/// or hides it far from the band.
fn draw(
    (vantage, palette, metered, clock): Sky,
    (forced, weather, settings): Cover,
    camera: Single<&GlobalTransform, (With<GameCamera>, Without<NearCloud>)>,
    mut near: Single<Near, With<NearCloud>>,
    (mut meshes, mut puffs, mut discs): (ResMut<Assets<Mesh>>, Local<Vec<Puff>>, Local<Discs>),
) {
    let cover = forced
        .0
        .unwrap_or_else(|| weather.as_deref().map_or(0.0, |w| w.cloud));
    let found = layer(&vantage, cover, clock.seconds).filter(|_| settings.graphics.clouds);
    let (transform, global, visibility, handle) = &mut *near;
    let Some(layer) = found else {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    };
    let observer = &vantage.observer;
    let eye = observer.spin.inverse() * (observer.eye - observer.center);
    lattice(eye, &layer, &mut puffs);
    let cloud = linear(lifted(palette.cloud, metered.gain())).truncate();
    let view = (camera.right().as_vec3(), camera.up().as_vec3());
    let render = |at: DVec3| {
        (observer.to_render * (observer.spin * at - (observer.eye - observer.center))).as_vec3()
    };
    let place: &dyn Fn(DVec3) -> Vec3 = &render;
    discs.lay((puffs.as_slice(), place), view, cloud);
    if let Some(mut mesh) = meshes.get_mut(&handle.0) {
        discs.write(&mut mesh);
    }
    crate::render::pin(
        Transform::from_translation(vantage.camera),
        transform,
        global,
    );
    visibility.set_if_neq(Visibility::Visible);
}

/// Registers the near cloud, laid after the cloud layer is known for the frame.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), spawn)
        .add_systems(
            PostUpdate,
            draw.after(super::drift).run_if(resource_exists::<Vantage>),
        );
}
