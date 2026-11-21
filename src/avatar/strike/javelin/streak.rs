//! Streak: the thin trail a javelin leaves behind its tail while it flies, fading within half a
//! second. It is the ribbon of a swing laid in the world: one row a frame across the tail, turned to
//! face the camera, on its own mesh that moves with the world when the render origin moves.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::super::canvas::{material, mesh, write};
use super::super::flight::{Rest, Spear};
use super::super::trail::Trail;
use super::tune::{STREAK_EVERY, STREAK_HALF, STREAK_LIFE, STREAK_PEAK};
use crate::render::GameCamera;
use crate::screens::Screen;
use crate::space::Rebased;

/// The trail of one javelin, on its own entity: the rows are kept in the frame of that entity.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Streak {
    /// The rows across the tail.
    pub trail: Trail,
}

/// Marks a javelin that has a trail, and names it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Streaked(pub Entity);

/// Run condition: a javelin flies with no trail yet.
pub fn bare(spears: Query<&Spear, Without<Streaked>>) -> bool {
    spears.iter().any(|spear| spear.rest == Rest::Flying)
}

/// Gives every flying javelin that has no trail one, where it is.
pub fn rig(
    mut commands: Commands,
    (mut meshes, mut materials): (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
    spears: Query<(Entity, &Spear, &Transform), Without<Streaked>>,
    mut shared: Local<Option<Handle<StandardMaterial>>>,
) {
    let look = shared
        .get_or_insert_with(|| materials.add(material()))
        .clone();
    for (entity, _, place) in spears.iter().filter(|(_, s, _)| s.rest == Rest::Flying) {
        let streak = commands
            .spawn((
                Name::new("Javelin trail"),
                Streak {
                    trail: Trail::lasting(STREAK_LIFE, STREAK_EVERY),
                },
                Mesh3d(meshes.add(mesh())),
                MeshMaterial3d(look.clone()),
                Transform::from_translation(place.translation),
                Visibility::Hidden,
                NoFrustumCulling,
                NotShadowCaster,
                (Rebased, DespawnOnEnter(Screen::Title)),
            ))
            .id();
        commands.entity(entity).insert(Streaked(streak));
    }
}

/// Returns the row a javelin with its tail at `tail` flying along unit `way` lays, seen from `eye`:
/// a short line across the tail, square to the path and to the line of sight.
pub fn across(tail: Vec3, way: Vec3, eye: Vec3) -> [Vec3; 2] {
    let side = way
        .cross(eye - tail)
        .normalize_or(way.any_orthonormal_vector());
    [tail - side * STREAK_HALF, tail + side * STREAK_HALF]
}

/// Returns the row `spear` placed at `place` lays this frame in the frame of its trail placed at
/// `frame`, seen from `eye`, and the speed it lays it at: nothing once it stopped flying.
fn laid(spear: &Spear, place: &Transform, frame: &Transform, eye: Vec3) -> ([Vec3; 2], f32) {
    let length = spear.item.build.grip + spear.item.build.reach();
    let tail = place.translation - place.rotation * Vec3::Y * length;
    let way = spear.velocity.normalize_or(place.rotation * Vec3::Y);
    let local = frame.compute_affine().inverse();
    let row = across(tail, way, eye).map(|end| local.transform_point3(end));
    let flying = spear.rest == Rest::Flying;
    (row, if flying { spear.velocity.length() } else { 0.0 })
}

/// The trails and the place of each.
type Trails<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Streak,
        &'static Transform,
        &'static Mesh3d,
        &'static mut Visibility,
    ),
>;

/// Lays the trail of every javelin from its tail while it flies, lets it fade once it landed, and
/// takes the trail away once it has faded.
pub fn trace(
    mut commands: Commands,
    (time, mut meshes): (Res<Time>, ResMut<Assets<Mesh>>),
    camera: Query<&GlobalTransform, With<GameCamera>>,
    spears: Query<(Entity, &Spear, &Transform, &Streaked), Without<Streak>>,
    mut trails: Trails,
) {
    let dt = time.delta_secs().max(1e-4);
    let eye = camera.single().map_or(Vec3::ZERO, |eye| eye.translation());
    for (entity, spear, place, streaked) in &spears {
        let Ok((mut streak, frame, handle, mut shown)) = trails.get_mut(streaked.0) else {
            continue;
        };
        let (row, speed) = laid(spear, place, frame, eye);
        streak.trail.feed(row, speed, dt);
        if streak.trail.is_empty() && spear.rest != Rest::Flying {
            commands.entity(streaked.0).despawn();
            commands.entity(entity).remove::<Streaked>();
            continue;
        }
        shown.set_if_neq(match streak.trail.is_empty() {
            true => Visibility::Hidden,
            false => Visibility::Inherited,
        });
        if let Some(mut mesh) = meshes.get_mut(&handle.0) {
            write(&mut mesh, &streak.trail, [STREAK_PEAK, 1.0]);
        }
    }
}
