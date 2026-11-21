//! Tests of the hands on the wire: every act comes back as it was, with its side, and the height of
//! the aim comes back in the spare bits.

use bevy::prelude::*;

use super::swing::{pack, unpack};
use crate::avatar::strike::{Act, Dir, Strike};

/// Every act and every side survive the packing, and a stun keeps the time it has left.
#[test]
fn every_act_and_side_comes_back() {
    let acts = [
        Act::Ready,
        Act::Windup(0.2),
        Act::Swing(0.3),
        Act::Recover(0.1),
        Act::Guard(0.5),
        Act::Draw(0.7),
        Act::Stun(0.25, 0.0),
        Act::Stow(0.2),
    ];
    for dir in [Dir::Right, Dir::Left, Dir::Up, Dir::Down] {
        for act in acts {
            let strike = Strike {
                act,
                dir,
                ..Strike::default()
            };
            let (swing, phase) = pack(&strike);
            let got = unpack(swing, phase);
            assert_eq!((got.act, got.dir), (strike.act, strike.dir));
        }
    }
}

/// An aim low at the feet comes back low and ahead, one level aims level, and a record with no aim,
/// as an older client sends it, aims at nothing.
#[test]
fn the_height_of_the_aim_comes_back() {
    let aimed = |aim: Vec3| {
        let strike = Strike {
            act: Act::Windup(0.1),
            dir: Dir::Left,
            aim: Some(aim),
            ..Strike::default()
        };
        let (swing, phase) = pack(&strike);
        let back = unpack(swing, phase);
        assert_eq!((back.act, back.dir), (strike.act, strike.dir));
        back.aim.expect("an aim")
    };
    let low = aimed(Vec3::new(0.1, -0.7, -0.9));
    assert!(low.y < -0.3 && low.z < 0.0, "{low}");
    let level = aimed(Vec3::new(0.2, 0.38, -1.2));
    assert!((level.y - 0.38).abs() < 0.15, "{level}");
    assert_eq!(unpack(pack(&Strike::default()).0, 0.0).aim, None);
}
