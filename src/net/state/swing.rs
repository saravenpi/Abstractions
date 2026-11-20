//! The hands of a robot on the wire: what they do with the tool and for how long, packed in two
//! fields of the robot record and unpacked into the strike state the hands pose from.

use crate::avatar::strike::{Act, Dir, Strike};

/// Returns the side `dir` as its two bits.
fn side(dir: Dir) -> u8 {
    match dir {
        Dir::Right => 0,
        Dir::Left => 1,
        Dir::Up => 2,
        Dir::Down => 3,
    }
}

/// Returns the side of two bits.
fn dir_of(bits: u8) -> Dir {
    match bits & 3 {
        0 => Dir::Right,
        1 => Dir::Left,
        2 => Dir::Up,
        _ => Dir::Down,
    }
}

/// Returns the act of `strike` as its rank with the side in bits 3 and 4, and the seconds it has
/// lasted.
pub fn pack(strike: &Strike) -> (u8, f32) {
    let (rank, phase) = match strike.act {
        Act::Ready => (0, 0.0),
        Act::Windup(t) => (1, t),
        Act::Swing(t) => (2, t),
        Act::Recover(t) => (3, t),
        Act::Guard(t) => (4, t),
        Act::Draw(t) => (5, t),
        Act::Stun(left, _) => (6, left),
    };
    (rank | side(strike.dir) << 3, phase)
}

/// Returns the strike state a record of `swing` and `phase` stands for: the act and the side, with
/// nothing landed and no blade yet.
pub fn unpack(swing: u8, phase: f32) -> Strike {
    let act = match swing & 7 {
        1 => Act::Windup(phase),
        2 => Act::Swing(phase),
        3 => Act::Recover(phase),
        4 => Act::Guard(phase),
        5 => Act::Draw(phase),
        6 => Act::Stun(phase, 0.0),
        _ => Act::Ready,
    };
    Strike {
        act,
        dir: dir_of(swing >> 3),
        ..Strike::default()
    }
}
