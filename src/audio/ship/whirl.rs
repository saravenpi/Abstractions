//! The whoosh of a hard roll or yaw: one swell of the wind each time the rate truly takes off, and
//! never again until it has fallen back and a refractory time has passed. A rate that hovers near
//! the onset, or the small corrections of the stick in steady flight, never retrigger it.

use super::air_tune::WHIRL;

/// The whoosh of one airframe.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Whirl {
    /// How much of the whoosh sounds now, from 0 to 1. It jumps to 1 at an onset and dies away;
    /// the easing of the gains of the airframe turns the jump into a swell.
    level: f32,
    /// Seconds until another onset may sound.
    wait: f32,
    /// Whether the rate has fallen back under the re-arming rate since the last onset.
    armed: bool,
}

impl Whirl {
    /// Follows a rate of roll and yaw of `turn` radians per second, `dt` seconds after the last
    /// call.
    pub fn tune(&mut self, turn: f32, dt: f32) {
        let (rearm, onset, decay, refractory, _) = WHIRL;
        self.wait = (self.wait - dt).max(0.0);
        self.level *= (-dt / decay).exp();
        self.armed |= turn < rearm;
        if self.armed && self.wait <= 0.0 && turn > onset {
            (self.level, self.wait, self.armed) = (1.0, refractory, false);
        }
    }

    /// Returns how much of the whoosh sounds now, from 0 to 1.
    pub fn level(&self) -> f32 {
        self.level
    }
}
