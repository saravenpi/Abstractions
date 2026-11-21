//! The grains of a ground, as a stochastic model of particles: after a strike the ground holds
//! an energy that dies away, and each grain is a chance event whose odds follow that energy.
//! Each grain is an impulse of a random strength into one of two resonances, tuned apart in the
//! band of the ground and a little differently on every step, so no two steps draw the same
//! grains. This is Cook's PhISM: https://ccrma.stanford.edu/~prc/Particles.html
//! (Cook, "Physically Informed Sonic Modeling (PhISM): Synthesis of Percussive Sounds", Computer
//! Music Journal 21:3, 1997).

use std::f32::consts::TAU;

use super::material::Material;
use crate::audio::dsp::filter::Svf;
use crate::audio::dsp::rand::Noise;

/// The grains of one step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grains {
    /// The two resonances.
    bands: [Svf; 2],
    /// Gain that makes an impulse into each resonance peak at its strength.
    scale: [f32; 2],
    /// Energy left in the ground, from 0 to 1.
    energy: f32,
    /// Factor the energy dies by each sample.
    decay: f32,
    /// Odds of a grain per sample at full energy.
    odds: f32,
    /// Peak of the strongest grain.
    peak: f32,
}

impl Default for Grains {
    /// Grains at rest that never break.
    fn default() -> Self {
        Self {
            bands: [Svf::default(); 2],
            scale: [0.0; 2],
            energy: 0.0,
            decay: 0.0,
            odds: 0.0,
            peak: 0.0,
        }
    }
}

impl Grains {
    /// Tunes the grains of `stuff` for one step: `spread` from 0 to 1 places the two resonances
    /// in its band, `pitch` turns them, `rate` scales how many grains break and `peak` how
    /// strong they are, at a sample rate `sample`.
    pub fn tune(
        &mut self,
        stuff: &Material,
        (spread, pitch): (f32, f32),
        (rate, peak): (f32, f32),
        sample: f32,
    ) {
        let (low, high, q, per_second, seconds, level) = stuff.grain;
        let span = high / low;
        for (side, at) in [0.25 * spread, 0.5 + 0.5 * spread].into_iter().enumerate() {
            let hz = (low * span.powf(at) * pitch).min(0.45 * sample);
            self.bands[side] = Svf::default();
            self.bands[side].tune(hz, q, sample);
            self.scale[side] = q * sample / (TAU * hz);
        }
        self.energy = 0.0;
        self.decay = (-1.0 / (seconds * sample)).exp();
        self.odds = (per_second * rate / sample).clamp(0.0, 0.5);
        self.peak = level * peak;
    }

    /// Makes the motion of the grains last `seconds` from now on.
    pub fn hold(&mut self, seconds: f32, sample: f32) {
        self.decay = (-1.0 / (seconds * sample)).exp();
    }

    /// Lands a `share` of a step on the ground: its energy rises by it, up to 1.
    pub fn kick(&mut self, share: f32) {
        self.energy = (self.energy + share).min(1.0);
    }

    /// Renders one sample from `noise`, and returns it with the strength of the grain that
    /// broke on it, 0 when none did.
    pub fn frame(&mut self, noise: &mut Noise) -> (f32, f32) {
        self.energy *= self.decay;
        let draw = 0.5 + 0.5 * noise.tick();
        let (strength, low) = match draw < self.odds * self.energy {
            true => {
                let size = 0.5 + 0.5 * noise.tick();
                (self.peak * size * size, noise.tick() < 0.0)
            }
            false => (0.0, false),
        };
        let hits = if low {
            [strength, 0.0]
        } else {
            [0.0, strength]
        };
        let [first, second] = &mut self.bands;
        let out = first.band(hits[0] * self.scale[0]) + second.band(hits[1] * self.scale[1]);
        (out, strength)
    }
}
