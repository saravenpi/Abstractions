//! Plans on the ground without legs: a snake's slither, a slime's ooze and a crystal's tumble.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::math::{Quat, Vec2, Vec3};

use super::super::gait::ramp;
use super::super::rig::{Drive, Motion};
use super::super::root::{heading, place, ride, slope, travel};
use crate::life::body::Body;

/// Moves the root of `m` level over `ground` and holds it on the slope there, riding the ground under
/// it; returns the speed.
fn creep(m: &mut Motion, body: &Body, drive: &Drive, ground: &dyn Fn(Vec2) -> f32, dt: f32) -> f32 {
    let speed = travel(m, drive, true, dt).length();
    ride(m, ground);
    let under = slope(ground, m.root.at, heading(m), 0.25 * body.length.max(0.1));
    m.root.hold.step(&m.tune.body(), under, dt);
    speed
}

/// Slithers `m`: the head swings on a sine laid along the ground, which the spine then follows,
/// so the wave travels from head to tail. The wave grows with speed.
pub fn slither(m: &mut Motion, body: &Body, drive: &Drive, ground: &dyn Fn(Vec2) -> f32, dt: f32) {
    let length = body.length.max(0.1);
    let speed = creep(m, body, drive, ground, dt);
    let wave = 0.5 * length;
    m.beat.cycle = (m.beat.cycle + speed * dt / wave).rem_euclid(1.0);
    let size = 0.08 * length * speed / (speed + 0.25 * length);
    let (sin, cos) = (TAU * m.beat.cycle).sin_cos();
    let hold = m.root.hold.at;
    place(m, hold, Vec3::new(size * sin, 0.0, 0.0), 0.0);
    m.root.turn *= Quat::from_rotation_y(-(size * TAU / wave * cos).atan());
}

/// Oozes `m`: the blob stretches along its way and squashes down at constant volume, with a creeping wobble.
pub fn ooze(m: &mut Motion, body: &Body, drive: &Drive, ground: &dyn Fn(Vec2) -> f32, dt: f32) {
    let length = body.length.max(0.1);
    let speed = creep(m, body, drive, ground, dt);
    m.beat.cycle =
        (m.beat.cycle + dt * (0.5 + 0.8 * drive.excite) + speed * dt / length).rem_euclid(1.0);
    let stretch = 1.0 + 0.35 * speed / (speed + 0.5 * length);
    let wobble = TAU * m.beat.cycle;
    let long = stretch * (1.0 + 0.04 * (wobble + 1.3).sin());
    let tall = (1.0 + 0.07 * wobble.sin()) / stretch.sqrt();
    m.beat.squash = Vec3::new(1.0 / (long * tall), tall, long);
    let hold = m.root.hold.at;
    place(m, hold, Vec3::ZERO, 0.0);
}

/// Tumbles `m`: rigid hops a body length long, a quarter turn forward each, finishing a hop
/// begun when it stops.
pub fn tumble(m: &mut Motion, body: &Body, drive: &Drive, ground: &dyn Fn(Vec2) -> f32, dt: f32) {
    let length = body.length.max(0.1);
    let speed = creep(m, body, drive, ground, dt);
    let hop = 0.8 * length;
    let roll = m.beat.roll;
    let ahead = match (speed > 0.05, roll.fract() > 1e-3) {
        (true, _) => roll + speed * dt / hop,
        (false, true) => (roll + 2.5 * dt).min(roll.floor() + 1.0),
        (false, false) => roll,
    };
    m.beat.roll = ahead.rem_euclid(4.0);
    let part = m.beat.roll.fract();
    let rise = 0.25 * length * (1.0 + drive.excite) * (PI * part).sin();
    let hold = m.root.hold.at;
    let pivot = body.height.max(0.05);
    place(m, hold, Vec3::Y * rise, pivot);
    let angle = -(m.beat.roll.floor() + ramp(0.0, 1.0, part)) * FRAC_PI_2;
    let up = Vec3::Y * pivot;
    let spin = Quat::from_rotation_x(angle);
    m.root.place += m.root.turn * (up - spin * up);
    m.root.turn *= spin;
}
