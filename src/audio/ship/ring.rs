//! Things that ring when struck: the thump of a lock, the clack of a latch, and the ticks of
//! hot nozzles as they cool.

use std::f32::consts::TAU;

use super::tune::EMBER;
use crate::audio::dsp::filter::Svf;
use crate::audio::dsp::rand::Noise;

/// Decay times a ringer is rendered for after a strike: it has lost 100 decibels by then.
const TAILS: f32 = 12.0;

/// A resonator struck by impulses. Its low-pass output starts on zero, so a strike never clicks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ringer {
    /// The resonator.
    filter: Svf,
    /// The resonator at rest, to go back to once a ring has died.
    clean: Svf,
    /// Impulse waiting for the next sample.
    hit: f32,
    /// Impulse that makes a ring of peak 1.
    scale: f32,
    /// Samples left to render of the current ring.
    left: u32,
    /// Samples a ring is rendered for.
    span: u32,
}

impl Ringer {
    /// Builds a ringer at rest that rings at `hz` with a resonance `q`.
    pub fn new(hz: f32, q: f32, rate: f32) -> Self {
        let mut ringer = Self {
            filter: Svf::default(),
            clean: Svf::default(),
            hit: 0.0,
            scale: 1.0,
            left: 0,
            span: 1,
        };
        ringer.tune(hz, q, rate);
        ringer
    }

    /// Sets the tone to `hz` and the resonance to `q`, and stops any ring.
    pub fn tune(&mut self, hz: f32, q: f32, rate: f32) {
        self.clean = Svf::default();
        self.clean.tune(hz, q, rate);
        self.filter = self.clean;
        self.scale = rate / (TAU * hz);
        self.span = (TAILS * 2.0 * q / (TAU * hz) * rate) as u32 + 1;
        self.left = 0;
    }

    /// Strikes the ringer: the ring peaks near `level`.
    pub fn strike(&mut self, level: f32) {
        self.hit += level * self.scale;
    }

    /// Renders one sample.
    pub fn frame(&mut self) -> f32 {
        let hit = std::mem::take(&mut self.hit);
        if hit != 0.0 {
            self.left = self.span;
        }
        if self.left == 0 {
            return 0.0;
        }
        self.left -= 1;
        let out = self.filter.low(hit);
        if self.left == 0 {
            self.filter = self.clean;
        }
        out
    }
}

/// The ticks of metal that cools: single high pings at random moments, each at its own pitch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ticks {
    /// The ping.
    ring: Ringer,
    /// Source of the moments and the pitches.
    noise: Noise,
    /// Sample rate in hertz.
    rate: f32,
}

impl Ticks {
    /// Builds silent ticks with their own noise `seed`.
    pub fn new(rate: f32, seed: u32) -> Self {
        Self {
            ring: Ringer::new(EMBER.1, EMBER.3, rate),
            noise: Noise::new(seed),
            rate,
        }
    }

    /// Renders one sample at a `pace` in ticks per second.
    pub fn frame(&mut self, pace: f32) -> f32 {
        if pace > 0.0 && self.noise.tick() > 1.0 - 2.0 * pace / self.rate {
            let pitch = 0.5 + 0.5 * self.noise.tick();
            self.ring
                .tune(EMBER.1 + EMBER.2 * pitch, EMBER.3, self.rate);
            self.ring.strike(EMBER.4 * (0.4 + 0.6 * pitch));
        }
        self.ring.frame()
    }
}
