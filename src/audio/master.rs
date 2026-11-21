//! The end of the mix: the music gives way to loud effects, the reduced dynamic range rides the
//! level when the player asks for it, both ears get the same sound when the player asks for
//! that, and the limiter guards the output.

use super::dsp::filter::{Lag, glide, pole};
use super::dsp::limit::{DcBlock, Limiter};
use super::tune::{DUCK, DUCK_TIMES, PIVOT, RATIO, REACH, RIDE};
use crate::render::smoothstep;

/// Corner in hertz of the filter that removes any constant offset.
const DC_HZ: f32 = 12.0;

/// How the music gives way to the effects.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Duck {
    /// Mean square level of the effects.
    power: Lag,
    /// Coefficients of a rise and of a fall of that level.
    pace: (f32, f32),
}

impl Duck {
    /// Builds a duck at rest for a sample rate.
    pub fn new(rate: f32) -> Self {
        Self {
            power: Lag::default(),
            pace: (glide(DUCK_TIMES.0, rate), glide(DUCK_TIMES.1, rate)),
        }
    }

    /// Hears one frame of the effects and returns the gain of the music under them, from
    /// [`DUCK`] to 1.
    pub fn gain(&mut self, effects: [f32; 2]) -> f32 {
        let now = 0.5 * (effects[0] * effects[0] + effects[1] * effects[1]);
        let k = if now > self.power.0 {
            self.pace.0
        } else {
            self.pace.1
        };
        let level = self.power.to(now, k).max(0.0).sqrt();
        1.0 - (1.0 - DUCK.2) * smoothstep(DUCK.0, DUCK.1, level)
    }
}

/// What the player asks of the output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Output {
    /// Master volume.
    pub volume: f32,
    /// How far the dynamic range is reduced, from 0 (full) to 1.
    pub reduced: f32,
    /// How far the two channels are folded into one, from 0 (stereo) to 1 (mono).
    pub mono: f32,
}

/// The master bus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Master {
    /// Offset removal, left and right.
    dc: [DcBlock; 2],
    /// Pole of the offset removal.
    pole: f32,
    /// Mean square level of the mix, for the reduced dynamic range.
    power: Lag,
    /// Gain the reduced dynamic range applies now.
    ride: Lag,
    /// Gain it moves toward, set a few hundred times per second.
    goal: f32,
    /// Coefficients of a rise and a fall of the level, and of the gain.
    pace: (f32, f32, f32),
    /// The limiter.
    guard: Limiter,
}

impl Master {
    /// Builds the master bus for a sample rate.
    pub fn new(rate: f32) -> Self {
        Self {
            dc: [DcBlock::default(); 2],
            pole: 1.0 - pole(DC_HZ, rate),
            power: Lag::default(),
            ride: Lag(1.0),
            goal: 1.0,
            pace: (
                glide(RIDE.0, rate),
                glide(RIDE.1, rate),
                glide(RIDE.2, rate),
            ),
            guard: Limiter::new(rate),
        }
    }

    /// Sets the gain of the reduced dynamic range from the level of the mix: a mix under the
    /// pivot is raised and one over it lowered, by the ratio, within its reach. Call it a few
    /// hundred times per second.
    pub fn tune(&mut self) {
        let level = self.power.0.max(1.0e-12).sqrt();
        let gain = (PIVOT / level).powf(1.0 - 1.0 / RATIO);
        self.goal = gain.clamp(REACH.1, REACH.0);
    }

    /// Finishes one frame of the mix and returns what goes to the speakers, always within
    /// full scale.
    pub fn run(&mut self, mix: [f32; 2], output: &Output) -> [f32; 2] {
        let now = 0.5 * (mix[0] * mix[0] + mix[1] * mix[1]);
        let k = if now > self.power.0 {
            self.pace.0
        } else {
            self.pace.1
        };
        self.power.to(now, k);
        let ride = 1.0 + output.reduced * (self.ride.to(self.goal, self.pace.2) - 1.0);
        let mid = 0.5 * (mix[0] + mix[1]);
        let gain = ride * output.volume;
        let frame = [0, 1].map(|side| {
            let folded = mix[side] + output.mono * (mid - mix[side]);
            self.dc[side].run(folded * gain, self.pole)
        });
        self.guard.run(frame)
    }
}
