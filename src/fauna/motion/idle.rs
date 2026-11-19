//! The idle layer: breathing, a slow weight shift and antenna flicks on seeded timers.
//!
//! Everything here runs on its own seeded clock, so two individuals of a species never breathe or
//! flick in step. An alarmed body breathes faster and flicks more often.

use std::f32::consts::{PI, TAU};

use super::tune::Tune;
use crate::universe::hash::Rng;

/// How long a flick lasts, in seconds.
const FLICK: f32 = 0.3;

/// The idle clocks of one individual.
#[derive(Debug, Clone, Copy)]
pub struct Idle {
    /// Seeded sequence of the flick timers.
    rng: Rng,
    /// Seconds until the next flick.
    next: f32,
    /// Seconds since the last flick started.
    age: f32,
    /// Side of the last flick, -1 or 1.
    side: i8,
    /// Phase of the breath, radians.
    breath: f32,
    /// Phase of the weight shift, radians.
    shift: f32,
}

impl Idle {
    /// Returns the idle clocks of the individual of `seed`.
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let breath = rng.range(0.0, f64::from(TAU)) as f32;
        let shift = rng.range(0.0, f64::from(TAU)) as f32;
        let next = rng.range(0.5, 4.0) as f32;
        Self {
            rng,
            next,
            age: FLICK,
            side: 1,
            breath,
            shift,
        }
    }

    /// Runs the clocks of a body tuned by `tune` and alarmed by `excite` for `dt` seconds.
    pub fn step(&mut self, tune: &Tune, excite: f32, dt: f32) {
        let pace = 1.0 + 1.5 * excite.clamp(0.0, 1.0);
        self.breath = (self.breath + TAU * tune.breath() * pace * dt).rem_euclid(TAU);
        self.shift = (self.shift + TAU * 0.13 * dt).rem_euclid(TAU);
        self.age += dt;
        self.next -= dt * pace;
        if self.next <= 0.0 {
            self.next = self.rng.range(1.5, 6.0) as f32;
            self.side = if self.rng.below(2) == 0 { -1 } else { 1 };
            self.age = 0.0;
        }
    }

    /// Returns the breath, from -1 out to 1 in.
    pub fn breath(&self) -> f32 {
        self.breath.sin()
    }

    /// Returns the weight shift, from -1 left to 1 right.
    pub fn shift(&self) -> f32 {
        self.shift.sin()
    }

    /// Returns the angle in radians of the flick running now and its side.
    pub fn flick(&self) -> (f32, i8) {
        let angle = match self.age < FLICK {
            true => 0.5 * (PI * self.age / FLICK).sin(),
            false => 0.0,
        };
        (angle, self.side)
    }
}
