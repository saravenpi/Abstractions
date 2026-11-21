//! What speed does to the picture and the sound, from real quantities only.
//!
//! The air gives dynamic pressure `1/2 rho v^2` and a speed of sound that follows from its
//! temperature and its gas mix; a vacuum gives neither, so every cue that needs air is zero
//! there. Nothing here fakes a speed the body does not have: a cue is a function of the speed, the
//! density or the wetness of the air the body is in, and the player's "Speed effects" setting
//! ([`crate::view::ViewPrefs::speed_fx`]) scales the field of view change, the shake and the
//! streaks of the air, and turns them off at 0.
//!
//! What it applies, and where that comes from, is in `handoff/specs/speed.md`.

mod air;
mod cues;
pub mod shake;
pub mod tune;

#[cfg(test)]
mod tests_air;
#[cfg(test)]
mod tests_cues;

pub use air::{dynamic_pressure, mach, sound_at, sound_speed, temperature_at, temperature_over};
pub use cues::{buffet, condensation, cone, contrail, moisture, persistence, saturation, vortex};
