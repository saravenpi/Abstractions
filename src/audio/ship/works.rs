//! The mechanisms of a ship: the gear motor and the thump of its locks, the canopy servo and
//! its seal, the hiss of the attitude jets of the stabiliser, the ticks of nozzles that cool,
//! the chambers lighting, and the gear taking a landing and settling.

use super::feed::{Ch, HullSound};
use super::law::band_gain;
use super::motor::Motor;
use super::ring::{Ringer, Ticks};
use super::tune::{EMBER, HISS, IGNITE, KNOCKS, LANDING, LOCK, MOTOR_HZ, PUFF, SEAL, SHARES};
use crate::audio::dsp::filter::{Lag, Svf, glide};
use crate::audio::dsp::rand::Noise;

/// The mechanisms of one ship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Works {
    /// The motor of the landing gear.
    gear: Motor,
    /// The servo of the canopy.
    canopy: Motor,
    /// The thump of a lock, a landing or the chambers lighting.
    thump: Ringer,
    /// The clack of a latch.
    clack: Ringer,
    /// The ticks of the nozzles as they cool.
    embers: Ticks,
    /// Colour of the hiss of the attitude jets and of a seal.
    hiss: Svf,
    /// Level of the puff of a seal or a strut, which dies away.
    puff: f32,
    /// The puff as it is heard.
    sigh: Lag,
    /// Share of a puff left after one reading, and the coefficient the sigh follows it with.
    fade: (f32, f32),
    /// Gain of the attitude jets and of a full puff.
    spray: (f32, f32),
    /// Ticks per second of the nozzles now.
    pace: f32,
    /// Samples until the gear gives back a landing, and the peak of that thump.
    later: (u32, f32),
    /// Noise source.
    noise: Noise,
    /// Sample rate in hertz.
    rate: f32,
}

impl Works {
    /// Builds silent mechanisms with their own noise `seed`. `dt` is the time between two
    /// calls of [`Works::tune`], in seconds.
    pub fn new(rate: f32, seed: u32, dt: f32) -> Self {
        let mut hiss = Svf::default();
        hiss.tune(HISS.0, HISS.1, rate);
        Self {
            gear: Motor::new(rate, seed ^ 0x6EA2, MOTOR_HZ.0),
            canopy: Motor::new(rate, seed ^ 0xCA90, MOTOR_HZ.1),
            thump: Ringer::new(KNOCKS[0].0, KNOCKS[0].1, rate),
            clack: Ringer::new(KNOCKS[1].0, KNOCKS[1].1, rate),
            embers: Ticks::new(rate, seed ^ 0xE3BE),
            hiss,
            puff: 0.0,
            sigh: Lag::default(),
            fade: ((-dt / PUFF.0).exp(), glide(PUFF.1, rate)),
            spray: (0.0, HISS.3 * band_gain(HISS.0, HISS.1, rate)),
            pace: 0.0,
            later: (0, 0.0),
            noise: Noise::new(seed),
            rate,
        }
    }

    /// Puts the gear and the canopy where `hull` has them, without any travel.
    pub fn rest(&mut self, hull: &HullSound) {
        self.gear.rest(hull.at(Ch::Gear));
        self.canopy.rest(hull.at(Ch::Canopy));
    }

    /// Follows the mechanisms of `hull`, whose chambers are `lit` from 0 to 1, `dt` seconds
    /// after the last call: a gear that arrives locks with a thump, a canopy that shuts
    /// latches and seals, and nozzles that are hot with the engines off tick.
    pub fn tune(&mut self, hull: &HullSound, lit: f32, dt: f32) {
        if let Some(down) = self.gear.follow(hull.at(Ch::Gear), dt) {
            self.thump.strike(if down { LOCK.0 } else { LOCK.1 });
            self.clack.strike(LOCK.2);
        }
        if self.canopy.follow(hull.at(Ch::Canopy), dt) == Some(false) {
            self.thump.strike(SEAL.0);
            self.clack.strike(SEAL.1);
            self.puff = self.puff.max(SEAL.2);
        }
        self.puff *= self.fade.0;
        self.spray.0 = hull.at(Ch::Rcs) * HISS.2 * band_gain(HISS.0, HISS.1, self.rate);
        self.pace = EMBER.0 * hull.at(Ch::Glow) * (1.0 - lit);
    }

    /// Lights the chambers: a thump and a puff.
    pub fn light(&mut self) {
        self.thump.strike(IGNITE.0);
        self.puff = self.puff.max(IGNITE.1);
    }

    /// Takes a landing of `strength` from 0 (feather) to 1 (hard): the gear thumps, gives a
    /// part of it back a moment later, and the struts sigh as they settle.
    pub fn land(&mut self, strength: f32) {
        let peak = LANDING.0 + (LANDING.1 - LANDING.0) * strength.clamp(0.0, 1.0);
        self.thump.strike(peak);
        self.clack.strike(LOCK.2 * strength.clamp(0.0, 1.0));
        self.later = ((LANDING.3 * self.rate) as u32, peak * LANDING.2);
        self.puff = self.puff.max(LANDING.4);
    }

    /// Renders one sample: what the mechanisms radiate into the air, and what they send
    /// through the airframe.
    pub fn frame(&mut self) -> (f32, f32) {
        if self.later.0 > 0 {
            self.later.0 -= 1;
            if self.later.0 == 0 {
                self.thump.strike(self.later.1);
            }
        }
        let hiss = self.hiss.band(self.noise.tick());
        let sigh = self.sigh.to(self.puff, self.fade.1) * self.spray.1;
        let solid =
            self.thump.frame() + self.clack.frame() + self.gear.frame() + self.canopy.frame();
        let jets = hiss * self.spray.0;
        let air = jets + hiss * sigh + self.embers.frame(self.pace) + SHARES.0 * solid;
        (air, solid + SHARES.1 * jets + hiss * sigh)
    }
}
