//! Tests of a body on the drawn ground: walking, running, fleeing and charging up a slope and over a
//! bump, near or far, every foot stays on the ground, a far body rides the ground under it without lag
//! and pitches with the slope, and neither a body coming near nor a ground drawn at another level
//! makes it sink.

use bevy::math::{Vec2, Vec3};

use super::fixtures::quadruped_with;
use super::rig::{Detail, Drive, Motion};
use super::tests_size::HIDE;
use crate::life::body::{Body, soup};

/// Seconds per step.
const DT: f32 = 1.0 / 30.0;

/// Speeds in metres per second of a walk, a run, a flight and a charge of a body with hips 1 m high.
const SPEEDS: [f32; 4] = [1.5, 4.0, 6.0, 8.0];

/// Metres the finer ground of a streamed chunk lies off the coarser one at most.
const GAP: f32 = 0.1;

/// Rise of the test slope, metres per metre along x.
const RISE: f32 = 0.25;

/// Returns the height of a slope rising along x with a bump 0.6 m high on it at x = 12 m.
fn hill(p: Vec2) -> f32 {
    let bump = 0.6 * (-((p.x - 12.0).powi(2) + p.y * p.y) / 8.0).exp();
    RISE * p.x + bump
}

/// Returns the drive of a body going up the slope at `speed` in `detail`.
fn uphill(speed: f32, detail: Detail) -> Drive {
    Drive {
        velocity: Vec3::X * speed,
        look: None,
        gravity: 9.81,
        excite: 0.8,
        detail,
    }
}

/// Returns how far the body origin of `m` stands under `ground` below its root, in metres.
fn sunk(m: &Motion, ground: &dyn Fn(Vec2) -> f32) -> f32 {
    ground(Vec2::new(m.root.at.x, m.root.at.z)) - m.root.hold.at.x
}

/// Asserts that no foot of `m` stands 2 cm under `ground`, nor a planted one 1 cm off it, and that no
/// drawn corner of a part of `body` posed by `m` goes 2 cm into it, the legs left out: their feet stand
/// for them.
fn grounded(m: &Motion, body: &Body, ground: &dyn Fn(Vec2) -> f32, what: &str) {
    for (i, foot) in m.feet.iter().enumerate() {
        let under = ground(Vec2::new(foot.at.x, foot.at.z)) - foot.at.y;
        assert!(under < 0.02, "{what}: foot {i} {under} m under the ground");
        assert!(
            !foot.planted || under.abs() < 0.01,
            "{what}: planted foot {i} {under} m off the ground"
        );
    }
    let legs: Vec<usize> = body
        .legs
        .iter()
        .flat_map(|l| l.bones.iter().map(|&b| usize::from(b)))
        .collect();
    for (i, (part, (at, turn, size))) in body.parts.iter().zip(m.pose(body)).enumerate() {
        if legs.contains(&i) {
            continue;
        }
        for corner in soup(part, &HIDE).faces.iter().flat_map(|f| f.corners) {
            let p = at + turn * (corner * size);
            let under = ground(Vec2::new(p.x, p.z)) - p.y;
            assert!(under < 0.02, "{what}: part {i} {under} m in the ground");
        }
    }
}

/// Steps `m` of `body` `frames` times as `drive` asks over `ground`, checking its feet and its parts
/// stay on the ground every step, and returns the deepest its body origin
/// stood under the ground.
fn run(
    m: &mut Motion,
    body: &Body,
    (drive, ground): (&Drive, &dyn Fn(Vec2) -> f32),
    frames: usize,
) -> f32 {
    let mut deepest = f32::MIN;
    for frame in 0..frames {
        m.step(body, drive, &ground, DT);
        let what = format!(
            "{:?} at {} m/s, frame {frame}",
            drive.detail, drive.velocity.x
        );
        grounded(m, body, ground, &what);
        deepest = deepest.max(sunk(m, ground));
    }
    deepest
}

/// At a walk, a run, a flight and a charge up a slope and over a bump, near or far, no foot and no
/// drawn part goes into the ground, and a far body holds its origin on the ground under it within a
/// centimetre, with no lag at any speed, and past the bump pitches nose up with the slope as a near
/// one does over its feet.
#[test]
fn bodies_stay_on_the_ground_at_every_pace() {
    let body = quadruped_with(1, 0.4);
    for speed in SPEEDS {
        for detail in [Detail::Cheap, Detail::Full] {
            let mut m = Motion::new(&body, Vec3::ZERO, Vec3::X, 3);
            let frames = (24.0 / speed / DT) as usize;
            let deepest = run(&mut m, &body, (&uphill(speed, detail), &hill), frames);
            println!(
                "{detail:?} at {speed} m/s: body origin at most {deepest:.3} m under the ground"
            );
            if detail == Detail::Cheap {
                assert!(deepest.abs() < 0.01, "far body {deepest} m off the ground");
                let pitch = m.root.hold.at.y;
                let rise = RISE.atan();
                assert!(
                    (pitch - rise).abs() < 0.03,
                    "at {speed} m/s pitched {pitch} on {rise}"
                );
            }
        }
    }
}

/// A far body that comes near, its feet sketched until then, keeps its feet on the ground, and its origin
/// moves onto the ground its feet give by less than 10 cm a frame: no more than its legs need to reach
/// the feet behind it on the slope.
#[test]
fn a_body_coming_near_does_not_sink() {
    let body = quadruped_with(1, 0.4);
    for speed in SPEEDS {
        let mut m = Motion::new(&body, Vec3::ZERO, Vec3::X, 7);
        run(&mut m, &body, (&uphill(speed, Detail::Cheap), &hill), 40);
        let mut last = sunk(&m, &hill);
        for frame in 0..30 {
            m.step(&body, &uphill(speed, Detail::Full), &hill, DT);
            let what = format!("coming near at {speed} m/s, frame {frame}");
            grounded(&m, &body, &hill, &what);
            let now = sunk(&m, &hill);
            assert!(
                (now - last).abs() < 0.1,
                "at {speed} m/s the body jumped {} m",
                now - last
            );
            last = now;
        }
    }
}

/// When the ground under a walking body is drawn at another level, as when a finer chunk streams in,
/// its planted feet stand on the new ground at once and its origin moves with the ground: off the
/// ground under it by no more than the two grounds differ, which its legs may take up near.
#[test]
fn a_ground_drawn_anew_carries_the_feet() {
    let body = quadruped_with(1, 0.4);
    let finer = |p: Vec2| hill(p) + GAP * (3.0 * p.x).sin() * (3.0 * p.y).cos();
    for detail in [Detail::Cheap, Detail::Full] {
        let mut m = Motion::new(&body, Vec3::ZERO, Vec3::X, 9);
        run(&mut m, &body, (&uphill(SPEEDS[0], detail), &hill), 60);
        let before = sunk(&m, &hill);
        m.step(&body, &uphill(SPEEDS[0], detail), &finer, DT);
        grounded(
            &m,
            &body,
            &finer,
            &format!("{detail:?} on the finer ground"),
        );
        let after = sunk(&m, &finer);
        assert!(
            (after - before).abs() < GAP + 0.02,
            "{detail:?}: the body moved {} m off the ground",
            after - before
        );
    }
}
