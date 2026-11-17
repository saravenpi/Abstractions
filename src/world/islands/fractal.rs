//! Fractal fields: the octave sums of the `noise` crate on the plane, rebuilt on the gradient noise in space.

use noise::{Fbm, NoiseFn, Perlin, RidgedMulti, Seedable};

use super::field::Field;
use super::solid::gradient;

impl Field for Fbm<Perlin> {
    fn flat(&self, p: [f64; 2]) -> f64 {
        self.get(p)
    }

    fn solid(&self, p: [f64; 3]) -> f64 {
        let mut q = p.map(|c| c * self.frequency);
        let (mut sum, mut total, mut gain) = (0.0, 0.0, self.persistence);
        for octave in 0..self.octaves as u32 {
            sum += gain * gradient(self.seed().wrapping_add(octave), q);
            total += gain;
            gain *= self.persistence;
            q = q.map(|c| c * self.lacunarity);
        }
        sum / total
    }
}

/// Returns the factor that brings a ridged sum of `octaves` octaves into 0 to 2, as the `noise` crate does.
fn ridge_scale(persistence: f64, attenuation: f64, octaves: usize) -> f64 {
    let (mut gain, mut span, mut signal, mut sum) = (1.0, 1.0, 1.0, 1.0);
    for _ in 0..octaves {
        gain *= persistence;
        span *= attenuation;
        signal = (signal / span).clamp(0.0, 1.0) * gain;
        sum += signal;
    }
    2.0 / sum
}

impl Field for RidgedMulti<Perlin> {
    fn flat(&self, p: [f64; 2]) -> f64 {
        self.get(p)
    }

    fn solid(&self, p: [f64; 3]) -> f64 {
        let mut q = p.map(|c| c * self.frequency);
        let (mut sum, mut weight, mut gain) = (0.0, 1.0, 1.0);
        for octave in 0..self.octaves as u32 {
            let ridge = 1.0 - gradient(self.seed().wrapping_add(octave), q).abs();
            let signal = ridge * ridge * weight;
            weight = (signal / self.attenuation).clamp(0.0, 1.0);
            sum += signal * gain;
            gain *= self.persistence;
            q = q.map(|c| c * self.lacunarity);
        }
        sum * ridge_scale(self.persistence, self.attenuation, self.octaves) - 1.0
    }
}
