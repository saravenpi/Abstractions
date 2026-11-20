//! Rolls and tumbles: the trunk turns whole turns as far as the body travels, and ends upright.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::FRAME;
use super::stage::Touchdown;
use crate::avatar::Mode;
use crate::avatar::pose::{Core, Limb};
use crate::avatar::touch::floor::lowest;
use crate::avatar::tune::SOLE_Y;
use crate::gear::Mode as Flight;
use crate::gear::tune::ROLL_TIME;

/// Returns a body that hit the ground at `velocity` and took the first frame of its touchdown.
fn touched(velocity: Vec3) -> Touchdown {
    let mut body = Touchdown::new(Flight::Fall, velocity, (0.0, 0.0), Mode::Fall);
    body.hit();
    body.ground(1);
    body
}

/// Runs a body through its roll and on for a second, and returns the largest turn about `axis`
/// and the largest gap between the share of the turn and the share of the distance travelled.
fn rolled(body: &mut Touchdown, axis: Vec2) -> (f32, f32) {
    let (mut along, mut log) = (0.0, Vec::new());
    while body.pose.mode == Mode::Roll {
        along += body.pose.speed * FRAME;
        log.push((along, body.anim.clock.turn.dot(axis)));
        body.ground(1);
    }
    body.ground(60);
    let most = log.iter().map(|(_, turn)| *turn).fold(f32::MIN, f32::max);
    let gap = log
        .iter()
        .map(|(at, turn)| (at / along - turn / most).abs());
    (most, gap.fold(0.0, f32::max))
}

/// Asserts that the body of `body` stands upright on its feet again.
fn upright(body: &Touchdown, what: &str) {
    let pose = body.anim.pose;
    assert_eq!(body.anim.clock.turn, Vec2::ZERO, "{what}: still turned");
    assert!(
        pose.core(Core::Pitch).abs() < 0.25,
        "{what}: leans {pose:?}"
    );
    assert!(pose.core(Core::Roll).abs() < 0.1, "{what}: rolled {pose:?}");
    assert!(
        (lowest(&pose) - SOLE_Y).abs() < 0.03,
        "{what}: not on its feet"
    );
}

/// A fast touchdown rolls once over the head the way it came in, or once over the side, as far
/// through the turn as through the distance the roll carries it, and ends upright on its feet.
#[test]
fn the_roll_turns_once_and_ends_upright() {
    let cases = [
        (Vec3::new(0.0, -14.0, -16.0), Vec2::X),
        (Vec3::new(0.0, -14.0, 16.0), Vec2::NEG_X),
        (Vec3::new(16.0, -14.0, 0.0), Vec2::Y),
        (Vec3::new(-16.0, -14.0, 2.0), Vec2::NEG_Y),
    ];
    for (velocity, axis) in cases {
        let mut body = touched(velocity);
        assert_eq!(body.pose.mode, Mode::Roll, "{velocity}");
        let (most, gap) = rolled(&mut body, axis);
        assert!((most - TAU).abs() < 0.05, "{velocity}: turned {most}");
        assert!(
            gap < 0.06,
            "{velocity}: the turn strays {gap} from the distance"
        );
        upright(&body, &format!("{velocity}"));
    }
}

/// A dive at 60 m/s into the ground tumbles: more than one turn in the time of a roll, the
/// limbs flailing, then the body gets up.
#[test]
fn a_crash_tumbles_and_gets_up() {
    let mut body = touched(Vec3::new(0.0, -42.0, -42.0));
    let start = body.frames.len();
    let (most, _) = rolled(&mut body, Vec2::X);
    assert!((most - 2.0 * TAU).abs() < 0.05, "turned {most}");
    let flung = body.frames[start..]
        .iter()
        .map(|(pose, _)| (pose.limb(Limb::Shoulder, 0) - pose.limb(Limb::Shoulder, 1)).abs());
    let most = flung.fold(0.0, f32::max);
    assert!(most > 0.3, "the arms only part by {most}");
    let frames = (ROLL_TIME / FRAME) as usize;
    assert!(
        body.frames.len() > start + frames,
        "the tumble was cut short"
    );
    upright(&body, "tumble");
}
