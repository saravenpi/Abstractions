//! The fade between two maps of the galaxy light: a new bake does not replace the map on show
//! at once, it is mixed in over [`FADE`] seconds, so a fast warp into a galaxy makes the sky
//! grow smoothly although the maps come seconds apart.

use super::tune::FADE;

/// Writes into `out` the bytes of `from` at `share` 0 and of `to` at `share` 1, and the
/// blend of the two in between, byte by byte.
pub fn mix(from: &[u8], to: &[u8], share: f32, out: &mut [u8]) {
    let share = share.clamp(0.0, 1.0);
    for ((out, from), to) in out.iter_mut().zip(from).zip(to) {
        let (from, to) = (f32::from(*from), f32::from(*to));
        *out = (from + (to - from) * share).round() as u8;
    }
}

/// The map on show, the map it was fading from and the one it is fading to.
#[derive(Default)]
pub struct Fade {
    /// The map the fade started from: what was on show when the last bake landed.
    from: Vec<u8>,
    /// The last bake, the map the fade ends on.
    to: Vec<u8>,
    /// The map on show now.
    shown: Vec<u8>,
    /// Seconds of the app clock at which the last bake landed.
    since: f64,
    /// Whether the map on show is still short of the last bake.
    active: bool,
}

impl Fade {
    /// Starts fading to `bytes`, a bake that landed at `now` seconds of the app clock, from the
    /// map on show, which is the bake itself when none is on show yet.
    pub fn begin(&mut self, bytes: Vec<u8>, now: f64) {
        let held = self.shown.len() == bytes.len();
        self.from = if held {
            self.shown.clone()
        } else {
            bytes.clone()
        };
        self.shown = self.from.clone();
        (self.to, self.since, self.active) = (bytes, now, true);
    }

    /// Returns the map on show at `now` seconds of the app clock while it changes, and nothing
    /// once it is the last bake and has been written.
    pub fn step(&mut self, now: f64) -> Option<&[u8]> {
        if !self.active {
            return None;
        }
        let share = ((now - self.since) / FADE).clamp(0.0, 1.0);
        self.active = share < 1.0;
        mix(&self.from, &self.to, share as f32, &mut self.shown);
        Some(&self.shown)
    }
}
