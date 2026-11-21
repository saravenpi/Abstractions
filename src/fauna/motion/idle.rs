//! The idle layer: breathing, a slow weight shift and antenna flicks on seeded timers.
//!
//! Everything here runs on its own seeded clock, so two individuals of a species never breathe or
//! flick in step. The resting breath rate falls with mass as `mass^-0.26` (see [`Tune::breath`]);
//! each breath is a little longer or shorter than the last. Effort, from running or alarm, raises
//! the rate at once and fades over [`RECOVER`] seconds, so a body pants after a chase.

use std::f32::consts::{PI, TAU};

use super::tune::{BREATH_JITTER, EFFORT_RISE, PANT, RECOVER, SHIFT_RATE, Tune};
use crate::universe::hash::Rng;

/// How long a flick lasts, in seconds.
const FLICK: f32 = 0.3;

/// The idle clocks of one individual.
#[derive(Debug, Clone, Copy)]
pub struct Idle {
    /// Seeded sequence of the flick timers and of the length of each breath.
    rng: Rng,
    /// Seconds until the next flick.
    next: f32,
    /// Seconds since the last flick started.
    age: f32,
    /// Side of the last flick, -1 or 1.
    side: i8,
    /// Phase of the breath, radians.
    breath: f32,
    /// Rate of this breath over the resting rate of the body, drawn anew at each breath.
    jitter: f32,
    /// Effort the body is breathing off, 0 at rest to 1 after a run or a fight.
    effort: f32,
    /// Phase of the weight shift, radians.
    shift: f32,
    /// Weight shifts per second of this individual.
    sway: f32,
}

impl Idle {
    /// Returns the idle clocks of the individual of `seed`, each at a seeded phase.
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let breath = rng.range(0.0, f64::from(TAU)) as f32;
        let shift = rng.range(0.0, f64::from(TAU)) as f32;
        let next = rng.range(0.5, 4.0) as f32;
        let jitter = rng.range(
            f64::from(1.0 - BREATH_JITTER),
            f64::from(1.0 + BREATH_JITTER),
        ) as f32;
        let sway = SHIFT_RATE * rng.range(0.75, 1.3) as f32;
        Self {
            rng,
            next,
            age: FLICK,
            side: 1,
            breath,
            jitter,
            effort: 0.0,
            shift,
            sway,
        }
    }

    /// Returns the breaths per second of a body tuned by `tune` now: its resting rate, this breath's
    /// jitter, and up to [`PANT`] times more at full effort.
    pub fn rate(&self, tune: &Tune) -> f32 {
        tune.breath() * self.jitter * (1.0 + (PANT - 1.0) * self.effort)
    }

    /// Runs the clocks of a body tuned by `tune` for `dt` seconds, alarmed by `excite` and working at
    /// `work`, each 0 to 1: the effort rises to the harder of the two within [`EFFORT_RISE`] seconds
    /// and falls back over [`RECOVER`].
    pub fn step(&mut self, tune: &Tune, (excite, work): (f32, f32), dt: f32) {
        let goal = excite.max(work).clamp(0.0, 1.0);
        let rate = if goal > self.effort {
            1.0 / EFFORT_RISE
        } else {
            1.0 / RECOVER
        };
        self.effort += (goal - self.effort) * (rate * dt).min(1.0);
        let breath = self.breath + TAU * self.rate(tune) * dt;
        if breath >= TAU {
            let spread = f64::from(BREATH_JITTER);
            self.jitter = self.rng.range(1.0 - spread, 1.0 + spread) as f32;
        }
        self.breath = breath.rem_euclid(TAU);
        self.shift = (self.shift + TAU * self.sway * dt).rem_euclid(TAU);
        self.age += dt;
        self.next -= dt * (1.0 + 1.5 * excite.clamp(0.0, 1.0));
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

    /// Returns the phase of the breath in radians, 0 to a full turn.
    #[cfg(test)]
    pub fn phase(&self) -> f32 {
        self.breath
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
