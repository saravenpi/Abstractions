//! The output guard: no constant offset, a limiter that looks ahead and turns the output down
//! before a peak arrives, and behind it a curve that is linear below its knee, soft above it
//! and never past its ceiling.

use super::filter::glide;
use crate::audio::tune::{LIMIT_AT, LIMIT_TIMES};

/// Level below which the signal passes the curve untouched: where the limiter starts to work.
pub const KNEE: f32 = LIMIT_AT;

/// Highest level the output can reach: -1.5 dBFS, so that the peaks between two samples stay
/// under -1 dBTP.
pub const CEILING: f32 = 0.841;

/// Frames the limiter looks ahead: 1.3 milliseconds at 48 kHz.
pub const LOOKAHEAD: usize = 64;

/// Returns `x` bounded to [`CEILING`], bending smoothly above [`KNEE`]. A non-finite input gives silence.
pub fn limit(x: f32) -> f32 {
    if !x.is_finite() {
        return 0.0;
    }
    let level = x.abs();
    if level <= KNEE {
        return x;
    }
    let room = CEILING - KNEE;
    (KNEE + room * ((level - KNEE) / room).tanh()).min(CEILING) * x.signum()
}

/// Removes any constant offset with a high-pass far below hearing.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DcBlock {
    /// Previous input.
    x: f32,
    /// Previous output.
    y: f32,
}

impl DcBlock {
    /// Runs one sample with the pole `r`, just under 1.
    pub fn run(&mut self, x: f32, r: f32) -> f32 {
        let y = x - self.x + r * self.y;
        self.x = x;
        self.y = if y.abs() < 1e-12 { 0.0 } else { y };
        self.y
    }
}

/// A stereo peak limiter. It delays the sound by [`LOOKAHEAD`] frames and uses that time to
/// turn both channels down together before a peak over [`KNEE`] comes out, then lets go slowly.
/// What still overshoots meets [`limit`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limiter {
    /// The frames on their way out.
    line: [[f32; 2]; LOOKAHEAD],
    /// Where the next frame is written.
    at: usize,
    /// Gain applied now, from 0 to 1.
    gain: f32,
    /// Coefficients of turning down and of letting go.
    pace: (f32, f32),
}

impl Limiter {
    /// Builds a limiter at rest for a sample rate.
    pub fn new(rate: f32) -> Self {
        Self {
            line: [[0.0; 2]; LOOKAHEAD],
            at: 0,
            gain: 1.0,
            pace: (glide(LIMIT_TIMES.0, rate), glide(LIMIT_TIMES.1, rate)),
        }
    }

    /// Takes one frame in and returns the frame that went in [`LOOKAHEAD`] frames ago, bounded.
    pub fn run(&mut self, frame: [f32; 2]) -> [f32; 2] {
        let peak = frame[0].abs().max(frame[1].abs());
        let need = if peak > KNEE { KNEE / peak } else { 1.0 };
        let k = if need < self.gain {
            self.pace.0
        } else {
            self.pace.1
        };
        self.gain += k * (need - self.gain);
        let out = std::mem::replace(&mut self.line[self.at], frame);
        self.at = (self.at + 1) % LOOKAHEAD;
        out.map(|sample| limit(sample * self.gain))
    }
}
