//! The beat of one attack: a wind-up that grows with the size of the body, the blow, and a recovery
//! longer than the wind-up. The same Froude scaling as the gaits.

use bevy::prelude::*;

use super::kinds::Kind;
use super::tune::{DASH, RECOVER, STRIKE, WIND_BASE, WIND_FROUDE};

/// The attack an animal is making now.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Attacking {
    /// Which attack.
    pub kind: Kind,
    /// Seconds since it started.
    pub t: f32,
    /// Seconds of its wind-up.
    pub wind: f32,
    /// True once the blow has landed or missed.
    pub struck: bool,
}

/// Returns the seconds of wind-up of a body `length` metres long under `gravity` metres per second
/// squared: 0.38 s at 0.1 m, 0.68 s at 1 m and 1.1 s at 4 m under 22 m/s².
pub fn wind(length: f32, gravity: f32) -> f32 {
    WIND_BASE + WIND_FROUDE * (length.max(0.0) / gravity.max(1e-3)).sqrt()
}

/// Returns the energy in joules of a blow of `kind` from a body of `mass` kilograms whose striking
/// part moves at `speed` metres per second.
pub fn joules(kind: Kind, mass: f32, speed: f32) -> f32 {
    0.5 * mass * kind.share() * speed * speed
}

impl Attacking {
    /// Returns an attack of `kind` that just began on a body `length` metres long under `gravity`.
    pub fn begin(kind: Kind, length: f32, gravity: f32) -> Self {
        Self {
            kind,
            t: 0.0,
            wind: wind(length, gravity),
            struck: false,
        }
    }

    /// Returns the seconds the blow itself takes.
    pub fn blow(&self) -> f32 {
        if self.kind.dashes() { DASH } else { STRIKE }
    }

    /// Returns true while the blow is on its way.
    pub fn striking(&self) -> bool {
        self.t >= self.wind && self.t < self.wind + self.blow()
    }

    /// Returns true once the recovery is over.
    pub fn over(&self) -> bool {
        self.t >= self.wind + self.blow() + RECOVER * self.wind
    }

    /// Returns how far the attack is thrown, from -1 (drawn all the way back) through 0 (rest) to 1
    /// (all the way out), the pose telegraphs follow.
    pub fn swing(&self) -> f32 {
        let (wind, blow) = (self.wind.max(1e-3), self.blow());
        let out = self.t - wind;
        if self.t < wind {
            -self.t / wind
        } else if out < blow {
            -1.0 + 2.0 * out / blow
        } else {
            (1.0 - (out - blow) / (RECOVER * wind)).max(0.0)
        }
    }
}
