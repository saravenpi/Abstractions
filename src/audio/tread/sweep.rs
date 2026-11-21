//! The wet part of a step: noise through a band whose centre sweeps as the foot goes into mud
//! or water, and back the other way as it pulls out. Its level swells and dies as a sine, so it
//! starts and ends on silence.

use std::f32::consts::PI;

use crate::audio::dsp::filter::Svf;
use crate::audio::dsp::rand::Noise;
use crate::audio::ship::law::band_gain;

/// Samples between two retunings of the band.
const RETUNE: u32 = 32;

/// One wet sweep.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sweep {
    /// The band.
    band: Svf,
    /// Samples into the sweep.
    at: u32,
    /// Samples the sweep lasts, 0 when there is none.
    span: u32,
    /// The centre it starts from and the one it ends on.
    ends: (f32, f32),
    /// Its level, and the gain of the band now that brings noise to a level of 1.
    level: (f32, f32),
    /// The resonance of the band.
    q: f32,
}

impl Sweep {
    /// Starts a sweep from `from` to `to` hertz over `seconds` at a peak `level`, through a band of
    /// resonance `q`.
    pub fn start(
        &mut self,
        (from, to): (f32, f32),
        (seconds, q): (f32, f32),
        level: f32,
        rate: f32,
    ) {
        *self = Self {
            ends: (from, to),
            span: (seconds * rate).max(1.0) as u32,
            level: (level, 0.0),
            q,
            ..Self::default()
        };
        self.retune(rate);
    }

    /// Sets the centre of the band for where the sweep is.
    fn retune(&mut self, rate: f32) {
        let (from, to) = self.ends;
        let hz = from * (to / from).powf(self.at as f32 / self.span as f32);
        self.band.tune(hz, self.q, rate);
        self.level.1 = band_gain(hz, self.q, rate);
    }

    /// Renders one sample from `noise`.
    pub fn frame(&mut self, noise: &mut Noise, rate: f32) -> f32 {
        if self.at >= self.span {
            return 0.0;
        }
        if self.at.is_multiple_of(RETUNE) {
            self.retune(rate);
        }
        let swell = (PI * self.at as f32 / self.span as f32).sin();
        self.at += 1;
        self.band.band(noise.tick()) * self.level.1 * self.level.0 * swell * swell
    }
}
