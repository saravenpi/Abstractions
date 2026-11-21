//! The air on the airframe, heard from the seat: the wind over the canopy, which follows the
//! dynamic pressure, the thuds of the buffet, the rattle of the frame in rough air, and the deep
//! roar of the shock layer of an entry through the hull, which follows the heating. As the air
//! thins on a climb each of them fades with the pressure or the heat that makes it, to silence.

use std::f32::consts::TAU;

use super::jet::swell;
use super::law::band_gain;
use super::tune::{BUFFET, FULL_PRESSURE, GUST, RATTLE, ROAR, WIND, WIND_HZ};
use crate::audio::dsp::filter::{Lag, Svf};
use crate::audio::dsp::rand::Noise;

/// What the airframe follows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flow {
    /// Speed through the air in metres per second.
    pub speed: f32,
    /// Dynamic pressure in pascals.
    pub pressure: f32,
    /// How hard the air shakes the hull, from 0 to 1.
    pub rough: f32,
    /// How hard the air heats the nose in an entry, from 0 to 1.
    pub roar: f32,
}

/// The airframe of one ship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Airframe {
    /// Colours: the wind, the thuds, the rattle and the roar.
    bands: [Svf; 4],
    /// Gains of the wind, the thuds, the knocks of the rattle and the roar.
    gains: [f32; 4],
    /// The random walk of the gusts.
    gust: Lag,
    /// Coefficient and gain of the gusts.
    swell: (f32, f32),
    /// Noise source.
    noise: Noise,
    /// Sample rate in hertz.
    rate: f32,
}

impl Airframe {
    /// Builds a silent airframe with its own noise `seed`.
    pub fn new(rate: f32, seed: u32) -> Self {
        let mut bands = [Svf::default(); 4];
        bands[1].tune(BUFFET.0, 0.9, rate);
        bands[2].tune(RATTLE.0, RATTLE.1, rate);
        bands[3].tune(ROAR.0, 0.7, rate);
        Self {
            bands,
            gains: [0.0; 4],
            gust: Lag::default(),
            swell: swell(GUST.0, rate),
            noise: Noise::new(seed),
            rate,
        }
    }

    /// Retunes the wind and sets the gains. The pressure fluctuations under a boundary layer
    /// follow the dynamic pressure; the level is its square root, which is how loud it is heard,
    /// and the pitch rises with the speed. The roar follows the heat of the entry.
    pub fn tune(&mut self, flow: &Flow) {
        let hz = (WIND_HZ.0 + WIND_HZ.1 * flow.speed).min(WIND_HZ.2);
        let loud = (flow.pressure / FULL_PRESSURE).clamp(0.0, 1.0).sqrt();
        self.bands[0].tune(hz, WIND.0, self.rate);
        self.gains = [
            loud * WIND.1 * band_gain(hz, WIND.0, self.rate),
            flow.rough * BUFFET.1 * band_gain(BUFFET.0, 1.0 / 0.9, self.rate),
            flow.rough * flow.rough * RATTLE.3 / RATTLE.2 * self.rate / (TAU * RATTLE.0),
            flow.roar.clamp(0.0, 1.0) * ROAR.1 * band_gain(ROAR.0, 1.0 / 0.7, self.rate),
        ];
    }

    /// Renders one sample of what the air does to the airframe.
    pub fn frame(&mut self) -> f32 {
        let white = self.noise.tick();
        let walk = self.gust.to(self.noise.tick(), self.swell.0) * self.swell.1;
        let gust = (1.0 + GUST.1 * walk).max(0.0);
        let knock = (self.noise.tick() - (1.0 - RATTLE.2)).max(0.0) * self.gains[2];
        let [wind, thud, rattle, roar] = &mut self.bands;
        let deep = thud.low(white) * self.gains[1] + roar.low(white) * self.gains[3];
        (wind.band(white) * self.gains[0] + deep) * gust + rattle.low(knock)
    }
}
