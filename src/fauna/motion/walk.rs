//! Walking on legs: the gait clock, the feet, and the body held over them.

use std::f32::consts::TAU;

use bevy::math::{Quat, Vec2, Vec3};

use super::feet::{Cue, Foot, Stride, on};
use super::gait::ramp;
use super::ik::unit;
use super::rig::{Detail, Drive, Motion};
use super::root::{heading, place, ride, travel};
use super::stance::{attitude, cap};
use super::tune::EASE;
use crate::life::body::Body;

/// Returns what the feet of `m` are asked this step, at velocity `vel`, on legs `leg` long under `gravity`.
fn stride(m: &Motion, vel: Vec3, leg: f32, gravity: f32) -> Stride {
    let beat = (leg / gravity.max(0.1)).sqrt();
    let clock = &m.clock;
    let period = clock.stride / vel.length().max(1e-3);
    Stride {
        vel,
        swing: ((1.0 - clock.duty) * period).clamp(0.25 * beat, 1.2 * beat),
        stance: (clock.duty * period).min(2.0 * beat),
        lift: leg * (0.12 + 0.1 * clock.mix.trot + 0.05 * clock.mix.gallop),
        step: 0.12 * leg,
        reach: 0.75 * leg,
        duty: clock.duty,
    }
}

/// Returns the hip of every leg of `body` in the body frame of `m` and how far it reaches.
fn hips<'a>(m: &'a Motion, body: &'a Body) -> impl Iterator<Item = (Vec3, f32)> + 'a {
    body.legs.iter().map(move |l| {
        let hip = l
            .bones
            .first()
            .and_then(|&b| m.frame.rest.get(usize::from(b)))
            .map_or(Vec3::ZERO, |r| r.0);
        (hip, 0.97 * l.lengths.iter().sum::<f32>())
    })
}

/// Returns the cue of foot `i` of `body` under `m`, which may step out of phase if `free`.
fn cue(m: &Motion, body: &Body, i: usize, free: bool) -> Cue {
    let leg = &body.legs[i];
    let ranks = body.legs.iter().map(|l| l.rank + 1).max().unwrap_or(1);
    let lag = m.clock.lag(body.legs.len(), ranks, leg.rank, leg.side);
    Cue {
        rest: m.root.at + heading(m) * leg.rest.with_y(0.0),
        phase: m.clock.own(lag),
        free,
    }
}

/// Steps every foot of `m` over `ground`: few feet at a time step out of phase while `slow`.
fn plant(
    m: &mut Motion,
    body: &Body,
    (stride, slow): (&Stride, bool),
    ground: &dyn Fn(Vec2) -> f32,
    dt: f32,
) {
    if m.detail != Some(Detail::Full) {
        for foot in &mut m.feet {
            *foot = Foot::new(on(ground, foot.at.x, foot.at.z));
        }
        m.root.easing = true;
    }
    for foot in &mut m.feet {
        m.root.easing |= foot.seat(ground);
    }
    let most = (m.feet.len() / 3).max(1);
    let mut up = m.feet.iter().filter(|f| !f.planted).count();
    for i in 0..m.feet.len() {
        let cue = cue(m, body, i, slow && up < most);
        if m.feet[i].step(&cue, stride, ground, dt) {
            up += 1;
        }
    }
}

/// Puts every foot of `m` where its phase says, on `ground` under it.
fn sketch(m: &mut Motion, body: &Body, stride: &Stride, ground: &dyn Fn(Vec2) -> f32) {
    let dir = unit(stride.vel, heading(m) * Vec3::NEG_Z);
    let length = m.clock.stride;
    for i in 0..m.feet.len() {
        let cue = cue(m, body, i, false);
        m.feet[i].sketch(&cue, stride, (ground, dir), length);
    }
}

/// Returns the height the body of `m` on legs of `body` is held at, held at `hold`, so that every planted
/// foot stays in reach of its hip, and clamps the hold spring's position and upward velocity when it
/// lowers the body. While it eases onto feet set down anew it lowers by no more than [`EASE`] hip heights
/// a second, and stops easing once in reach; else it drops into reach at once.
fn reach(m: &mut Motion, body: &Body, (was, hold): (f32, Vec3), dt: f32) -> f32 {
    let leg = body.height.max(0.05);
    let turn = heading(m) * Quat::from_rotation_x(hold.y) * Quat::from_rotation_z(hold.z);
    let capped = hold
        .x
        .min(cap(&m.feet, hips(m, body), m.root.at, (turn, leg)))
        .min(was + EASE * leg * dt);
    let held = match m.root.easing {
        true => capped.max(was - EASE * leg * dt),
        false => capped,
    };
    m.root.easing &= held > capped;
    if held < hold.x {
        m.root.hold.vel.x = m.root.hold.vel.x.min(0.0);
    }
    m.root.hold.at.x = held;
    held
}

/// Walks `m` for `dt` seconds as `drive` asks over `ground`.
pub fn step(m: &mut Motion, body: &Body, drive: &Drive, ground: &dyn Fn(Vec2) -> f32, dt: f32) {
    let leg = body.height.max(0.05);
    let vel = travel(m, drive, true, dt);
    let base = ride(m, ground);
    let speed = vel.length();
    m.clock.advance(speed * dt, speed, drive.gravity, leg);
    let stride = stride(m, vel, leg, drive.gravity);
    let walking = ramp(0.0, 0.3 * (drive.gravity.max(0.1) * leg).sqrt(), speed);
    let target = match drive.detail {
        Detail::Full => {
            plant(m, body, (&stride, walking < 0.5), ground, dt);
            attitude(&m.feet, m.root.at, heading(m))
        }
        Detail::Cheap => {
            m.root.easing = true;
            sketch(m, body, &stride, ground);
            attitude(&m.feet, m.root.at, heading(m)).with_x(base)
        }
    };
    let was = m.root.hold.at.x;
    let mut hold = m.root.hold.step(&m.tune.body(), target, dt);
    if drive.detail == Detail::Full {
        hold.x = reach(m, body, (was, hold), dt);
    }
    let bob =
        -leg * (0.015 + 0.03 * m.clock.mix.trot) * walking * (2.0 * TAU * m.clock.phase).cos();
    let sway = 0.015 * leg * m.idle.shift() * (1.0 - walking);
    place(m, hold, Vec3::new(sway, bob, 0.0), leg);
}
