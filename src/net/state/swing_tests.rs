//! Tests of the hands on the wire: every act comes back as it was, with its side.

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
