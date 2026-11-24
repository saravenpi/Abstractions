//! Pure collision math and sync helpers for creatures against solids, robots and each other.

use bevy::math::{DVec3, Vec2, Vec3, Vec3Swizzles};
use bevy::prelude::*;

use crate::fauna::beast::{Beast, Part};
use crate::fauna::bulk::capsule;
use crate::fauna::place::Spot;
use crate::life::body::{Body, Locomotion};
use crate::world::Terrain;

/// Horizontal reach around the local player within which creatures collide against static solids.
pub(super) const SOLID_NEAR: f64 = 96.0;

/// Extra clearance in metres added past a collider surface after depenetration.
pub(super) const PUSH_PAD: f32 = 0.02;

/// Half the standing height of the robot capsule in metres.
pub(super) const ROBOT_HALF: f32 = 0.5 * crate::player::BODY_LENGTH + crate::player::BODY_RADIUS;

/// Mutable query of the child body-part transforms shifted with a corrected beast.
pub(in crate::fauna) type Parts<'w, 's> = Query<
    'w,
    's,
    &'static mut Transform,
    (
        With<Part>,
        Without<crate::player::Player>,
        Without<crate::avatar::Avatar>,
    ),
>;

/// Snapshot of one living drawn creature for pairwise and player separation passes.
#[derive(Clone, Copy)]
pub(in crate::fauna) struct Actor {
    /// Entity of the living beast.
    pub(in crate::fauna) entity: Entity,
    /// Index of its species in the planet life list.
    pub(in crate::fauna) species: usize,
    /// Horizontal offset accumulated from pairwise and robot pushes in local XZ.
    pub(in crate::fauna) shift: Vec2,
    /// Centre position in planet axes.
    pub(in crate::fauna) pos: DVec3,
    /// Horizontal collision radius in metres.
    pub(in crate::fauna) rad: f32,
    /// Minimum radial distance from the planet centre in metres.
    pub(in crate::fauna) r_min: f64,
    /// Maximum radial distance from the planet centre in metres.
    pub(in crate::fauna) r_max: f64,
    /// Body mass in kilograms, clamped to at least 1 kg.
    pub(in crate::fauna) mass: f32,
    /// Tangent frame the beast steps in.
    pub(in crate::fauna) spot: Spot,
}

/// Returns true when a body stands on the ground surface rather than flying or swimming.
pub(in crate::fauna) fn grounded(locomotion: Locomotion) -> bool {
    matches!(
        locomotion,
        Locomotion::Walk | Locomotion::Slither | Locomotion::Ooze | Locomotion::Tumble
    )
}

/// Returns the horizontal collision radius and vertical radial interval around `pos` for `body`.
pub(in crate::fauna) fn capsule_span(pos: DVec3, body: &Body) -> (f32, f64, f64) {
    let (radius, long) = capsule(body.length, body.height);
    let rad = radius + 0.5 * long;
    let r = pos.length();
    if grounded(body.locomotion) {
        (rad, r, r + f64::from((body.height + radius).max(0.5)))
    } else {
        let half = f64::from(radius);
        (rad, r - half, r + half)
    }
}

/// Removes any horizontal velocity component pointing opposite `outward`.
pub(in crate::fauna) fn strip_inward(vel: &mut Vec3, outward: Vec2) {
    let Some(dir) = outward.try_normalize() else {
        return;
    };
    let inward = vel.xz().dot(dir);
    if inward < 0.0 {
        vel.x -= dir.x * inward;
        vel.z -= dir.y * inward;
    }
}

/// Returns the horizontal depenetration shift in local XZ when a sphere of `radius` centred at
/// `probe` overlaps a collider whose nearest surface projection is `proj`.
pub(in crate::fauna) fn depenetrate_offset(
    probe: Vec3,
    proj: Vec3,
    is_inside: bool,
    radius: f32,
    fallback: Vec2,
) -> Vec2 {
    if is_inside {
        let flat = (proj - probe).xz();
        return match flat.try_normalize() {
            Some(dir) => dir * (flat.length() + radius + PUSH_PAD),
            None => fallback.normalize_or(Vec2::X) * (radius + 0.5),
        };
    }
    let outward = (probe - proj).xz();
    let dist = probe.distance(proj);
    if dist >= radius || outward.length() < 0.25 * dist {
        return Vec2::ZERO;
    }
    let dir = outward
        .try_normalize()
        .unwrap_or_else(|| fallback.normalize_or(Vec2::X));
    dir * ((radius - dist) + PUSH_PAD)
}

