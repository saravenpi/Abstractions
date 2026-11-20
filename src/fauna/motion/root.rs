//! The root of a body: its velocity on a spring, its heading, and how it is placed over the ground.

use std::f32::consts::{PI, TAU};

use bevy::math::{Quat, Vec2, Vec3};

use super::rig::{Drive, Motion};

/// Speed in metres per second under which a body keeps its heading.
const STILL: f32 = 0.03;

/// Returns `angle` brought within half a turn of 0.
pub fn wrap(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

/// Returns the yaw of `dir` about +Y, 0 for -Z.
pub fn yaw_of(dir: Vec3) -> f32 {
    (-dir.x).atan2(-dir.z)
}

/// Moves the root of `m` by `dt` seconds toward the velocity `drive` asks, kept level if `flat`,
/// turns its heading into its way and returns its velocity.
pub fn travel(m: &mut Motion, drive: &Drive, flat: bool, dt: f32) -> Vec3 {
    let want = if flat {
        drive.velocity.with_y(0.0)
    } else {
        drive.velocity
    };
    let vel = m.root.vel.step(&m.tune.root(), want, dt);
    m.root.at += vel * dt;
    let now = m.root.yaw.at;
    let goal = match vel.with_y(0.0).length() > STILL {
        true => now + wrap(yaw_of(vel) - now),
        false => now,
    };
    m.root.yaw.step(&m.tune.heading(), goal, dt);
    vel
}

/// Returns the heading of `m` as a rotation about +Y.
pub fn heading(m: &Motion) -> Quat {
    Quat::from_rotation_y(m.root.yaw.at)
}

/// Returns the height of `ground` under `at`, and the pitch and roll of the ground there seen
/// from a heading `yaw`, sampled `reach` metres around it.
pub fn slope(ground: &dyn Fn(Vec2) -> f32, at: Vec3, yaw: Quat, reach: f32) -> Vec3 {
    let height = |offset: Vec3| {
        let p = at + yaw * offset;
        ground(Vec2::new(p.x, p.z))
    };
    let (front, back) = (height(Vec3::NEG_Z * reach), height(Vec3::Z * reach));
    let (right, left) = (height(Vec3::X * reach), height(Vec3::NEG_X * reach));
    let span = 2.0 * reach.max(1e-3);
    let middle = 0.25 * (front + back + right + left);
    Vec3::new(
        middle,
        ((front - back) / span).atan(),
        ((right - left) / span).atan(),
    )
}

/// Places the root of `m` at the height, pitch and roll `hold`, shifted by `lift` in the frame of
/// its heading, tilted about a point `pivot` metres above its origin.
pub fn place(m: &mut Motion, hold: Vec3, lift: Vec3, pivot: f32) {
    let yaw = heading(m);
    let turn = yaw * Quat::from_rotation_x(hold.y) * Quat::from_rotation_z(hold.z);
    let up = Vec3::Y * pivot;
    m.root.place = m.root.at.with_y(hold.x) + yaw * lift + up - turn * up;
    m.root.turn = turn;
}
