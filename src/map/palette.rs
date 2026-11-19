//! The twelve player colours, used for player dots, friend compass marks and waypoints.
//!
//! A player keeps the same colour on every screen: the colour comes from a hash of the player's id.

use bevy::prelude::*;

/// The twelve player colours, far apart in hue and bright enough to read on the dark map.
pub const PALETTE: [Color; 12] = [
    Color::srgb(0.95, 0.30, 0.30),
    Color::srgb(0.98, 0.60, 0.20),
    Color::srgb(0.98, 0.88, 0.25),
    Color::srgb(0.62, 0.90, 0.25),
    Color::srgb(0.25, 0.80, 0.40),
    Color::srgb(0.25, 0.85, 0.80),
    Color::srgb(0.30, 0.65, 0.98),
    Color::srgb(0.40, 0.42, 0.95),
    Color::srgb(0.68, 0.40, 0.95),
    Color::srgb(0.95, 0.40, 0.85),
    Color::srgb(0.98, 0.65, 0.70),
    Color::srgb(0.80, 0.72, 0.55),
];

/// Returns the colour of the player with this id, the same for the same id on every screen.
pub fn player_color(id: u32) -> Color {
    let mut x = id.wrapping_mul(0x9E37_79B9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 13;
    PALETTE[(x % PALETTE.len() as u32) as usize]
}

#[cfg(test)]
#[path = "palette_tests.rs"]
mod tests;
