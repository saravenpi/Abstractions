//! Rechargeable energy shield that absorbs impact joules before armor and hull integrity.

use bevy::prelude::*;

/// Seconds after taking a hit before the energy shield begins recharging.
pub const RECHARGE_DELAY: f32 = 5.0;

/// Fraction of maximum shield capacity recharged per second once delay expires.
pub const RECHARGE_RATE: f32 = 0.14;

/// Active rechargeable energy shield pool granted by equipped Aegis armor.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Shield {
    /// Current shield energy in joules.
    pub now: f32,
    /// Maximum shield capacity in joules.
    pub max: f32,
    /// Remaining recharge lockout timer in seconds.
    pub delay: f32,
}

impl Shield {
    /// Returns the charged fraction in `0.0..=1.0`, or `0.0` when no shield is equipped.
    pub fn share(&self) -> f32 {
        if self.max <= 0.0 {
            return 0.0;
        }
        (self.now / self.max).clamp(0.0, 1.0)
    }

    /// Absorbs `joules` from `now`, resets `delay` to [`RECHARGE_DELAY`], and returns overflow joules.
    pub fn absorb(&mut self, joules: f32) -> f32 {
        let hit = joules.max(0.0);
        if hit <= 0.0 {
            return 0.0;
        }
        if self.max > 0.0 {
            self.delay = RECHARGE_DELAY;
        }
        let taken = self.now.min(hit);
        self.now = (self.now - taken).max(0.0);
        hit - taken
    }

    /// Advances recharge delay and restores shield capacity at `0.14 * max` joules per second.
    pub fn tick(&mut self, dt: f32) {
        let step = dt.max(0.0);
        if self.max <= 0.0 {
            self.now = 0.0;
            self.delay = 0.0;
            return;
        }
        if self.delay > 0.0 {
            self.delay = (self.delay - step).max(0.0);
            return;
        }
        self.now = (self.now + RECHARGE_RATE * self.max * step).min(self.max);
    }

    /// Updates maximum capacity to `max` joules, clamping current charge within the new limit.
    pub fn set_max(&mut self, max: f32) {
        self.max = max.max(0.0);
        self.now = self.now.min(self.max);
    }
}

/// ECS system ticking energy shield recharge each frame.
pub(super) fn recharge(time: Res<Time>, mut shield: ResMut<Shield>) {
    shield.tick(time.delta_secs());
}
