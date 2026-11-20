//! The simulated life of a body: a small simulation of the controller and the flight gear
//! that stands, walks, runs, jumps, flies, touches down, swims and dives on cue.

use bevy::prelude::*;

use crate::gear::{FlightState, Mode as Flight};
use crate::player::Motion;

/// Pull of gravity on the simulated body, in meters per second squared.
const GRAVITY: f32 = crate::gear::tune::REF_GRAVITY;

/// What the body is asked to do for a while.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Act {
    /// Reach this ground speed, in meters per second, turning at this rate in radians per second.
    Go(f32, f32),
    /// Jump, then fall back to the ground.
    Hop,
    /// Spread the wings and glide.
    Soar,
    /// Fire the boots on the wings and pull up.
    Burn,
    /// Fold the wings and come down on the boots.
    Flare,
    /// Swim at the surface, or under it.
    Swim(bool),
    /// Touch down from the air, the gear taking it as this mode, and shed speed on the ground.
    Touch(Flight),
}

/// The life: seconds of each act, in order. It stands, walks, runs, jumps at a run and lands
/// in its stride, stops, jumps on the spot, jumps off into a glide, flies under power, flares
/// down on the boots, lands hard, glides again into a roll and a skid, swims, dives and stands
/// again.
pub const LIFE: [(f32, Act); 21] = [
    (0.6, Act::Go(0.0, 0.0)),
    (1.0, Act::Go(5.0, 0.0)),
    (1.2, Act::Go(8.5, 0.8)),
    (1.1, Act::Hop),
    (0.6, Act::Go(8.5, -0.8)),
    (0.9, Act::Go(0.0, 0.0)),
    (1.2, Act::Hop),
    (0.5, Act::Go(3.0, 0.0)),
    (0.6, Act::Hop),
    (2.4, Act::Soar),
    (1.6, Act::Burn),
    (1.0, Act::Soar),
    (2.6, Act::Flare),
    (1.0, Act::Go(0.0, 0.0)),
    (0.6, Act::Hop),
    (1.5, Act::Soar),
    (0.7, Act::Touch(Flight::Roll)),
    (1.2, Act::Touch(Flight::Slide)),
    (1.2, Act::Swim(false)),
    (0.8, Act::Swim(true)),
    (1.0, Act::Go(0.0, 0.0)),
];

/// The simulated body: what the controller and the gear would hold.
#[derive(Default)]
pub struct Body {
    /// The controller state.
    pub motion: Motion,
    /// The gear state.
    pub gear: FlightState,
    /// Free height under the body, in meters.
    pub height: f32,
    /// Whether it is deep under water.
    pub deep: bool,
}

/// Returns `value` moved toward `goal` by at most `step`.
fn toward(value: f32, goal: f32, step: f32) -> f32 {
    value + (goal - value).clamp(-step, step)
}

impl Body {
    /// Moves the ground speed toward `speed` and turns the heading, on the feet.
    fn go(&mut self, (speed, turn): (f32, f32), dt: f32) {
        self.motion.swimming = false;
        self.deep = false;
        if !self.motion.grounded && self.gear.mode == Flight::Off {
            return self.fall(dt);
        }
        (self.motion.grounded, self.gear.mode) = (true, Flight::Off);
        self.motion.yaw += turn * dt;
        let ahead = toward(self.motion.velocity.z.abs(), speed, 25.0 * dt);
        self.motion.velocity = Vec3::new(0.0, 0.0, -ahead);
        self.gear.wings = toward(self.gear.wings, 0.0, dt / 0.4);
    }

    /// Falls freely and stands once the ground is reached.
    fn fall(&mut self, dt: f32) {
        self.motion.velocity.y -= GRAVITY * dt;
        self.height += self.motion.velocity.y * dt;
        if self.height <= 0.0 {
            (self.height, self.motion.grounded) = (0.0, true);
            self.motion.velocity.y = 0.0;
        }
    }

    /// Moves the gear toward a flight in `mode`: the wings and the boots spool at a steady
    /// rate, the attitude, the load and the speeds ease, as the real gear does.
    fn fly(&mut self, mode: Flight, (wings, thrust, pitch, load): (f32, f32, f32, f32), dt: f32) {
        let ease = 1.0 - (-4.0 * dt).exp();
        let gear = &mut self.gear;
        (gear.mode, self.motion.grounded) = (mode, false);
        gear.wings = toward(gear.wings, wings, dt / 0.4);
        gear.thrust = toward(gear.thrust, thrust, dt / 0.5);
        gear.prone += (wings - gear.prone) * ease;
        gear.pitch += (pitch - gear.pitch) * ease;
        gear.load += (load - gear.load) * ease;
        gear.bank += (0.5 * pitch.signum() - gear.bank) * ease * 0.5;
        gear.airspeed += (10.0 + 30.0 * wings - gear.airspeed) * ease;
        let sink = Vec3::new(0.0, -6.0 + 8.0 * pitch, -gear.airspeed);
        gear.velocity += (sink - gear.velocity) * ease;
        self.motion.velocity = gear.velocity;
    }

    /// Comes down on the boots from 8 m up and stands, the gear off, once the ground is reached.
    fn flare(&mut self, dt: f32) {
        if self.motion.grounded {
            self.gear.mode = Flight::Off;
            return self.go((0.0, 0.0), dt);
        }
        self.fly(Flight::BootFall, (0.0, 1.0, 0.0, 1.4), dt);
        self.height = (self.height.min(8.0) + self.gear.velocity.y * dt).max(0.0);
        self.motion.grounded = self.height <= 0.0;
    }

    /// Touches down in `mode`: the first frame the gear, still in the air, records the hit and
    /// loses the fall into the ground, then it rolls or slides on, shedding speed.
    fn touch(&mut self, mode: Flight, dt: f32) {
        let gear = &mut self.gear;
        if gear.mode.airborne() && gear.servo.impact == 0.0 {
            (gear.servo.struck, gear.servo.impact) = (gear.velocity, -gear.velocity.y);
            gear.velocity.y = 0.0;
        } else {
            (gear.mode, gear.servo.impact, self.motion.grounded) = (mode, 0.0, true);
            gear.velocity *= (-2.0 * dt).exp();
            gear.prone = toward(gear.prone, 0.0, dt / 0.4);
            gear.wings = toward(gear.wings, 0.0, dt / 0.4);
        }
        (self.motion.velocity, self.height) = (gear.velocity, 0.0);
    }

    /// Advances the body by `dt` seconds of `act`.
    pub fn act(&mut self, act: Act, dt: f32) {
        match act {
            Act::Go(speed, turn) => self.go((speed, turn), dt),
            Act::Hop => {
                if self.motion.grounded {
                    self.motion.grounded = false;
                    self.motion.velocity.y = 4.7;
                }
                self.fall(dt);
            }
            Act::Soar => self.fly(Flight::Glide, (1.0, 0.0, -0.3, 1.0), dt),
            Act::Burn => self.fly(Flight::Fly, (1.0, 1.0, 0.6, 2.8), dt),
            Act::Flare => self.flare(dt),
            Act::Swim(deep) => {
                (self.motion.swimming, self.motion.grounded) = (true, false);
                (self.deep, self.gear.mode) = (deep, Flight::Off);
                self.motion.velocity = Vec3::new(0.0, 0.0, -2.0);
            }
            Act::Touch(mode) => self.touch(mode, dt),
        }
        self.height = if self.gear.mode.winged() {
            60.0
        } else {
            self.height
        };
    }
}
