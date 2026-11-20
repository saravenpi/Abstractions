//! The rush of air: a soft pale ring that closes in from the corners of the view at very high speed.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::FlightFx;
use super::kit::Kit;
use crate::render::GameCamera;
use crate::view::ViewPrefs;

/// Distance of the ring in front of the camera, in metres.
const DISTANCE: f32 = 0.4;

/// Size of the clear middle of the ring as a share of the view half height: out of sight
/// beyond the corners with no rush, inside the edges at full rush.
const OPENING: (f32, f32) = (1.5, 0.85);

/// The rush ring, a child of the game camera.
#[derive(Component, Debug, Clone, Copy)]
pub struct Haze;

/// Gives every new game camera its rush ring.
pub fn fit(
    mut commands: Commands,
    kit: Option<Res<Kit>>,
    cameras: Query<Entity, Added<GameCamera>>,
) {
    let Some(kit) = kit else {
        return;
    };
    for camera in &cameras {
        commands.entity(camera).with_child((
            Haze,
            Mesh3d(kit.ring.clone()),
            MeshMaterial3d(kit.glow.clone()),
            Transform::from_xyz(0.0, 0.0, -DISTANCE),
            Visibility::Hidden,
            NoFrustumCulling,
            NotShadowCaster,
        ));
    }
}

/// Returns the scale of the ring for a view of vertical field `fov` and `aspect`, at `rush`.
///
/// The ring is an ellipse that matches the view, so it closes in evenly from every corner.
pub fn scale(fov: f32, aspect: f32, rush: f32) -> Vec3 {
    let half = (fov * 0.5).tan() * DISTANCE;
    let opening = OPENING.0 + (OPENING.1 - OPENING.0) * rush.clamp(0.0, 1.0);
    Vec3::new(half * aspect * opening, half * opening, 1.0)
}

/// Sizes the ring to the view and the rush, which the "Speed effects" setting scales. Does nothing
/// while neither changes.
pub fn draw(
    fx: Res<FlightFx>,
    prefs: Res<ViewPrefs>,
    mut rings: Query<(&mut Transform, &mut Visibility, &ChildOf), With<Haze>>,
    cameras: Query<&Projection, With<GameCamera>>,
) {
    if !fx.is_changed() && !prefs.is_changed() {
        return;
    }
    let rush = fx.rush * prefs.speed_fx.clamp(0.0, 1.0);
    for (mut transform, mut visibility, parent) in &mut rings {
        let Ok(Projection::Perspective(lens)) = cameras.get(parent.parent()) else {
            continue;
        };
        transform.scale = scale(lens.fov, lens.aspect_ratio, rush);
        let wanted = if rush > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        visibility.set_if_neq(wanted);
    }
}
