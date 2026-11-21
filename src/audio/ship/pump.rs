//! The two turbopumps of the main engines: the one tonal part of the ship. Their speed follows
//! what is asked of them with the inertia of a rotor, so the whine lags the lever, climbs on a
//! start and winds down for seconds after a shutdown. Two rotors never turn alike, so they beat.

use super::jet::swell;
use super::tune::{ORDERS, PUMP_LAG, PUMP_LEVEL, SHAFT_HZ, TWIN, WANDER};
use crate::audio::dsp::filter::{Lag, glide};
use crate::audio::dsp::osc::{Phasor, sine};
use crate::audio::dsp::rand::Noise;

/// Speed under which a pump that is asked to stop is stopped.
const STOPPED: f32 = 1.0e-4;

/// The pumps of one ship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pump {
    /// Speed of the rotors as a share of their full speed.
    speed: f32,
    /// Phases of each pump: three shaft orders and the blade whine.
    shafts: [[Phasor; 4]; 2],
    /// The slow wander of the speed.
    drift: Lag,
    /// Coefficient of the wander and its depth as a share of the speed.
    wander: (f32, f32),
    /// Coefficients of a rise and of a fall of the speed.
    inertia: (f32, f32),
    /// Noise source.
    noise: Noise,
    /// Sample rate in hertz.
    rate: f32,
}

impl Pump {
    /// Builds stopped pumps with their own noise `seed`.
    pub fn new(rate: f32, seed: u32) -> Self {
        let (k, gain) = swell(WANDER.0, rate);
        let start = [Phasor(0.0), Phasor(0.23), Phasor(0.61), Phasor(0.4)];
        Self {
            speed: 0.0,
            shafts: [start, start.map(|phase| Phasor((phase.0 + 0.37).fract()))],
            drift: Lag::default(),
            wander: (k, gain * WANDER.1),
            inertia: (glide(PUMP_LAG.0, rate), glide(PUMP_LAG.1, rate)),
            noise: Noise::new(seed),
            rate,
        }
    }

    /// Returns the speed of the rotors as a share of their full speed.
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Moves the rotors toward the speed `asked`, from 0 to 1, and renders one sample: the
    /// whine of the blades in the air, and the shaft orders through the airframe. Both grow as
    /// the square of the speed, as the pressure a pump makes does.
    pub fn frame(&mut self, asked: f32) -> (f32, f32) {
        let k = if asked > self.speed {
            self.inertia.0
        } else {
            self.inertia.1
        };
        self.speed += k * (asked - self.speed);
        if asked == 0.0 && self.speed < STOPPED {
            self.speed = 0.0;
        }
        let drift = 1.0 + self.wander.1 * self.drift.to(self.noise.tick(), self.wander.0);
        let step = self.speed * SHAFT_HZ * drift / self.rate;
        let (mut whine, mut body) = (0.0, 0.0);
        for (shaft, twin) in self.shafts.iter_mut().zip([1.0, TWIN]) {
            let [first, second, third, blades] = shaft;
            let turn = step * twin;
            body += sine(first.tick(turn))
                + ORDERS.0 * sine(second.tick(2.0 * turn))
                + ORDERS.1 * sine(third.tick(3.0 * turn));
            whine += sine(blades.tick(ORDERS.2 * turn));
        }
        let power = self.speed * self.speed;
        (whine * power * PUMP_LEVEL.1, body * power * PUMP_LEVEL.0)
    }
}
