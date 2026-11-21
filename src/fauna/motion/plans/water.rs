//! Plans in open water or air: a jelly's pulse and a swimmer's body wave.

use std::f32::consts::{PI, TAU};

use bevy::math::{Quat, Vec2, Vec3};

use super::super::gait::ramp;
use super::super::ik::unit;
use super::super::rig::{Drive, Motion};
use super::super::root::{heading, travel};
use super::super::spring::Spring;
use crate::life::body::Body;

/// Share of a pulse spent contracting: fast, then a slow relax.
const SQUEEZE: f32 = 0.3;

/// Drag on a pulsing body, per second.
const DRAG: f32 = 1.5;

/// Returns how far the bell is squeezed at `cycle`, 0 open to 1 shut, and how hard it pushes now.
fn squeeze(cycle: f32) -> (f32, f32) {
    match cycle < SQUEEZE {
        true => (
            ramp(0.0, 1.0, cycle / SQUEEZE),
            (PI * cycle / SQUEEZE).sin(),
        ),
        false => (
            1.0 - ramp(0.0, 1.0, (cycle - SQUEEZE) / (1.0 - SQUEEZE)),
            0.0,
        ),
    }
}

/// Keeps `at` at least `clear` metres over `ground`.
fn above(at: Vec3, ground: &dyn Fn(Vec2) -> f32, clear: f32) -> Vec3 {
    at.with_y(at.y.max(ground(Vec2::new(at.x, at.z)) + clear))
}

/// Pulses `m`: a fast contraction pushes the bell along its way, drag slows it while it relaxes,
/// and the bell turns its top into its way. The bell keeps its volume as it squeezes.
pub fn pulse(m: &mut Motion, body: &Body, drive: &Drive, ground: &dyn Fn(Vec2) -> f32, dt: f32) {
    let length = body.length.max(0.1);
    let want = drive.velocity.length();
    let rate = (0.4 + 0.5 * want / length + 0.6 * drive.excite).min(2.0);
    m.beat.cycle = (m.beat.cycle + rate * dt).rem_euclid(1.0);
    let (shut, push) = squeeze(m.beat.cycle);
    let way = unit(drive.velocity, Vec3::Y);
    let thrust = (want + 0.1 * length) * DRAG * PI / (2.0 * SQUEEZE);
    let vel = (m.root.vel.at + way * thrust * push * dt) * (-DRAG * dt).exp();
    m.root.vel.at = vel;
    m.root.at = above(m.root.at + vel * dt, ground, 0.6 * length);
    let radial = 1.0 - 0.22 * shut;
    m.beat.squash = Vec3::new(radial, 1.0 / (radial * radial), radial);
    let lean = Quat::from_rotation_arc(Vec3::Y, unit(vel + Vec3::Y * 0.2 * length, Vec3::Y));
    let spring = Spring::new(0.6 * m.tune.f, 0.9, 0.0);
    m.root.turn = m.root.tilt.step(&spring, lean, dt);
    m.root.place = m.root.at;
}

/// Swims `m`: the body follows its velocity in three dimensions, pitches into a climb or a dive,
/// and beats a body wave whose rate grows with speed over length.
pub fn swim(m: &mut Motion, body: &Body, drive: &Drive, ground: &dyn Fn(Vec2) -> f32, dt: f32) {
    let length = body.length.max(0.1);
    let vel = travel(m, drive, false, dt);
    m.root.at = above(m.root.at, ground, 0.6 * body.height.max(0.05));
    let speed = vel.length();
    m.beat.cycle = (m.beat.cycle + (0.5 + 1.4 * speed / length) * dt).rem_euclid(1.0);
    let pitch = vel.y.atan2(vel.with_y(0.0).length()).clamp(-0.8, 0.8) * ramp(0.05, 0.3, speed);
    let beat = (TAU * m.beat.cycle).sin();
    let size = 0.06 * length * ramp(0.0, 0.5 * length, speed + 0.2 * length);
    let aim = heading(m) * Quat::from_rotation_x(pitch);
    m.root.turn = m.root.tilt.step(&m.tune.heading(), aim, dt) * Quat::from_rotation_y(0.08 * beat);
    m.root.place = m.root.at + m.root.turn * Vec3::X * (size * beat);
}
