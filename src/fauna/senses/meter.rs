//! The awareness meter: how sure an animal is, 0 to 1, that each threat near it is there.
//!
//! It fills while the animal sees, hears or smells the threat, faster the nearer it is, and drains
//! slowly once it does not. An animal keeps a meter for a few threats at once, by their keys.

use super::tune::{
    CROUCH_FILL, DRAIN, FAR, FAST, FILL_FAR, FILL_MID, FILL_NEAR, LEAST_FILL, LOUD, MID, NEAR,
    STILL,
};

/// Threats one animal keeps a meter for at once.
pub const SLOTS: usize = 4;

/// The meters of one animal, one per threat it has noticed, by the threat's key.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Meter {
    /// Key of the threat and its awareness, 0 for a free slot.
    slots: [(u64, f32); SLOTS],
}

impl Meter {
    /// Drains every meter for `dt` seconds.
    pub fn drain(&mut self, dt: f32) {
        for slot in &mut self.slots {
            slot.1 = (slot.1 - DRAIN * dt).max(0.0);
        }
    }

    /// Adds `amount` to the meter of the threat `key`, taking the emptiest slot for a new one, and
    /// returns its awareness.
    pub fn raise(&mut self, key: u64, amount: f32) -> f32 {
        let index = self
            .slots
            .iter()
            .position(|s| s.0 == key)
            .unwrap_or_else(|| {
                let free = (0..SLOTS).min_by(|a, b| self.slots[*a].1.total_cmp(&self.slots[*b].1));
                free.unwrap_or(0)
            });
        let slot = &mut self.slots[index];
        if slot.0 != key {
            *slot = (key, 0.0);
        }
        slot.1 = (slot.1 + amount).min(1.0);
        slot.1
    }

    /// Returns the awareness of the threat `key`.
    pub fn of(&self, key: u64) -> f32 {
        self.slots.iter().find(|s| s.0 == key).map_or(0.0, |s| s.1)
    }
}

/// Returns the seconds the meter takes to fill for a threat `metres` away, by the bands of
/// [`NEAR`], [`MID`] and [`FAR`]; past [`FAR`], what only the ear or the nose catches.
pub fn fill_seconds(metres: f32) -> f32 {
    match metres {
        m if m < NEAR => FILL_NEAR,
        m if m < MID => FILL_MID,
        _ => FILL_FAR,
    }
}

/// Returns the factor of the fill rate for a threat going `speed` metres per second at loudness
/// `loud`, `crouch` down from 0 to 1: half for a still or creeping one, twice for a fast or loud one,
/// and half again at full crouch, never under [`LEAST_FILL`].
pub fn stealth(speed: f32, loud: f32, crouch: f32) -> f32 {
    let base = if speed > FAST || loud > LOUD {
        2.0
    } else if speed < STILL {
        0.5
    } else {
        1.0
    };
    let hidden = 1.0 - (1.0 - CROUCH_FILL) * crouch.clamp(0.0, 1.0);
    (base * hidden).max(LEAST_FILL)
}

/// Returns true when a threat `metres` away, seen through senses of range factor `keen`, is close
/// enough to be seen at all.
pub fn in_sight(metres: f32, keen: f32) -> bool {
    metres < FAR * keen
}
