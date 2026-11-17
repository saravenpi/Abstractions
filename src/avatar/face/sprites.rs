//! The pixel shapes of the face: seven by seven eyes and eight by two mouths, one bit per pixel.

use super::{Lid, Mien};

/// An eye: seven rows of seven pixels, the highest of the seven bits on the left.
pub type Eye = [u8; 7];

/// A mouth: two rows of eight pixels, the highest bit on the left.
pub type Mouth = [u8; 2];

/// Calm open eye, a little taller than the companion's.
const OPEN: Eye = [
    0b0011100, 0b0111110, 0b0111110, 0b0111110, 0b0111110, 0b0011100, 0,
];

/// Half-closed eye.
const HALF: Eye = [0, 0, 0, 0b0111110, 0b0111110, 0b0011100, 0];

/// Nearly shut eye of a sleepy face.
const LOW: Eye = [0, 0, 0, 0, 0b0111110, 0b0011100, 0];

/// Shut eye.
const SHUT: Eye = [0, 0, 0, 0, 0b0111110, 0, 0];

/// Narrowed eye under a brow that drops toward the middle of the face, as drawn for the left eye.
const SET: Eye = [0b1100000, 0b0011100, 0b0000011, 0, 0b0111110, 0b0111110, 0];

/// Round wide eye with a dark centre.
const RING: Eye = [
    0b0011100, 0b0111110, 0b1110111, 0b1100011, 0b1110111, 0b0111110, 0b0011100,
];

/// Arched happy eye.
const ARCH: Eye = [0, 0, 0b0011100, 0b0110110, 0b1100011, 0, 0];

/// Arched happy eye, one pixel higher.
const ARCH_UP: Eye = [0, 0b0011100, 0b0110110, 0b1100011, 0, 0, 0];

/// Eye screwed shut, pointing toward the middle of the face, as drawn for the left eye.
const SCREW: Eye = [0, 0b0110000, 0b0001100, 0b0000011, 0b0001100, 0b0110000, 0];

/// Small round eye.
const DOT: Eye = [0, 0, 0b0011100, 0b0011100, 0b0011100, 0, 0];

/// Returns an eye flipped left to right.
pub fn mirror(eye: Eye) -> Eye {
    eye.map(|row| row.reverse_bits() >> 1)
}

/// Returns the open left eye of a face on one of its two frames.
fn open_eye(mien: Mien, beat: bool) -> Eye {
    match (mien, beat) {
        (Mien::Neutral, _) => OPEN,
        (Mien::Focused, _) => SET,
        (Mien::Wide, _) => RING,
        (Mien::Happy, false) => ARCH,
        (Mien::Happy, true) => ARCH_UP,
        (Mien::Wince, _) => SCREW,
        (Mien::Sleepy, false) => HALF,
        (Mien::Sleepy, true) => LOW,
        (Mien::Submerged, _) => DOT,
    }
}

/// Returns the left and right eyes as they appear on the screen. The right eye mirrors the left one.
///
/// A wince keeps its shape through a blink: those eyes are already shut.
pub fn eyes(mien: Mien, lid: Lid, beat: bool) -> (Eye, Eye) {
    let left = match (lid, mien) {
        (Lid::Open, _) | (_, Mien::Wince) => open_eye(mien, beat),
        (Lid::Half, _) => HALF,
        (Lid::Closed, _) => SHUT,
    };
    (left, mirror(left))
}

/// Returns the mouth of a face on one of its two frames.
pub fn mouth(mien: Mien, beat: bool) -> Mouth {
    match (mien, beat) {
        (Mien::Neutral, _) => [0b00111100, 0],
        (Mien::Focused, _) => [0b01111110, 0],
        (Mien::Wide, false) => [0b00011000, 0b00011000],
        (Mien::Wide, true) => [0b00111100, 0b00111100],
        (Mien::Happy, false) => [0b01000010, 0b00111100],
        (Mien::Happy, true) => [0b01111110, 0b00111100],
        (Mien::Wince, _) => [0b01011010, 0b10100101],
        (Mien::Sleepy, false) => [0, 0b00011000],
        (Mien::Sleepy, true) => [0, 0b00111100],
        (Mien::Submerged, false) => [0b01100110, 0b10011001],
        (Mien::Submerged, true) => [0b10011001, 0b01100110],
    }
}
