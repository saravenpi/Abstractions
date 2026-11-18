//! The landing gear: it folds into the hull above the ground and comes down for a landing, and
//! the contact points of its feet follow it.

use bevy::math::DVec3;

use super::body::Ship;

/// Feet of the landing gear in body axes when it is down: nose, left and right.
pub const GEAR: [DVec3; 3] = [
    DVec3::new(0.0, -1.25, -2.4),
    DVec3::new(-1.9, -1.25, 1.5),
    DVec3::new(1.9, -1.25, 1.5),
];

/// Height in metres of a gear strut above its foot, where it hinges into the hull.
pub const STRUT: f64 = 1.12;

/// Share of the strut length a folded leg keeps: its pad then sits flush with the belly as a skid.
pub const FOLDED: f64 = 0.42;

/// Heights in metres of the gear above the ground over which it folds up and under which it
/// comes down. The gap between them keeps a bumpy approach from cycling it.
pub const HEIGHTS: (f64, f64) = (40.0, 25.0);

/// Seconds the gear takes to come down or fold up all the way.
pub const TRAVEL: f64 = 2.0;

/// Moves the gear for `h` seconds with the feet `radar` metres above the ground: up above the
/// first height of [`HEIGHTS`], down under the second, and on its way between them.
pub fn stow(ship: &mut Ship, radar: f64, h: f64) {
    if radar > HEIGHTS.0 {
        ship.stowed = true;
    } else if radar < HEIGHTS.1 {
        ship.stowed = false;
    }
    let way = if ship.stowed { -1.0 } else { 1.0 };
    ship.gear = (ship.gear + way * h / TRAVEL).clamp(0.0, 1.0);
}

/// Returns the share of the strut length a leg shows for a gear extension from 0 (folded) to 1.
pub fn reach(extension: f64) -> f64 {
    FOLDED + (1.0 - FOLDED) * extension.clamp(0.0, 1.0)
}

/// Returns where the feet of the gear are in body axes for the extension of the gear of `ship`.
pub fn feet(ship: &Ship) -> [DVec3; 3] {
    let drop = STRUT * (1.0 - reach(ship.gear));
    GEAR.map(|foot| foot + DVec3::Y * drop)
}

/// Returns true when the gear is down and locked: only then do its feet spring and carry the ship.
pub fn locked(ship: &Ship) -> bool {
    ship.gear >= 1.0
}
