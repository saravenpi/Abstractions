//! Walking over rough ground: planted feet never slide, no foot sinks, legs reach their feet.

use bevy::math::{Quat, Vec2, Vec3};

use super::fixtures::quadruped_with;
use super::fixtures_legs::{biped, crawler};
use super::rig::{Detail, Drive, Motion};
use crate::life::body::Body;

/// Gravity on the test planet, metres per second squared.
const G: f32 = 9.81;

/// Returns the height of a sloped, bumpy test ground at `p`.
fn rough(p: Vec2) -> f32 {
    0.12 * p.x + 0.08 * p.y + 0.15 * (0.9 * p.x).sin() * (0.7 * p.y).cos()
}

/// What a walk measured: the worst slide of a planted foot, the deepest sink and the worst miss
/// of a leg reaching for its foot, in metres.
#[derive(Debug, Default)]
struct Walked {
    /// Largest move of a foot planted before and after a step.
    slide: f32,
    /// Deepest a foot went under the ground.
    sink: f32,
    /// Largest distance between the end of a leg and its foot.
    miss: f32,
}

/// Walks `body` 50 m on a curving path over the rough ground at a speed that changes with
/// `pace(metres walked)` in body lengths per second, and measures the walk after the first step,
/// which sets the feet on the ground.
fn walk(body: &Body, pace: impl Fn(f32) -> f32) -> Walked {
    let mut motion = Motion::new(body, Vec3::ZERO, Vec3::NEG_Z, 11);
    let mut walked = Walked::default();
    let (mut moved, mut frame) = (0.0, 0);
    while moved < 50.0 && frame < 60 * 600 {
        let heading =
            Quat::from_rotation_y(0.15 * (frame as f32 / 60.0).sin() + 0.002 * frame as f32);
        let velocity = heading * Vec3::NEG_Z * pace(moved) * body.length;
        let drive = Drive {
            velocity,
            look: None,
            gravity: G,
            excite: 0.0,
            detail: Detail::Full,
        };
        let (before, at) = (motion.feet.clone(), motion.root.at);
        motion.step(body, &drive, &rough, 1.0 / 30.0);
        moved += (motion.root.at - at).length();
        if frame > 0 {
            measure(&mut walked, body, &motion, &before);
        }
        frame += 1;
    }
    assert!(moved >= 50.0, "walked only {moved} m");
    walked
}

/// Widens `walked` with the step of `motion` on `body` whose feet were `before`.
fn measure(walked: &mut Walked, body: &Body, motion: &Motion, before: &[super::feet::Foot]) {
    let pose = motion.pose(body);
    for (i, (was, now)) in before.iter().zip(&motion.feet).enumerate() {
        if was.planted && now.planted {
            walked.slide = walked.slide.max(was.at.distance(now.at));
        }
        walked.sink = walked
            .sink
            .max(rough(Vec2::new(now.at.x, now.at.z)) - now.at.y);
        let leg = &body.legs[i];
        let (last, length) = (
            usize::from(leg.bones[leg.bones.len() - 1]),
            leg.lengths[leg.lengths.len() - 1],
        );
        let (at, turn, _) = pose[last];
        let end = at + turn * (motion.frame.axes[last] * length);
        if now.planted {
            walked.miss = walked.miss.max(end.distance(now.at));
        }
    }
}

/// Every walker crosses 50 m of sloped, bumpy ground at a walk without a planted foot moving
/// a millimetre or a foot sinking two centimetres, its legs reaching their feet.
#[test]
fn walkers_keep_their_feet_on_rough_ground() {
    let bodies = [
        quadruped_with(2, 0.3),
        quadruped_with(4, 0.35),
        biped(),
        crawler(3, 0.9),
        crawler(6, 1.6),
    ];
    for (i, body) in bodies.iter().enumerate() {
        let walked = walk(body, |_| 0.8);
        assert!(
            walked.slide < 1e-3,
            "body {i}: planted foot moved {} m",
            walked.slide
        );
        assert!(walked.sink < 0.02, "body {i}: foot sank {} m", walked.sink);
        assert!(
            walked.miss < 0.03 * body.height,
            "body {i}: leg missed its foot by {} m",
            walked.miss
        );
    }
}

/// A quadruped speeding up from a walk to a gallop and back keeps its planted feet still.
#[test]
fn gait_changes_keep_feet_planted() {
    let body = quadruped_with(2, 0.3);
    let walked = walk(&body, |moved| {
        0.4 + 3.0 * (moved / 50.0 * std::f32::consts::PI).sin()
    });
    assert!(walked.slide < 1e-3, "planted foot moved {} m", walked.slide);
    assert!(walked.sink < 0.02, "foot sank {} m", walked.sink);
}

/// A body turning on the spot steps its feet back under it, and the same seed walks the same way.
#[test]
fn turning_on_the_spot_steps_and_repeats() {
    let body = quadruped_with(2, 0.3);
    let run = || {
        let mut motion = Motion::new(&body, Vec3::ZERO, Vec3::NEG_Z, 5);
        let mut lifted = 0;
        for frame in 0..240 {
            let velocity = if frame < 20 {
                Vec3::X * 0.3
            } else {
                Vec3::ZERO
            };
            let drive = Drive {
                velocity,
                look: Some(Vec3::new(3.0, 1.0, 2.0)),
                gravity: G,
                excite: 0.2,
                detail: Detail::Full,
            };
            let before = motion.feet.clone();
            motion.step(&body, &drive, &rough, 1.0 / 30.0);
            lifted += before
                .iter()
                .zip(&motion.feet)
                .filter(|(a, b)| a.planted && !b.planted)
                .count();
        }
        (lifted, motion.pose(&body))
    };
    let (lifted, pose) = run();
    assert!(lifted >= 4, "only {lifted} steps while turning");
    assert_eq!(pose, run().1);
}
