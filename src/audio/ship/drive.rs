//! The warp drive, heard through the hull: three pairs of tones that rise with the spool, each
//! pair beating faster as it charges, and under the cruise a bed of narrow noise that wanders.

use super::jet::swell;
use super::law::band_gain;
use super::tune::{BED, CRUISE, WARP_BEAT, WARP_HZ, WARP_LEVEL, WARP_PARTIALS};
use crate::audio::dsp::filter::{Lag, Svf};
use crate::audio::dsp::osc::{Phasor, sine};
use crate::audio::dsp::rand::Noise;
use crate::render::smoothstep;

/// Rate in hertz of the wander of the bed of the cruise.
const WANDER_HZ: f32 = 0.2;

/// The warp drive of one ship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drive {
    /// Phases of the two tones of each partial.
    tones: [[Phasor; 2]; 3],
    /// Colour of the bed of the cruise.
    bed: Svf,
    /// Gain of the bed.
    gain: f32,
    /// The random walk of the centre of the bed.
    drift: Lag,
    /// Coefficient and gain of that walk, per call of [`Drive::tune`].
    swell: (f32, f32),
    /// Noise source.
    noise: Noise,
    /// Sample rate in hertz.
    rate: f32,
}

impl Drive {
    /// Builds a drive at rest with its own noise `seed`. `dt` is the time between two calls of
    /// [`Drive::tune`], in seconds.
    pub fn new(rate: f32, seed: u32, dt: f32) -> Self {
        Self {
            tones: [
                [Phasor(0.0), Phasor(0.5)],
                [Phasor(0.2), Phasor(0.7)],
                [Phasor(0.4), Phasor(0.9)],
            ],
            bed: Svf::default(),
            gain: 0.0,
            drift: Lag::default(),
            swell: swell(WANDER_HZ, 1.0 / dt),
            noise: Noise::new(seed),
            rate,
        }
    }

    /// Lets the bed of the cruise wander. Call it a few hundred times per second.
    pub fn tune(&mut self) {
        let walk = self.drift.to(self.noise.tick(), self.swell.0) * self.swell.1;
        let hz = BED.0 * (1.0 + BED.2 * walk.clamp(-2.0, 2.0));
        self.bed.tune(hz, BED.1, self.rate);
        self.gain = BED.3 * band_gain(hz, BED.1, self.rate);
    }

    /// Renders one sample for a drive `warp` of the way up, from 0 to 1.
    pub fn frame(&mut self, warp: f32) -> f32 {
        if warp <= 0.0 {
            return 0.0;
        }
        let step = (WARP_HZ.0 + WARP_HZ.1 * warp * warp) / self.rate;
        let beat = (WARP_BEAT.0 + WARP_BEAT.1 * warp) / self.rate;
        let mut sum = 0.0;
        for (pair, (ratio, level)) in self.tones.iter_mut().zip(WARP_PARTIALS) {
            let [low, high] = pair;
            sum += level * (sine(low.tick(step * ratio)) + sine(high.tick(step * ratio + beat)));
        }
        let cruise = smoothstep(CRUISE.0, CRUISE.1, warp);
        sum * warp * WARP_LEVEL + self.bed.band(self.noise.tick()) * self.gain * cruise
    }
}
