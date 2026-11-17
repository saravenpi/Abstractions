//! Pipeline warm-up: while the boot screen covers the loaded world, every effect is drawn once,
//! tiny, in front of the camera, so that no shader compiles the first time the player flies.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::flare::Flare;
use super::haze::Haze;
use super::motes::Mote;
use super::ribbon::{Look, Strip, fill, write};
use super::streaks::Streak;
use super::trail::Trail;
use super::trails::Ribbon;
use crate::render::GameCamera;

/// Size of a warmed-up effect in metres.
const SPECK: f32 = 0.002;

/// Distance in front of the camera at which the effects are warmed up, in metres.
const AHEAD: f32 = 2.0;

/// Every pooled effect entity that is not a ribbon.
type Pooled = Or<(With<Streak>, With<Mote>, With<Flare>, With<Haze>)>;

/// Returns a speck of ribbon just in front of a camera at `eye` looking along `forward`.
fn speck(eye: Vec3, forward: Vec3) -> Strip {
    let mut trail = Trail::default();
    let start = (eye + forward * AHEAD).as_dvec3();
    trail.feed(start, 1.0, 0.0);
    trail.feed(start + DVec3::splat(f64::from(SPECK)), 1.0, 0.0);
    let look = Look {
        eye,
        width: SPECK,
        spread: 1.0,
        life: 1.0,
        tint: Vec3::ONE,
    };
    let mut strip = Strip::default();
    fill(&trail, &look, |point| point.as_vec3(), &mut strip);
    strip
}

/// Shows every effect as a speck in front of the camera.
pub fn show(
    camera: Single<&GlobalTransform, With<GameCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    ribbons: Query<&Mesh3d, With<Ribbon>>,
    mut pooled: Query<(&mut Transform, Option<&mut Visibility>, Has<ChildOf>), Pooled>,
) {
    let view = camera.compute_transform();
    let strip = speck(view.translation, view.forward().as_vec3());
    for handle in &ribbons {
        if let Some(mut mesh) = meshes.get_mut(&handle.0) {
            write(&mut mesh, &strip);
        }
    }
    for (mut transform, visibility, child) in &mut pooled {
        transform.scale = Vec3::splat(SPECK);
        if !child {
            transform.translation = view.translation + view.forward() * AHEAD;
        }
        if let Some(mut visibility) = visibility {
            visibility.set_if_neq(Visibility::Inherited);
        }
    }
}

/// Clears every effect once the warm-up ends. The draw systems take over from here.
pub fn hide(
    mut meshes: ResMut<Assets<Mesh>>,
    ribbons: Query<&Mesh3d, With<Ribbon>>,
    mut pooled: Query<&mut Transform, Pooled>,
) {
    for handle in &ribbons {
        if let Some(mut mesh) = meshes.get_mut(&handle.0) {
            write(&mut mesh, &Strip::default());
        }
    }
    for mut transform in &mut pooled {
        transform.scale = Vec3::ZERO;
    }
}
