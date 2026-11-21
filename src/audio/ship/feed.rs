//! What one ship sounds like right now, as plain numbers, and the lock-free cells that carry
//! them to the audio thread. The game writes one [`HullSound`] per ship in earshot every frame;
//! the audio thread reads them a few hundred times per second.

use std::sync::atomic::{AtomicU32, Ordering};

/// Ships that can sound at once: the one this machine flies, then the nearest two of the others.
pub const HULLS: usize = 3;

/// Number of [`Ch`] values.
pub const CHANNELS: usize = 18;

/// One number of the sound of a ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ch {
    /// 1 while the ship exists within earshot, else 0.
    Present,
    /// 1 while the listener sits in this ship, 0 when it is heard from outside.
    Inside,
    /// 1 while the engines run: someone is aboard or the ship flies.
    Power,
    /// Thrust of the main engines: 1 at full throttle, more with the boost.
    Thrust,
    /// Thrust of the lift nozzles, 1 at their full push.
    Lift,
    /// How far the warp drive is up, from 0 to 1: the share of its spool, then all of it.
    Warp,
    /// How hard the attitude jets of the stabiliser fire, from 0 to 1.
    Rcs,
    /// Heat left in the nozzles, from 0 to 1.
    Glow,
    /// Density of the air around the ship as a share of the air at sea level at home.
    Air,
    /// Speed of sound in that air, in metres per second.
    Sound,
    /// Speed of the ship through the air, in metres per second.
    Speed,
    /// Height of the gear above the ground, in metres.
    Height,
    /// Extension of the landing gear, from 0 (folded) to 1 (down and locked).
    Gear,
    /// How far the canopy is open, from 0 (shut) to 1.
    Canopy,
    /// How hard the air shakes the hull, from 0 to 1.
    Rough,
    /// Distance from the listener to the ship, in metres.
    Distance,
    /// Side of the ship for the listener, from -1 (left) to 1 (right).
    Side,
    /// Cosine of the angle between the exhaust and the line from the ship to the listener: 1
    /// straight behind the nozzles, -1 ahead of the nose.
    Aspect,
}

impl Ch {
    /// Returns the lowest and the highest value the channel may hold.
    pub const fn span(self) -> (f32, f32) {
        match self {
            Ch::Thrust | Ch::Lift => (0.0, 2.0),
            Ch::Air => (0.0, 8.0),
            Ch::Sound => (50.0, 3000.0),
            Ch::Speed | Ch::Height | Ch::Distance => (0.0, 1.0e5),
            Ch::Side | Ch::Aspect => (-1.0, 1.0),
            _ => (0.0, 1.0),
        }
    }
}

/// The sound of one ship, one value per [`Ch`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HullSound(pub [f32; CHANNELS]);

impl Default for HullSound {
    /// No ship: silent, in still air of the home planet, gear down, far above any ground.
    fn default() -> Self {
        Self([0.0; CHANNELS])
            .with(Ch::Sound, 340.0)
            .with(Ch::Gear, 1.0)
            .with(Ch::Height, 1.0e4)
            .with(Ch::Aspect, -1.0)
    }
}

impl HullSound {
    /// Returns the value of one channel.
    pub fn at(&self, ch: Ch) -> f32 {
        self.0[ch as usize]
    }

    /// Returns the sound with one channel set, held inside its span. A value that is not a
    /// number becomes the lowest the channel may hold.
    pub fn with(mut self, ch: Ch, value: f32) -> Self {
        let (low, high) = ch.span();
        self.0[ch as usize] = if value.is_finite() {
            value.clamp(low, high)
        } else {
            low
        };
        self
    }
}

/// The cells the sounds of the ships cross to the audio thread in: atomics only.
#[derive(Debug)]
pub struct Deck {
    /// Bits of the `f32` value of each channel of each ship.
    cells: [[AtomicU32; CHANNELS]; HULLS],
}

impl Default for Deck {
    /// Starts with no ship.
    fn default() -> Self {
        let deck = Self {
            cells: std::array::from_fn(|_| std::array::from_fn(|_| AtomicU32::new(0))),
        };
        deck.publish(&[HullSound::default(); HULLS]);
        deck
    }
}

impl Deck {
    /// Stores the sound of every ship.
    pub fn publish(&self, hulls: &[HullSound; HULLS]) {
        for (cells, hull) in self.cells.iter().zip(hulls) {
            for (cell, value) in cells.iter().zip(hull.0) {
                cell.store(value.to_bits(), Ordering::Relaxed);
            }
        }
    }

    /// Returns the sound of the ship in `slot`.
    pub fn read(&self, slot: usize) -> HullSound {
        let cells = &self.cells[slot];
        HullSound(std::array::from_fn(|i| {
            f32::from_bits(cells[i].load(Ordering::Relaxed))
        }))
    }
}