/// Returns the mass-weighted horizontal push vectors in each beast's local XZ plane when `a` and `b`
/// overlap both horizontally and radially, or `None` when separated.
pub(in crate::fauna) fn separate_pair(a: &Actor, b: &Actor) -> Option<(Vec2, Vec2)> {
    if a.r_min >= b.r_max || b.r_min >= a.r_max {
        return None;
    }
    let up = (a.pos + b.pos).normalize_or_zero();
    let diff = a.pos - b.pos;
    let flat = diff - up * diff.dot(up);
    let dist = flat.length();
    let reach = f64::from(a.rad + b.rad);
    if dist >= reach {
        return None;
    }
    let fallback = (a.spot.rot * DVec3::X).normalize_or(DVec3::X);
    let away = if dist > 1e-4 { flat / dist } else { fallback };
    let pen = reach - dist;
    let (inv_a, inv_b) = (1.0 / f64::from(a.mass), 1.0 / f64::from(b.mass));
    let share_a = inv_a / (inv_a + inv_b);
    let world_a = away * (pen * share_a);
    let world_b = -away * (pen * (1.0 - share_a));
    let push_a = (a.spot.rot.inverse() * world_a).as_vec3().xz();
    let push_b = (b.spot.rot.inverse() * world_b).as_vec3().xz();
    Some((push_a, push_b))
}

/// Returns the local XZ push away from a standing robot at planet position `robot_planet`, or
/// `None` when horizontally clear or vertically disjoint.
pub(in crate::fauna) fn push_from_robot(
    spot: &Spot,
    at: Vec3,
    body: &Body,
    robot_planet: DVec3,
) -> Option<Vec2> {
    let robot = spot.local(robot_planet);
    let (radius, long) = capsule(body.length, body.height);
    let (b_min, b_max) = if grounded(body.locomotion) {
        (at.y, at.y + (body.height + radius).max(0.5))
    } else {
        (at.y - radius, at.y + radius)
    };
    if robot.y - ROBOT_HALF >= b_max || b_min >= robot.y + ROBOT_HALF {
        return None;
    }
    let min_dist = radius + 0.5 * long + crate::player::BODY_RADIUS;
    let gap = (at - robot).xz();
    let dist = gap.length();
    if dist >= min_dist {
        return None;
    }
    let away = if dist > 1e-4 { gap / dist } else { Vec2::X };
    Some(away * (min_dist - dist))
}

/// Applies a net local XZ shift to `beast`, re-grounding grounded locomotion on `terrain` and
/// shifting its `parts` transforms in lockstep.
pub(super) fn apply_shift(
    beast: &mut Beast,
    body: &Body,
    terrain: &Terrain,
    shift_xz: Vec2,
    parts: &mut Parts,
) {
    if shift_xz.length_squared() <= 1e-8 {
        return;
    }
    let spot = beast.spot;
    let flat = Vec3::new(shift_xz.x, 0.0, shift_xz.y);
    beast.motion.root.at += flat;
    beast.motion.root.place += flat;
    for foot in &mut beast.motion.feet {
        foot.at += flat;
        foot.to += flat;
    }
    let dy = ground_sync(beast, body, terrain, &spot);
    let delta = Vec3::new(shift_xz.x, dy, shift_xz.y);
    for part in beast.parts.iter().flatten() {
        if let Ok(mut place) = parts.get_mut(*part) {
            place.translation += delta;
        }
    }
}

/// Re-grounds a corrected beast on `terrain` when its locomotion uses the surface and returns the
/// vertical root shift applied.
fn ground_sync(beast: &mut Beast, body: &Body, terrain: &Terrain, spot: &Spot) -> f32 {
    if !grounded(body.locomotion) {
        return 0.0;
    }
    let base = spot.ground(terrain, beast.motion.root.at.xz());
    let dy = base - beast.motion.root.floor;
    beast.motion.root.hold.shift(Vec3::X * dy);
    beast.motion.root.floor = base;
    beast.motion.root.at.y = base;
    beast.motion.root.place.y += dy;
    for foot in &mut beast.motion.feet {
        foot.at.y = foot.at.y.max(spot.ground(terrain, foot.at.xz()));
        foot.to.y = foot.to.y.max(spot.ground(terrain, foot.to.xz()));
    }
    dy
}
