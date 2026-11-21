//! A file and a scraper: each tooth the scraper strikes is a kick that rings a resonance of the shell.
//!
//! A cricket strikes its teeth at the rate its wing rings, so the kicks feed one pure tone; a beetle
//! scrapes slower teeth over a dull shell, a buzz; crystals knock facets now and then, ticks that ring.

use crate::audio::dsp::filter::Svf;
use crate::audio::dsp::rand::Noise;

/// Share of noise under the kicks: the scrape between two teeth.
const SCRAPE: f32 = 0.03;

/// Share by which a kick varies in strength.
const KICK_SPREAD: f32 = 0.3;

/// The state of a file.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Pulses {
    /// Phase toward the next tooth, 0 to 1.
    phase: f32,
    /// The resonance the teeth ring.
    ring: Svf,
}

impl Pulses {
    /// Starts a new file at `hz` with resonance `q`, the scraper on a tooth so the first sample strikes.
    pub fn reset(&mut self, hz: f32, q: f32, rate: f32) {
        *self = Self {
            phase: 1.0,
            ..Self::default()
        };
        self.tune(hz, q, rate);
    }

    /// Tunes the resonance to `hz` with resonance `q`, keeping what rings.
    pub fn tune(&mut self, hz: f32, q: f32, rate: f32) {
        self.ring.tune(hz, q, rate);
    }

    /// Returns one sample of teeth struck `teeth` times a second.
    pub fn tick(&mut self, teeth: f32, rate: f32, noise: &mut Noise) -> f32 {
        self.phase += teeth / rate;
        let kick = match self.phase >= 1.0 {
            true => {
                self.phase -= self.phase.floor();
                1.0 + KICK_SPREAD * noise.tick()
            }
            false => 0.0,
        };
        self.ring.band(kick + SCRAPE * noise.tick())
    }
}
