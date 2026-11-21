//! Folding the wings: the body goes from the glide into the fall by the shortest rotation,
//! from every attitude it may fold in, a half loop included.

use bevy::prelude::*;

use super::aloft::{Aloft, SNAP};
use crate::gear::GearInput;

/// Seconds the body is watched for after the wings are folded.
const AFTER: f32 = 1.5;

/// Radians the body may turn in all beyond the angle between where it folded and where it
/// ends: the fall it folds into keeps moving while it settles.
const SPARE: f32 = 0.6;

/// Returns the input of the keys `pitch` and `roll` held on the wings, the boosters on or not.
fn held(pitch: f32, roll: f32, boost: bool) -> GearInput {
    GearInput {
        deploy: true,
        boost,
        pitch,
        roll,
        ..GearInput::default()
    }
}

/// Returns the glides to fold from: level, banked by the roll key, nose down by the forward
/// key, and after a half loop under power, the right way up again and read past the vertical.
fn glides() -> [(&'static str, Aloft); 4] {
    let level = Aloft::gliding(Vec3::new(0.0, -7.0, -30.0));
    let mut banked = Aloft::gliding(Vec3::new(0.0, -7.0, -30.0));
    banked.run(&held(0.0, 1.0, false), 0.4);
    let mut down = Aloft::gliding(Vec3::new(0.0, -7.0, -30.0));
    down.run(&held(-1.0, 0.0, false), 1.5);
    let mut over = Aloft::gliding(Vec3::new(0.0, 0.0, -75.0));
    over.run(&held(0.0, 0.0, true), 2.0);
    let mut frames = 0;
    while (over.state.frame() * Vec3::Y).y > -0.8 && frames < 600 {
        over.tick(&held(1.0, 0.0, true));
        frames += 1;
    }
    over.run(&held(0.0, 0.0, true), 1.5);
    [
        ("level", level),
        ("banked", banked),
        ("nose down", down),
        ("half loop", over),
    ]
}

/// Folding the wings from every glide turns the body by at most the angle between where it
/// folded and where it ends, and a little for the fall that moves meanwhile: never round the
/// long way, never a whole flip, and no frame turns it by more than [`SNAP`]. After a half loop
/// the gear reads the body past the vertical with its heading turned round: that changes
/// nothing either.
#[test]
fn folding_the_wings_takes_the_shortest_way() {
    for (what, mut aloft) in glides() {
        let from = aloft.body;
        let (mut swept, mut worst) = (0.0, 0.0_f32);
        for _ in 0..(AFTER * 60.0) as u32 {
            let turn = aloft.tick(&GearInput::default());
            (swept, worst) = (swept + turn, worst.max(turn));
        }
        let direct = from.angle_between(aloft.body);
        let said = format!("{what}: turned {swept} rad for {direct}, worst frame {worst}");
        assert!(swept <= direct + SPARE && swept < 2.5, "{said}");
        assert!(worst < SNAP, "{said}");
    }
}
