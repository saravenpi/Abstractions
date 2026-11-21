//! The cabin as a small room, about 1.5 cubic metres: a few early reflections off the canopy and
//! the panels within 2 to 6 milliseconds, alternating between the ears, then a short dense tail
//! that dies in about 0.15 seconds, and the boxy rise of its lowest modes between 200 and 500 Hz.
//!
//! It is cheap: four taps of one delay line, one damped feedback comb and one band-pass, about 20
//! floating point operations per sample, and silent while the ears are outside. No convolution.

use super::tune::{BOX, COMB, ROOM_LINE, SEAT, TAPS, WET};
use crate::audio::dsp::filter::{Lag, Svf, pole};

/// Mask that wraps an index into the lines.
const MASK: usize = ROOM_LINE - 1;

/// The cabin: what goes in comes back as its reflections only, in two channels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Room {
    /// What came in, for the early reflections.
    early: [f32; ROOM_LINE],
    /// What the tail sends round again.
    tail: [f32; ROOM_LINE],
    /// Where the next sample is written in both lines.
    at: usize,
    /// The early reflections as delays in samples, their gains and their ears.
    taps: [(usize, f32, usize); 4],
    /// Delay of the tail in samples, and the gain each pass keeps.
    comb: (usize, f32),
    /// The damping of the tail: the walls take the highs on each pass.
    damp: (Lag, f32),
    /// The modes of the cabin.
    modes: Svf,
}

impl Room {
    /// Builds a silent cabin for a sample `rate`. The gain of the tail follows from its decay: a
    /// pass of `d` seconds keeps `10^(-3 d / T60)`, so the tail is 60 dB down after `T60`.
    pub fn new(rate: f32) -> Self {
        let samples = |ms: f32| ((ms * 1.0e-3 * rate) as usize).clamp(1, ROOM_LINE - 2);
        let taps = TAPS.map(|(ms, gain, ear)| (samples(ms), gain, ear));
        let loop_seconds = COMB.0 * 1.0e-3;
        let keep = 10.0_f32.powf(-3.0 * loop_seconds / COMB.1);
        let mut modes = Svf::default();
        modes.tune(BOX.0, BOX.1, rate);
        Self {
            early: [0.0; ROOM_LINE],
            tail: [0.0; ROOM_LINE],
            at: 0,
            taps,
            comb: (samples(COMB.0), keep),
            damp: (Lag::default(), pole(COMB.2, rate)),
            modes,
        }
    }

    /// Returns the sample written `late` samples before the next one in `line`.
    fn read(line: &[f32; ROOM_LINE], at: usize, late: usize) -> f32 {
        line[(at + ROOM_LINE - late) & MASK]
    }

    /// Renders the reflections of one sample `x` of what sounds in the cabin.
    pub fn frame(&mut self, x: f32) -> [f32; 2] {
        let boxy = x + BOX.2 * self.modes.band(x);
        self.early[self.at] = boxy;
        let mut out = [0.0; 2];
        let mut first = 0.0;
        for (late, gain, ear) in self.taps {
            let echo = Self::read(&self.early, self.at, late) * gain;
            out[ear] += echo;
            first += echo;
        }
        let back = Self::read(&self.tail, self.at, self.comb.0);
        let damped = self.damp.0.to(back, self.damp.1);
        let tail = first + self.comb.1 * damped;
        self.tail[self.at] = tail;
        self.at = (self.at + 1) & MASK;
        out.map(|side| (side + 0.5 * tail) * WET)
    }
}

/// The seat: what the ships make, quieter for ears in the cabin than for a chase view, and the
/// cabin around those ears.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seat {
    /// The cabin.
    room: Room,
}

impl Seat {
    /// Builds an empty cabin for a sample `rate`.
    pub fn new(rate: f32) -> Self {
        Self {
            room: Room::new(rate),
        }
    }

    /// Renders one frame: the `ships` brought down to [`SEAT`] as far as the ears are in the
    /// `cabin`, from 0 to 1, with the reflections of the cabin of them and of what the one-shot
    /// sounds `sent` it.
    pub fn frame(&mut self, ships: [f32; 2], sent: f32, cabin: f32) -> [f32; 2] {
        let gain = 1.0 - (1.0 - SEAT) * cabin.clamp(0.0, 1.0);
        let inside = 0.5 * (ships[0] + ships[1]) * SEAT * cabin + sent;
        let echo = self.room.frame(inside);
        [ships[0] * gain + echo[0], ships[1] * gain + echo[1]]
    }
}
