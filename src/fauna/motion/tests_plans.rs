//! Bodies that do not walk: slimes keep their volume, jellies push only while contracting,
//! snakes carry a wave down their spine, crystals tumble and land.

use bevy::math::{Vec2, Vec3};

use super::fixtures_soft::{crystal, jelly, slime, snake};
use super::rig::{Detail, Drive, Motion};
use crate::life::body::Body;

/// Returns a drive at `velocity` on an Earth-like planet, alarmed by `excite`.
pub fn drive(velocity: Vec3, excite: f32) -> Drive {
    Drive {
        velocity,
        look: None,
        gravity: 9.81,
        excite,
        detail: Detail::Full,
    }
}

/// Returns the height of a gently rolling test ground at `p`.
pub fn hills(p: Vec2) -> f32 {
    0.3 * (0.4 * p.x).sin() + 0.2 * (0.3 * p.y).cos()
}

/// Returns a body standing at the origin of the hills, facing -Z.
fn start(body: &Body) -> Motion {
    Motion::new(body, Vec3::new(0.0, hills(Vec2::ZERO), 0.0), Vec3::NEG_Z, 4)
}

/// A slime keeps the product of its scales at 1 within 1e-3 at any speed and alarm.
#[test]
fn slime_keeps_its_volume() {
    let body = slime();
    let mut motion = start(&body);
    for frame in 0..600 {
        let speed = (frame as f32 * 0.01).sin().abs() * 1.5;
        motion.step(
            &body,
            &drive(Vec3::new(speed, 0.0, -0.5), (frame % 200) as f32 / 200.0),
            &hills,
            1.0 / 30.0,
        );
        let scale = motion.pose(&body)[0].2;
        assert!(
            (scale.x * scale.y * scale.z - 1.0).abs() < 1e-3,
            "frame {frame}: {scale}"
        );
    }
    assert!(
        motion.beat.squash.z > 1.05,
        "no stretch along its way: {}",
        motion.beat.squash
    );
}

/// A jelly only gains speed while its bell contracts, and it still gets where it is asked.
#[test]
fn jelly_gains_speed_only_while_contracting() {
    let body = jelly();
    let mut motion = Motion::new(&body, Vec3::Y * 3.0, Vec3::NEG_Z, 8);
    let asked = drive(Vec3::new(0.0, 0.2, -0.4), 0.0);
    let (mut speed, mut pushed) = (0.0, 0);
    for _ in 0..900 {
        motion.step(&body, &asked, &|_| 0.0, 1.0 / 30.0);
        let now = motion.velocity().length();
        if now > speed + 1e-6 {
            assert!(
                motion.beat.cycle < 0.3 + 1e-4,
                "sped up relaxing at {}",
                motion.beat.cycle
            );
            pushed += 1;
        }
        speed = now;
    }
    assert!(
        pushed > 20 && motion.root.at.z < -3.0,
        "jelly at {}",
        motion.root.at
    );
    let tentacle = &motion.strands[0].points;
    assert!(
        tentacle[tentacle.len() - 1].y < tentacle[0].y,
        "tentacle not hanging"
    );
}

/// A slithering snake keeps its spine's lengths, lies along the ground, and is bent by its wave.
#[test]
fn snake_carries_a_wave_down_its_spine() {
    let body = snake();
    let mut motion = start(&body);
    let mut bent: f32 = 0.0;
    for frame in 0..400 {
        motion.step(&body, &drive(Vec3::NEG_Z * 0.8, 0.0), &hills, 1.0 / 30.0);
        let spine = &motion.strands[0].points;
        for pair in spine.windows(2) {
            assert!(
                (pair[0].distance(pair[1]) - 0.2).abs() < 1e-4,
                "frame {frame}"
            );
        }
        let (head, tail) = (spine[0], spine[spine.len() - 1]);
        let side = (head - tail).with_y(0.0).normalize().cross(Vec3::Y);
        bent = bent.max(
            spine
                .iter()
                .map(|p| (*p - tail).dot(side).abs())
                .fold(0.0, f32::max),
        );
        let low = spine
            .iter()
            .map(|p| (p.y - hills(Vec2::new(p.x, p.z))).abs())
            .fold(0.0, f32::max);
        assert!(low < 0.2, "frame {frame}: spine {low} m off the ground");
    }
    assert!(bent > 0.05, "spine never bent: {bent}");
}

/// A crystal rolls a quarter turn a hop, and when it stops it finishes its hop on the ground.
#[test]
fn crystal_tumbles_and_lands() {
    let body = crystal();
    let mut motion = start(&body);
    let mut high: f32 = 0.0;
    for _ in 0..90 {
        motion.step(&body, &drive(Vec3::NEG_Z * 1.2, 0.0), &hills, 1.0 / 30.0);
        high = high.max(motion.root.place.y - hills(Vec2::new(motion.root.at.x, motion.root.at.z)));
    }
    for _ in 0..90 {
        motion.step(&body, &drive(Vec3::ZERO, 0.0), &hills, 1.0 / 30.0);
    }
    assert!(high > 0.05, "never hopped: {high}");
    assert!(
        motion.beat.roll.fract() < 1e-3,
        "stopped mid-hop at {}",
        motion.beat.roll
    );
    let up = motion.root.turn * Vec3::Y;
    assert!(
        up.dot(Vec3::Y).abs() > 0.9 || up.y.abs() < 0.3,
        "not resting on a face: {up}"
    );
}
