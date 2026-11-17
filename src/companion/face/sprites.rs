//! The pixel shapes of the face: seven by seven eyes and eight by two mouths, one bit per pixel.

use super::{Expression, Lid};

/// An eye: seven rows of seven pixels, the highest of the seven bits on the left.
pub type Eye = [u8; 7];

/// A mouth: two rows of eight pixels, the highest bit on the left.
pub type Mouth = [u8; 2];

/// Calm open eye.
const OPEN: Eye = [0, 0b0011100, 0b0111110, 0b0111110, 0b0111110, 0b0011100, 0];

/// Half-closed eye.
const HALF: Eye = [0, 0, 0, 0b0111110, 0b0111110, 0b0011100, 0];

/// Nearly shut eye of a sleepy face.
const LOW: Eye = [0, 0, 0, 0, 0b0111110, 0b0011100, 0];

/// Shut eye.
const SHUT: Eye = [0, 0, 0, 0, 0b0111110, 0, 0];

/// Arched happy eye.
const ARCH: Eye = [0, 0, 0b0011100, 0b0110110, 0b1100011, 0, 0];

/// Arched happy eye, one pixel higher.
const ARCH_UP: Eye = [0, 0b0011100, 0b0110110, 0b1100011, 0, 0, 0];

/// Round wide eye with a dark centre.
const RING: Eye = [
    0b0011100, 0b0111110, 0b1110111, 0b1100011, 0b1110111, 0b0111110, 0b0011100,
];

/// Big round eye.
const BIG: Eye = [
    0b0011100, 0b0111110, 0b1111111, 0b1111111, 0b1111111, 0b0111110, 0b0011100,
];

/// Small eye under a brow that rises toward the middle of the face, as drawn for the left eye.
const WORRY: Eye = [0b0000011, 0b0001100, 0b0110000, 0, 0b0011100, 0b0011100, 0];

/// The worried eye one pixel lower, for the tremble.
const WORRY_LOW: Eye = [0b0000011, 0b0001100, 0b0110000, 0, 0, 0b0011100, 0b0011100];

/// Heart.
const HEART: Eye = [
    0, 0b0110110, 0b1111111, 0b1111111, 0b0111110, 0b0011100, 0b0001000,
];

/// Small heart.
const HEART_SMALL: Eye = [0, 0, 0b0010100, 0b0111110, 0b0011100, 0b0001000, 0];

/// Returns an eye flipped left to right.
pub fn mirror(eye: Eye) -> Eye {
    eye.map(|row| row.reverse_bits() >> 1)
}

/// Returns the open left eye of an expression on one of its two frames.
fn open_eye(expr: Expression, beat: bool) -> Eye {
    match (expr, beat) {
        (Expression::Neutral, _) => OPEN,
        (Expression::Happy, false) => ARCH,
        (Expression::Happy, true) => ARCH_UP,
        (Expression::Curious, _) => BIG,
        (Expression::Surprised, _) => RING,
        (Expression::Sleepy, false) => HALF,
        (Expression::Sleepy, true) => LOW,
        (Expression::Worried, false) => WORRY,
        (Expression::Worried, true) => WORRY_LOW,
        (Expression::Love, false) => HEART,
        (Expression::Love, true) => HEART_SMALL,
    }
}

/// Returns the left and right eyes as they appear on the screen.
///
/// The right eye mirrors the left one, except on the curious face where it stays small.
pub fn eyes(expr: Expression, lid: Lid, beat: bool) -> (Eye, Eye) {
    let left = match lid {
        Lid::Open => open_eye(expr, beat),
        Lid::Half => HALF,
        Lid::Closed => SHUT,
    };
    let odd = expr == Expression::Curious && lid == Lid::Open;
    (left, if odd { OPEN } else { mirror(left) })
}

/// Returns the mouth of an expression on one of its two frames.
pub fn mouth(expr: Expression, beat: bool) -> Mouth {
    match (expr, beat) {
        (Expression::Neutral, _) => [0b00011000, 0],
        (Expression::Happy | Expression::Love, false) => [0b01000010, 0b00111100],
        (Expression::Happy | Expression::Love, true) => [0b01111110, 0b00111100],
        (Expression::Curious, _) => [0b00001110, 0],
        (Expression::Surprised, false) => [0b00011000, 0b00011000],
        (Expression::Surprised, true) => [0b00111100, 0b00111100],
        (Expression::Sleepy, false) => [0, 0b00011000],
        (Expression::Sleepy, true) => [0, 0b00111100],
        (Expression::Worried, false) => [0b00111100, 0b01000010],
        (Expression::Worried, true) => [0b00011000, 0b00100100],
    }
}
