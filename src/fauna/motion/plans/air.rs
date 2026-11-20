//! Plans in the air: a flyer's flap, glide and bank, and a swarm member's bob and flutter.
//!
//! Flocking itself is the runtime's job: a swarm member only follows the velocity it is given.

use std::f32::consts::TAU;

use bevy::math::{Quat, Vec2, Vec3};

use super::super::gait::ramp;
use super::super::rig::{Drive, Motion};
use super::super::root::{heading, travel};
use crate::effects::creatures::anim::{glide, wing_angle};
use crate::life::body::Body;

/// Moves `m` through the air, at least `clear` metres over `ground`; returns its velocity.
fn aloft(m: &mut Motion, drive: &Drive, ground: &dyn Fn(Vec2) -> f32, clear: f32, dt: f32) -> Vec3 {
    let vel = travel(m, drive, false, dt);
    let floor = ground(Vec2::new(m.root.at.x, m.root.at.z)) + clear;
    m.root.at.y = m.root.at.y.max(floor);
    vel
}

/// Flies `m`: wings flap at the body's rate, faster when alarmed or climbing, and glide now and
/// then at speed; the body banks into its turn and pitches into a climb or a dive.
pub fn fly(m: &mut Motion, body: &Body, drive: &Drive, ground: &dyn Fn(Vec2) -> f32, dt: f32) {
    let (span, rate) = body.wings.map_or((body.length, 4.0), |w| (w.span, w.rate));
    let vel = aloft(m, drive, ground, 0.5 * span.max(0.1), dt);
    let (speed, climb) = (vel.length(), vel.y);
    let pace = 1.0 + 0.5 * drive.excite + 0.3 * ramp(0.0, 2.0, climb);
    m.beat.cycle = (m.beat.cycle + rate * pace * dt).rem_euclid(1.0);
    let soar =
        glide(m.time, m.beat.seed, 0.5) * ramp(1.0, 3.0, speed) * (1.0 - ramp(0.2, 1.0, climb));
    m.beat.wing = wing_angle(TAU * m.beat.cycle, soar, 0.9, 0.15);
    let bank = (0.15 * m.root.yaw.vel * speed).clamp(-0.9, 0.9);
    let pitch = 0.7 * climb.atan2(vel.with_y(0.0).length().max(0.1));
    let aim = heading(m) * Quat::from_rotation_x(pitch) * Quat::from_rotation_z(bank);
    m.root.turn = m.root.tilt.step(&m.tune.body(), aim, dt);
    let bob = -0.03 * span * (1.0 - soar) * (TAU * m.beat.cycle).sin();
    m.root.place = m.root.at + Vec3::Y * bob;
}

/// Moves a swarm member `m`: it follows its velocity with a seeded bob and a quick flutter.
pub fn swarm(m: &mut Motion, body: &Body, drive: &Drive, ground: &dyn Fn(Vec2) -> f32, dt: f32) {
    let length = body.length.max(0.02);
    let rate = body.wings.map_or(7.0, |w| w.rate);
    let vel = aloft(m, drive, ground, 2.0 * length, dt);
    m.beat.cycle = (m.beat.cycle + rate * (1.0 + 0.5 * drive.excite) * dt).rem_euclid(1.0);
    m.beat.wing = wing_angle(TAU * m.beat.cycle, 0.0, 1.0, 0.2);
    let bob = 1.5 * length * (TAU * (0.9 * m.time + m.beat.seed)).sin();
    let flutter = 0.25 * (TAU * (2.3 * m.time + m.beat.seed)).sin();
    let pitch = 0.5 * vel.y.atan2(vel.with_y(0.0).length().max(0.1));
    m.root.turn = heading(m) * Quat::from_rotation_x(pitch) * Quat::from_rotation_z(flutter);
    m.root.place = m.root.at + Vec3::Y * bob;
}
