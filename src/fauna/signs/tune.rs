//! Tuning of the drawn and heard signs of life: how far, how often and how many.
//!
//! Sound falls off as one over the distance in open air (spherical spreading, 6 dB per doubling), and
//! louder animals carry farther: the level of a call grows with body size.

/// Metres along the ground from the local robot within which signs are drawn.
pub const RADIUS: f64 = 60.0;

/// Seconds between two lays of the signs.
pub const LAY: f32 = 1.0;

/// Metres the robot walks from where the watch of the signs was rolled before it is rolled again.
pub const MOVE: f64 = 40.0;

/// Seconds after it was rolled that the watch of the signs is rolled again.
pub const ROLL: f64 = 100.0;

/// Most quads of the mesh of the signs, a fixed size; signs past it are not drawn.
pub const QUADS: usize = 4096;

/// Metres the signs float over the drawn ground, so it never hides them.
pub const LIFT: f32 = 0.03;

/// Seconds between two hearings of the calls.
pub const HEAR: f32 = 0.25;

/// Metres along the ground within which calls are heard.
pub const EARSHOT: f64 = 250.0;

/// Metres from the listener at which a call of a one kilogram animal arrives at half its level; the
/// distance grows as the cube root of the mass.
pub const HALF_LEVEL: f32 = 12.0;

/// Most calls sent to the audio in one hearing, the loudest first.
pub const CALLS: usize = 2;

/// Least level, 0 to 1, a call must arrive at to be sent.
pub const QUIET: f32 = 0.07;
