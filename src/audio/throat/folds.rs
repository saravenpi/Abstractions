//! The vocal folds: a train of glottal pulses whose every period is a little different.
//!
//! Each period is one Rosenberg pulse of air, and what the ear hears is its slope, the way the lips
//! radiate it. Jitter moves the length of each period and shimmer its height, as in every real voice.
//! As the voice roughens, every other pulse grows weaker and longer, which doubles the period and puts
//! subharmonics under the note, and then a logistic map in its chaotic range takes over the periods:
//! deterministic chaos, the broadband roar of a strained throat. A biphonating voice adds a second
//! tone at a frequency of its own, as when the two folds part ways.

use super::tune::{BIPHONIC, CHAOS_R, CHAOTIC, OPEN, SUBHARMONIC};
use crate::audio::dsp::osc::{Phasor, sine};
use crate::audio::dsp::rand::Noise;
use crate::render::smoothstep;

/// What the folds are asked for at one sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Push {
    /// Pitch in hertz.
    pub hz: f32,
    /// Roughness, 0 clean to 1 chaotic.
    pub rough: f32,
    /// Share of breath, 0 to 1.
    pub breath: f32,
    /// Ratio of the second tone of a biphonating voice to the first, 0 for none.
    pub second: f32,
}

/// The state of the folds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Folds {
    /// Phase in the current period, 0 to 1.
    phase: f32,
    /// Height of the current pulse.
    height: f32,
    /// Pitch factor of the current period.
    stretch: f32,
    /// Flow of air at the last sample, to take its slope.
    flow: f32,
    /// True on every other period.
    odd: bool,
    /// State of the logistic map, inside 0 to 1.
    chaos: f32,
    /// Phase of the second tone.
    second: Phasor,
}

/// Returns the flow of one Rosenberg pulse at `phase`: a slow opening, a fast closing, then shut.
fn pulse(phase: f32) -> f32 {
    let (open, close) = OPEN;
    if phase < open {
        0.5 * (1.0 - (std::f32::consts::PI * phase / open).cos())
    } else if phase < open + close {
        (std::f32::consts::FRAC_PI_2 * (phase - open) / close).cos()
    } else {
        0.0
    }
}

impl Folds {
    /// Returns folds at rest whose chaos starts from `seed`, 0 to 1.
    pub fn new(seed: f32) -> Self {
        Self {
            phase: 0.0,
            height: 1.0,
            stretch: 1.0,
            flow: 0.0,
            odd: false,
            chaos: 0.1 + 0.8 * seed.clamp(0.0, 1.0),
            second: Phasor(0.0),
        }
    }

    /// Draws the length and height of the next period for a voice of roughness `rough`.
    fn cycle(&mut self, rough: f32, noise: &mut Noise) {
        let jitter = 0.008 + 0.05 * rough;
        let shimmer = 0.04 + 0.25 * rough;
        let sub = smoothstep(SUBHARMONIC.0, SUBHARMONIC.1, rough);
        let chaos = smoothstep(CHAOTIC.0, CHAOTIC.1, rough);
        self.chaos = (CHAOS_R * self.chaos * (1.0 - self.chaos)).clamp(0.001, 0.999);
        self.odd = !self.odd;
        let swing = self.chaos - 0.5;
        let (weak, long) = match self.odd {
            true => (1.0 - 0.6 * sub, 1.0 + 0.08 * sub),
            false => (1.0, 1.0 - 0.08 * sub),
        };
        self.height = weak * (1.0 + shimmer * noise.tick() + 0.8 * chaos * swing);
        let period = long * (1.0 + jitter * noise.tick() + 0.5 * chaos * swing);
        self.stretch = 1.0 / period.max(0.3);
    }

    /// Returns one sample of the voice for `push` at the sample `rate`, about -1 to 1.
    pub fn tick(&mut self, push: &Push, rate: f32, noise: &mut Noise) -> f32 {
        let hz = push.hz.max(1.0);
        self.phase += hz * self.stretch / rate;
        if self.phase >= 1.0 {
            self.phase -= self.phase.floor();
            self.cycle(push.rough, noise);
        }
        let flow = self.height * pulse(self.phase);
        let slope = (flow - self.flow) * rate * OPEN.1 / (std::f32::consts::FRAC_PI_2 * hz);
        self.flow = flow;
        let breath = push.breath * noise.tick() * (0.35 + flow);
        let two = match push.second > 0.0 {
            true => BIPHONIC * sine(self.second.tick(hz * push.second / rate)),
            false => 0.0,
        };
        slope * (1.0 - 0.5 * push.breath) + breath + two
    }
}
