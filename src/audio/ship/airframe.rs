//! The air on the airframe: the wind over the canopy, the thuds of the buffet, the rattle of the
//! frame in rough air, and the deep roar of the shock layer of an entry, which follows the heating.
//! As the air thins on a climb each of them fades with the pressure or the heat that makes it, to
//! silence.
//!
//! The wind follows the real flow, not the airspeed alone. Its level in decibels follows the log of
//! the dynamic pressure `1/2 rho v^2`, and its centre rises with the speed. A high angle of attack
//! or a sideslip tears the flow off the canopy and the wings: the band widens and grows, and the
//! buffet rumbles under it. A hard roll or yaw whooshes once, and a hard pull of the stick
//! buffets. Gusts come only with rough air. Every level is a continuous modulation of the bed that
//! eases over a tenth of a second, so the air follows the stick, never clicks, and steady flight in
//! calm air is steady.

use std::f32::consts::TAU;

use super::air_tune::{
    BUFFET, CHASE_WIND, FLOW_EASE, FULL_PRESSURE, GUST, PULL, RATTLE, ROAR, SLIP_LEAN, SLOW, TORN,
    WHIRL, WIND, WIND_HZ,
};
use super::feed::{Ch, HullSound};
use super::jet::swell;
use super::law::band_gain;
use super::whirl::Whirl;
use crate::audio::dsp::filter::glide;
use crate::audio::dsp::filter::{Lag, Svf};
use crate::audio::dsp::rand::Noise;
use crate::render::smoothstep;

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
    /// Angle of attack in radians.
    pub attack: f32,
    /// Sideslip in radians, positive from the right.
    pub slip: f32,
    /// Rate of roll and yaw in radians per second.
    pub turn: f32,
    /// Load of the turn in g.
    pub load: f32,
}

/// Returns the left and right gains of the wind a chase view hears in a sideslip of `slip`
/// radians, positive from the right: louder on the side facing the wind, the same both sides in
/// straight flight.
pub fn balance(slip: f32) -> [f32; 2] {
    let lean = (slip.sin() * SLIP_LEAN).clamp(-0.9, 0.9);
    [(1.0 - lean).sqrt(), (1.0 + lean).sqrt()]
}

impl Flow {
    /// Returns the flow over the airframe of `hull` at a dynamic `pressure` in pascals, with the
    /// air `rough` as it is.
    pub fn of(hull: &HullSound, pressure: f32, rough: f32) -> Self {
        Self {
            speed: hull.at(Ch::Speed),
            pressure,
            rough,
            roar: hull.at(Ch::Roar),
            attack: hull.at(Ch::Attack),
            slip: hull.at(Ch::Slip),
            turn: hull.at(Ch::Turn),
            load: hull.at(Ch::Load),
        }
    }

    /// Still air at rest: nothing moves over the airframe.
    #[cfg(test)]
    pub const STILL: Flow = Flow {
        speed: 0.0,
        pressure: 0.0,
        rough: 0.0,
        roar: 0.0,
        attack: 0.0,
        slip: 0.0,
        turn: 0.0,
        load: 1.0,
    };
}

/// The airframe of one ship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Airframe {
    /// Colours: the wind, the thuds, the rattle and the roar.
    bands: [Svf; 4],
    /// Gains the wind, the thuds, the knocks of the rattle and the roar move toward, and how rough
    /// the air is for the gusts.
    gains: [f32; 5],
    /// The same, eased, and the coefficient of the easing.
    eased: ([Lag; 5], f32),
    /// The whoosh of a hard roll or yaw.
    whirl: Whirl,
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
            gains: [0.0; 5],
            eased: ([Lag::default(); 5], glide(FLOW_EASE, rate)),
            whirl: Whirl::default(),
            gust: Lag::default(),
            swell: swell(GUST.0, rate),
            noise: Noise::new(seed),
            rate,
        }
    }

    /// Retunes the wind and sets the gains. The pressure fluctuations under a boundary layer
    /// follow the dynamic pressure, and the level is its square root, 10 dB for each tenfold
    /// pressure; the pitch rises with the speed. A torn flow widens the band and adds to its level
    /// and to the buffet, the onset of a hard roll or yaw adds one whoosh, a hard pull buffets. The
    /// roar follows the heat of the entry. Call it every `dt` seconds.
    pub fn tune(&mut self, flow: &Flow, dt: f32) {
        let hz = (WIND_HZ.0 + WIND_HZ.1 * flow.speed).min(WIND_HZ.2);
        let loud = (flow.pressure / FULL_PRESSURE).clamp(0.0, 1.0).sqrt();
        let moving = smoothstep(SLOW.0, SLOW.1, flow.speed);
        let torn = smoothstep(TORN.0, TORN.1, flow.attack.max(flow.slip.abs())) * moving;
        self.whirl.tune(flow.turn, dt);
        let pulled = smoothstep(PULL.0, PULL.1, flow.load);
        let q = WIND.0 * (1.0 - TORN.2 * torn);
        let rough = flow.rough.max(loud * (TORN.3 * torn).max(pulled));
        self.bands[0].tune(hz, q, self.rate);
        let swell = 1.0 + TORN.4 * torn + WHIRL.4 * self.whirl.level();
        self.gains = [
            loud * WIND.1 * swell * band_gain(hz, q, self.rate),
            rough * BUFFET.1 * band_gain(BUFFET.0, 1.0 / 0.9, self.rate),
            rough * rough * RATTLE.3 / RATTLE.2 * self.rate / (TAU * RATTLE.0),
            flow.roar.clamp(0.0, 1.0) * ROAR.1 * band_gain(ROAR.0, 1.0 / 0.7, self.rate),
            rough,
        ];
    }

    /// Puts every eased level where it is heading at once, to compare an eased change with one
    /// that is not.
    #[cfg(test)]
    pub fn snap(&mut self) {
        self.eased.0 = self.gains.map(Lag);
    }

    /// Returns one sample of the wind on the airframe for ears `inside` the cabin, from 0 to 1, and
    /// what a chase view `riding` along hears of it in each ear in a sideslip of `slip` radians.
    /// Nothing is rendered when neither hears it.
    pub fn heard(&mut self, inside: f32, riding: f32, slip: f32) -> (f32, [f32; 2]) {
        if inside <= 0.0 && riding <= 0.0 {
            return (0.0, [0.0; 2]);
        }
        let wind = self.frame();
        let rush = balance(slip).map(|gain| gain * wind * riding * CHASE_WIND);
        (wind, rush)
    }

    /// Renders one sample of what the air does to the airframe.
    pub fn frame(&mut self) -> f32 {
        let (eased, k) = &mut self.eased;
        let gains: [f32; 5] = std::array::from_fn(|i| eased[i].to(self.gains[i], *k));
        let white = self.noise.tick();
        let walk = self.gust.to(self.noise.tick(), self.swell.0) * self.swell.1;
        let gust = (1.0 + GUST.1 * gains[4] * walk).max(0.0);
        let knock = (self.noise.tick() - (1.0 - RATTLE.2)).max(0.0) * gains[2];
        let [wind, thud, rattle, roar] = &mut self.bands;
        let deep = thud.low(white) * gains[1] + roar.low(white) * gains[3];
        (wind.band(white) * gains[0] + deep) * gust + rattle.low(knock)
    }
}
