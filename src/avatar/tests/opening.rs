//! Opening the wings: the body passes from the jump, the run or the fall it is in through the
//! spread into the glide in one motion, lying exactly at the attitude the gear flies.

use bevy::prelude::*;

use super::aloft::{Aloft, SNAP, spread};

/// The starts of a glide: a name and the velocity at the press, in metres per second: the top
/// of a jump at a walk, a run off a cliff, and a fall of two and a half seconds.
const STARTS: [(&str, Vec3); 3] = [
    ("jump", Vec3::new(0.0, 0.3, -5.0)),
    ("cliff", Vec3::new(0.0, -1.0, -8.5)),
    ("fall", Vec3::new(0.0, -25.0, -1.0)),
];

/// Returns the angle in radians between the head of a body turned by `body` and the local up.
fn tipped(body: Quat) -> f32 {
    (body * Vec3::Y).angle_between(Vec3::Y)
}

/// From every start the opening is one motion: no frame turns the body by more than [`SNAP`].
/// The top of a jump and a run start upright and lie forward, by a quarter turn and more within
/// the second; a fall opens from the arch it was in. A second after the press the gear lies
/// prone and the body is drawn at the very attitude the gear flies, to a degree.
#[test]
fn the_body_lies_into_the_wings_as_the_gear_lays_it() {
    for (start, velocity) in STARTS {
        let mut aloft = Aloft::falling(velocity);
        let before = tipped(aloft.body);
        let mut worst = 0.0_f32;
        for _ in 0..60 {
            worst = worst.max(aloft.tick(&spread()));
        }
        let (prone, off) = (
            aloft.state.prone,
            aloft.body.angle_between(aloft.state.attitude),
        );
        let what = format!("{start}: from {before} rad off upright, prone {prone}, off by {off}");
        assert!(worst < SNAP, "{what}: a frame turns {worst} rad");
        assert!(prone > 0.9 && off < 0.02, "{what}");
        assert!(tipped(aloft.body) > 1.0, "{what}: still upright");
        if start != "fall" {
            assert!(before < 0.3, "{what}: not upright at the press");
        }
    }
}

/// While the wings open the gear lays the body along them as they spread, from rest at the top
/// of a jump: the share it lies prone is never ahead of the spread and never more than three
/// quarters behind it, and it is nine tenths prone within 0.9 s of the press.
#[test]
fn the_lie_follows_the_spread() {
    let mut aloft = Aloft::falling(STARTS[0].1);
    for frame in 0..60 {
        aloft.tick(&spread());
        let (spread, prone) = (aloft.state.spread(), aloft.state.prone);
        let what = format!("frame {frame}: prone {prone} at spread {spread}");
        assert!(prone > spread - 0.75 && prone <= spread + 0.02, "{what}");
        assert!(frame < 54 || prone > 0.9, "{what}");
    }
}
