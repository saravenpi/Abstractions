//! The cabin itself: the breath and the blades of the cooling fan and the hum of the inverter.
//! A cockpit with its avionics on is never silent, in an atmosphere or in a vacuum.

use super::law::band_gain;
use super::tune::{BED_TONES, FAN, STANDBY};
use crate::audio::dsp::filter::Svf;
use crate::audio::dsp::osc::{Phasor, sine};
use crate::audio::dsp::rand::Noise;

/// The cabin of one ship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cabin {
    /// Colour of the breath of the fan.
    fan: Svf,
    /// Gain of the breath.
    gain: f32,
    /// Phases of the blade tone, of its octave and of the inverter.
    tones: [Phasor; 3],
    /// Noise source.
    noise: Noise,
    /// Sample rate in hertz.
    rate: f32,
}

impl Cabin {
    /// Builds a cabin with its own noise `seed`.
    pub fn new(rate: f32, seed: u32) -> Self {
        let mut fan = Svf::default();
        fan.tune(FAN.0, FAN.1, rate);
        Self {
            fan,
            gain: FAN.2 * band_gain(FAN.0, FAN.1, rate),
            tones: [Phasor(0.0), Phasor(0.4), Phasor(0.7)],
            noise: Noise::new(seed),
            rate,
        }
    }

    /// Renders one sample of the bed, with the engines at `power` from 0 (off) to 1 (running).
    pub fn frame(&mut self, power: f32) -> f32 {
        let breath = self.fan.band(self.noise.tick()) * self.gain;
        let [blade, octave, mains] = &mut self.tones;
        let step = BED_TONES.0 / self.rate;
        let fan = sine(blade.tick(step)) + 0.4 * sine(octave.tick(2.003 * step));
        let hum = sine(mains.tick(BED_TONES.2 / self.rate));
        let bed = breath + fan * BED_TONES.1 + hum * BED_TONES.3;
        bed * (STANDBY + (1.0 - STANDBY) * power)
    }
}
