//! Looking at the ship itself: a camera held at points given in hull axes, shots around the
//! landed ship, in the air, and of its cockpit seen from outside.

use super::*;

/// A camera held at a point of the hull and aimed at another, both in hull axes (the nose along
/// -Z, the roof along +Y), or nothing when the game camera is free.
#[derive(Resource, Default)]
pub struct Eye(pub Option<(Vec3, Vec3)>);

/// Puts the game camera at the point of the hull [`Eye`] gives, after the transforms are
/// propagated.
pub fn hold(
    eye: Res<Eye>,
    ships: Query<&GlobalTransform, (With<Ship>, Without<abstractions::render::GameCamera>)>,
    mut cameras: Query<&mut GlobalTransform, With<abstractions::render::GameCamera>>,
) {
    let (Some((at, target)), Ok(hull)) = (eye.0, ships.single()) else {
        return;
    };
    let up = match (target - at).normalize_or_zero().y.abs() > 0.95 {
        true => Vec3::NEG_Z,
        false => Vec3::Y,
    };
    let local = Transform::from_translation(at).looking_at(target, up);
    for mut camera in &mut cameras {
        *camera = hull.mul_transform(local);
    }
}

/// Views around the ship on the ground: name, camera and target in hull axes.
const ROUND: [(&str, Vec3, Vec3); 8] = [
    (
        "front",
        Vec3::new(1.5, 0.4, -10.0),
        Vec3::new(0.0, 0.0, 0.0),
    ),
    ("side", Vec3::new(-10.0, 0.6, 0.2), Vec3::new(0.0, 0.1, 0.2)),
    ("rear3q", Vec3::new(6.0, 2.2, 9.0), Vec3::new(0.0, 0.2, 0.0)),
    (
        "front3q",
        Vec3::new(-5.0, 1.4, -7.0),
        Vec3::new(0.0, 0.2, -0.5),
    ),
    ("above", Vec3::new(0.0, 11.0, 1.5), Vec3::new(0.0, 0.0, 0.0)),
    (
        "canopy",
        Vec3::new(-3.0, 1.9, -3.4),
        Vec3::new(0.0, 0.5, -1.4),
    ),
    (
        "engines",
        Vec3::new(0.0, 0.9, 8.0),
        Vec3::new(0.0, 0.2, 3.5),
    ),
    (
        "far",
        Vec3::new(-62.0, 14.0, 80.0),
        Vec3::new(0.0, 0.0, 0.0),
    ),
];

/// Frames between two views, and the frame of a view at which its shot is taken.
const DWELL: u32 = 14;
const DWELL_SHOT: u32 = 10;

/// Shots of the landed ship from the front, the side, the rear three-quarter, the front
/// three-quarter, above, the canopy, the engines and 100 m away. The robot stands at the left
/// wing root for the proportions.
pub fn round(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let Some(hull) = ship(world) else {
        return Act::Fail("no ship".into());
    };
    if frame == 1 {
        let beside = hull.pos + hull.rot * DVec3::new(-2.2, -0.9, -2.3);
        stand(world, beside, hull.pos, 0.0);
        view(world, ViewMode::Third, ViewMode::Third);
    }
    let (step, phase) = ((frame / DWELL) as usize, frame % DWELL);
    let Some((name, at, target)) = ROUND.get(step).copied() else {
        world.resource_mut::<Eye>().0 = None;
        view(world, ViewMode::First, ViewMode::First);
        return Act::Next;
    };
    match phase {
        1 => world.resource_mut::<Eye>().0 = Some((at, target)),
        DWELL_SHOT => shot(world, &format!("30-round-{name}")),
        _ => {}
    }
    Act::Wait
}
