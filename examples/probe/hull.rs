//! Looking at the ship itself: a camera held at points given in hull axes, and the turntable of
//! the landed ship, with its canopy open and the robot beside it, and shut with the robot in it.

use super::*;

/// A camera held at a point of the hull and aimed at another, both in hull axes (the nose along
/// -Z, the roof along +Y), or nothing when the game camera is free.
#[derive(Resource, Default)]
pub struct Eye(pub Option<(Vec3, Vec3)>);

/// One view of the ship: its name, where the camera is and what it looks at, in hull axes.
pub type View = (&'static str, Vec3, Vec3);

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

/// Views around the ship on the ground, near, and one from 100 m.
const ROUND: [View; 10] = [
    (
        "side",
        Vec3::new(-5.6, 0.3, -0.2),
        Vec3::new(0.0, 0.2, -0.2),
    ),
    ("front", Vec3::new(0.5, 0.2, -8.4), Vec3::new(0.0, 0.1, 0.0)),
    (
        "front3q",
        Vec3::new(-4.4, 1.2, -6.4),
        Vec3::new(0.0, 0.2, -0.6),
    ),
    ("rear3q", Vec3::new(4.8, 1.9, 7.4), Vec3::new(0.0, 0.3, 0.6)),
    ("above", Vec3::new(0.0, 8.5, 0.4), Vec3::new(0.0, 0.0, 0.0)),
    (
        "below",
        Vec3::new(-3.6, -0.95, -5.2),
        Vec3::new(0.0, -0.2, 0.0),
    ),
    (
        "engines",
        Vec3::new(0.3, 0.7, 7.0),
        Vec3::new(0.0, 0.3, 3.4),
    ),
    (
        "canopy",
        Vec3::new(-2.4, 1.5, -3.3),
        Vec3::new(0.0, 0.6, -1.5),
    ),
    ("8m", Vec3::new(-7.5, 2.4, -8.5), Vec3::new(0.0, 0.0, 0.0)),
    (
        "far",
        Vec3::new(-62.0, 14.0, 80.0),
        Vec3::new(0.0, 0.0, 0.0),
    ),
];

/// Views of the ship with its canopy shut and the robot in the seat.
const SHUT: [View; 4] = [
    (
        "side",
        Vec3::new(-5.6, 0.3, -0.2),
        Vec3::new(0.0, 0.2, -0.2),
    ),
    (
        "front3q",
        Vec3::new(-4.4, 1.2, -6.4),
        Vec3::new(0.0, 0.2, -0.6),
    ),
    (
        "canopy",
        Vec3::new(-2.4, 1.5, -3.3),
        Vec3::new(0.0, 0.6, -1.5),
    ),
    ("rear3q", Vec3::new(4.8, 1.9, 7.4), Vec3::new(0.0, 0.3, 0.6)),
];

/// Frames between two views, and the frame of a view at which its shot is taken.
const DWELL: (u32, u32) = (14, 10);

/// Holds the camera at each of `list` in turn from `frame` 1 on and saves a shot of each, named
/// `prefix` and the name of the view. Returns true once all are taken, with the camera free again.
pub fn views(world: &mut World, frame: u32, prefix: &str, list: &[View]) -> bool {
    let (step, phase) = ((frame / DWELL.0) as usize, frame % DWELL.0);
    let Some((name, at, target)) = list.get(step).copied() else {
        world.resource_mut::<Eye>().0 = None;
        return true;
    };
    if phase == 1 {
        world.resource_mut::<Eye>().0 = Some((at, target));
    }
    if phase == DWELL.1 {
        shot(world, &format!("{prefix}-{name}"));
    }
    false
}

/// Shots round the landed ship. The robot stands at the left of the cockpit for the
/// proportions, so the canopy is open.
pub fn round(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let Some(hull) = ship(world) else {
        return Act::Fail("no ship".into());
    };
    if frame == 1 {
        let beside = hull.pos + hull.rot * DVec3::new(-2.3, -1.2, -2.4);
        stand(world, beside, hull.pos, 0.0);
        view(world, ViewMode::Third, ViewMode::Third);
    }
    if frame < 60 {
        return Act::Wait;
    }
    if views(world, frame - 59, "30-round", &ROUND) {
        view(world, ViewMode::First, ViewMode::First);
        return Act::Next;
    }
    Act::Wait
}

/// Shots round the landed ship with the robot in the seat and the canopy shut.
pub fn shut(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        view(world, ViewMode::First, ViewMode::Third);
    }
    if frame < 50 {
        return Act::Wait;
    }
    if views(world, frame - 49, "31-shut", &SHUT) {
        view(world, ViewMode::First, ViewMode::First);
        return Act::Next;
    }
    Act::Wait
}
