//! Diving: the body lies along its way through the water, head down while it goes down, head up
//! while it climbs, level while it swims level, and back to level when it stops, without a jump.

use std::f32::consts::FRAC_PI_2;

use super::FRAME;
use crate::avatar::animate::Anim;
use crate::avatar::blend::Weights;
use crate::avatar::pose::Core;
use crate::avatar::{AvatarPose, Mode};

/// Largest change of the drawn lean of the body in one frame, in radians.
const LEAN_STEP: f32 = 0.05;

/// One leg of a swim: the mode, the speeds ahead and up, and how many frames it lasts.
type Leg = (Mode, f32, f32, usize);

/// Swims every leg in turn and returns the drawn lean below level of every frame, after
/// checking that it never jumps and that the body never rolls.
fn swum(legs: &[Leg]) -> Vec<f32> {
    let mut anim = Anim {
        weights: Weights::only(legs[0].0),
        ..Anim::default()
    };
    let mut leans = Vec::new();
    for &(mode, speed, vertical_speed, frames) in legs {
        let input = AvatarPose {
            mode,
            speed,
            vertical_speed,
            ..AvatarPose::default()
        };
        for _ in 0..frames {
            let pose = anim.step(&input, FRAME);
            assert_eq!(pose.core(Core::Roll), 0.0, "{mode:?}: the body rolls");
            leans.push(pose.core(Core::Pitch) - FRAC_PI_2);
        }
    }
    for (index, pair) in leans.windows(2).enumerate() {
        let jump = (pair[1] - pair[0]).abs();
        assert!(jump < LEAN_STEP, "frame {index}: the body jumps {jump}");
    }
    leans
}

/// A dive at 65 degrees down pitches the body head down along it, a stop brings it back level,
/// a climb pitches it head up, and a level swim under water stays level.
#[test]
fn a_diving_body_lies_along_its_way() {
    let down = 3.2_f32.atan2(1.5);
    let legs = [
        (Mode::Dive, 3.0, 0.0, 120),
        (Mode::Dive, 1.5, -3.2, 180),
        (Mode::Dive, 0.0, 0.0, 240),
        (Mode::Dive, 1.5, 3.2, 180),
        (Mode::Dive, 3.5, 0.0, 240),
    ];
    let leans = swum(&legs);
    let at = |frame: usize| leans[frame - 1];
    assert!(at(120).abs() < 0.01, "level swim leans {}", at(120));
    assert!(
        (at(300) - down).abs() < 0.03,
        "dives at {} for {down}",
        at(300)
    );
    let descent = &leans[120..300];
    assert!(descent.windows(2).all(|pair| pair[1] >= pair[0] - 1e-4));
    assert!(at(540).abs() < 0.02, "stopped at {}", at(540));
    assert!(
        (at(720) + down).abs() < 0.03,
        "climbs at {} for {down}",
        at(720)
    );
    assert!(at(960).abs() < 0.02, "level again at {}", at(960));
}

/// Going down from the surface tips the swimming body head down too, never past 70 degrees.
#[test]
fn a_swimmer_going_down_tips_head_down() {
    let legs = [(Mode::Swim, 3.0, 0.0, 120), (Mode::Swim, 0.2, -4.0, 240)];
    let leans = swum(&legs);
    let (level, down) = (leans[119], leans[359]);
    assert!(down > level + 1.0, "tips from {level} to {down} only");
    assert!(
        down - level < 1.23,
        "tips past 70 degrees: {}",
        down - level
    );
}
