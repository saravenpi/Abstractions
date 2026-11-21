//! A motor that moves a part of the ship: it whirs while the part travels, its pitch climbing
//! as it comes up to speed and sinking as it stops, and it says when the part has arrived.

use super::law::band_gain;
use super::tune::{HISS, MOTOR, MOTOR_PACE, MOTOR_SPIN};
use crate::audio::dsp::filter::{Lag, Svf, glide};
use crate::audio::dsp::osc::{Phasor, sine};
use crate::audio::dsp::rand::Noise;

/// The motor of one mechanism.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Motor {
    /// Where the part was at the last reading, from 0 to 1.
    last: f32,
    /// Whether the part travels.
    moving: bool,
    /// How far the motor is up to speed, from 0 to 1.
    run: Lag,
    /// Coefficient of the spin up and down.
    spin: f32,
    /// Phases of the tone and of its third harmonic.
    tone: [Phasor; 2],
    /// Colour of the hiss of the gears or the fluid.
    hiss: Svf,
    /// Gain of the hiss.
    gain: f32,
    /// Noise source.
    noise: Noise,
    /// Turns per sample at speed.
    step: f32,
}

impl Motor {
    /// Builds a stopped motor that hums at `hz` at speed, with its own noise `seed`.
    pub fn new(rate: f32, seed: u32, hz: f32) -> Self {
        let mut hiss = Svf::default();
        hiss.tune(HISS.0, HISS.1, rate);
        Self {
            last: 0.0,
            moving: false,
            run: Lag::default(),
            spin: glide(MOTOR_SPIN, rate),
            tone: [Phasor(0.0), Phasor(0.3)],
            hiss,
            gain: MOTOR.2 * band_gain(HISS.0, HISS.1, rate),
            noise: Noise::new(seed),
            step: hz / rate,
        }
    }

    /// Puts the part at `position` without any travel.
    pub fn rest(&mut self, position: f32) {
        self.last = position;
        self.moving = false;
    }

    /// Reads where the part is, `dt` seconds after the last reading. Returns the end it has
    /// just stopped at, if it has: true for the far end, at 1, false for the near end, at 0.
    /// A part at rest at an end starts to travel once it has left that end.
    pub fn follow(&mut self, position: f32, dt: f32) -> Option<bool> {
        let pace = (position - self.last).abs() / dt;
        let was = self.moving;
        let home = position <= 0.01 || position >= 0.99;
        self.last = position;
        self.moving = pace > MOTOR_PACE && (was || !home);
        (was && !self.moving && home).then_some(position > 0.5)
    }

    /// Renders one sample.
    pub fn frame(&mut self) -> f32 {
        let run = self.run.settle(f32::from(u8::from(self.moving)), self.spin);
        if run == 0.0 {
            return 0.0;
        }
        let step = self.step * (MOTOR.0 + (1.0 - MOTOR.0) * run);
        let [first, third] = &mut self.tone;
        let hum = sine(first.tick(step)) + MOTOR.1 * sine(third.tick(3.0 * step));
        let hiss = self.hiss.band(self.noise.tick()) * self.gain;
        (hum + hiss) * run * MOTOR.3
    }
}
