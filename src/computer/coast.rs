//! Moving the map without zooming: the speed a drag leaves it with, the drift that slows to a
//! stop after the button is let go, and the keys that pan it.
//!
//! W, A, S and D pan the map whenever nothing is typed. The arrows pan it too while the
//! pointer is over the map; elsewhere they move through the list, as before.

use bevy::prelude::*;

use super::hand::Hand;
use super::map::SIZE;

/// How fast the drift after a drag dies down, per second.
pub const FRICTION: f32 = 6.0;

/// How fast the measured speed of a drag follows the pointer, per second.
pub const GRIP: f32 = 25.0;

/// Speed in map pixels per second under which the drift stops.
pub const STOP: f32 = 4.0;

/// Fastest drift a drag leaves, in map pixels per second.
pub const FLING: f32 = 3.0 * SIZE;

/// Speed of the map in map pixels per second while a pan key is held.
pub const KEY_SPEED: f32 = 0.6 * SIZE;

/// The keys that pan the map and the way each moves the picture: W shows what is north.
const WASD: [(KeyCode, Vec2); 4] = [
    (KeyCode::KeyW, Vec2::Y),
    (KeyCode::KeyS, Vec2::NEG_Y),
    (KeyCode::KeyA, Vec2::X),
    (KeyCode::KeyD, Vec2::NEG_X),
];

/// The arrows, which pan the map the same way while the pointer is over it.
const ARROWS: [(KeyCode, Vec2); 4] = [
    (KeyCode::ArrowUp, Vec2::Y),
    (KeyCode::ArrowDown, Vec2::NEG_Y),
    (KeyCode::ArrowLeft, Vec2::X),
    (KeyCode::ArrowRight, Vec2::NEG_X),
];

/// Returns the way the held pan keys move the picture, one unit long or zero. The arrows
/// count only when `arrows` is true.
pub fn keyed(keys: &ButtonInput<KeyCode>, arrows: bool) -> Vec2 {
    let held = |pairs: &[(KeyCode, Vec2)]| {
        let pressed = pairs.iter().filter(|(key, _)| keys.pressed(*key));
        pressed.map(|(_, way)| *way).sum::<Vec2>()
    };
    let arrowed = [Vec2::ZERO, held(&ARROWS)][usize::from(arrows)];
    (held(&WASD) + arrowed).normalize_or_zero()
}

/// Returns true for the arrows, which pan the map instead while the pointer is over it.
pub fn arrow(key: KeyCode) -> bool {
    ARROWS.iter().any(|(arrow, _)| *arrow == key)
}

impl Hand {
    /// Follows a drag that moved the map by `moved` map pixels in `dt` seconds, so the drift
    /// it leaves is the speed of its last moments.
    pub fn follow(&mut self, moved: Vec2, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let share = 1.0 - (-GRIP * dt).exp();
        let speed = (moved / dt).clamp_length_max(FLING);
        self.drift = self.drift.lerp(speed, share);
    }

    /// Returns the map pixels the map moves this frame of `dt` seconds: the drift, which slows
    /// down, and the keys held the way `keys` says.
    pub fn coast(&mut self, keys: Vec2, dt: f32) -> Vec2 {
        let drift = self.drift * dt;
        self.drift *= (-FRICTION * dt).exp();
        if self.drift.length() < STOP {
            self.drift = Vec2::ZERO;
        }
        drift + keys * KEY_SPEED * dt
    }
}
