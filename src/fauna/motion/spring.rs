//! Second-order dynamics: a value that follows a target with a frequency, a damping and a response.
//!
//! After t3ssel8r's "Giving Personality to Procedural Animations using Math": `f` is the natural
//! frequency in hertz, `zeta` the damping (below 1 it overshoots, 1 is critical) and `r` the
//! initial response (0 eases in, 1 follows at once, above 1 overshoots, below 0 anticipates). The
//! `k2` term is raised far enough each step to stay stable for any time step.

use std::f32::consts::PI;
use std::ops::{Add, Mul, Sub};

/// A value a spring can move: it adds, subtracts and scales like a vector.
pub trait Value: Copy + Add<Output = Self> + Sub<Output = Self> + Mul<f32, Output = Self> {}

impl<T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>> Value for T {}

/// The three constants of a second-order system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    /// Damping term.
    k1: f32,
    /// Inertia term, before it is made stable for a step.
    k2: f32,
    /// Response term, how much the target's own velocity drives the value.
    k3: f32,
}

impl Spring {
    /// Returns the system of frequency `f` in hertz, damping `zeta` and response `r`.
    pub fn new(f: f32, zeta: f32, r: f32) -> Self {
        let w = 2.0 * PI * f.max(1e-3);
        Self {
            k1: zeta / (PI * f.max(1e-3)),
            k2: 1.0 / (w * w),
            k3: r * zeta / w,
        }
    }

    /// Returns `k2` raised so that a step of `dt` seconds neither jitters nor blows up.
    fn stable(&self, dt: f32) -> f32 {
        self.k2
            .max(dt * dt / 2.0 + dt * self.k1 / 2.0)
            .max(dt * self.k1)
    }

    /// Moves `at` and its velocity `vel` toward `target`, which moves at `pull`, by `dt` seconds.
    pub fn drive<T: Value>(&self, (at, vel): (&mut T, &mut T), target: T, pull: T, dt: f32) {
        *at = *at + *vel * dt;
        *vel = *vel + self.accel(target - *at, pull, *vel, dt);
    }

    /// Returns the change of velocity over `dt` of a value off its target by `error`, at velocity
    /// `vel`, chasing a target that moves at `pull`.
    pub fn accel<T: Value>(&self, error: T, pull: T, vel: T, dt: f32) -> T {
        (error + pull * self.k3 - vel * self.k1) * (dt / self.stable(dt))
    }
}

/// A value that follows its target through a [`Spring`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Follow<T> {
    /// Current value.
    pub at: T,
    /// Current velocity, per second.
    pub vel: T,
    /// Target of the last step, to estimate how fast it moves.
    last: T,
}

impl<T: Value> Follow<T> {
    /// Returns a value resting at `at`, with `zero` as its velocity.
    pub fn new(at: T, zero: T) -> Self {
        Self {
            at,
            vel: zero,
            last: at,
        }
    }

    /// Moves the value toward `target` by `dt` seconds of `spring` and returns it.
    pub fn step(&mut self, spring: &Spring, target: T, dt: f32) -> T {
        if dt <= 0.0 {
            return self.at;
        }
        let pull = (target - self.last) * (1.0 / dt);
        self.last = target;
        spring.drive((&mut self.at, &mut self.vel), target, pull, dt);
        self.at
    }
}
