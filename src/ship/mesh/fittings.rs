//! What is bolted into the cockpit tub: the seat with its harness, the side consoles, the
//! ribs along the walls, the dashboard with its instrument pods and its raked panel, the
//! column of the stick and the black faces of the instruments.
//!
//! The ribs follow the lips of the mouth and the walls of the tub, which `shell` and `tub`
//! own, and the rest stands on the places `cockpit::tune` gives: nothing here knows the size
//! of the hull.

mod cushions;
mod dash;
mod ribs;

use bevy::prelude::*;

use crate::ship::cockpit::{BACK, FLOOR, STICK};
use crate::ship::solid::Solid;

pub use cushions::seat;
pub use dash::faces;

/// Height of the top of the side consoles: the throttle hinges on the left one.
pub const CONSOLE: f32 = 0.22;

/// Adds the box between `lo` and `hi` and its mirror image on the left of the hull.
pub(super) fn pair(solid: &mut Solid, lo: Vec3, hi: Vec3) {
    solid.slab(lo, hi);
    solid.slab(Vec3::new(-hi.x, lo.y, lo.z), Vec3::new(-lo.x, hi.y, hi.z));
}

/// Returns the dark furniture of the tub: the foot of the seat, the column of the stick, the
/// two side consoles with their plates, the ribs along the walls and the dashboard.
pub fn furniture() -> Solid {
    let mut dark = Solid::default();
    dark.slab(Vec3::new(-0.14, FLOOR, -1.3), Vec3::new(0.14, 0.0, -1.0));
    dark.slab(
        Vec3::new(-0.025, FLOOR, STICK.z - 0.025),
        Vec3::new(0.025, STICK.y, STICK.z + 0.025),
    );
    pair(
        &mut dark,
        Vec3::new(0.22, FLOOR, -1.8),
        Vec3::new(0.36, CONSOLE, BACK),
    );
    pair(
        &mut dark,
        Vec3::new(0.235, CONSOLE, -1.7),
        Vec3::new(0.345, CONSOLE + 0.012, -1.1),
    );
    dark.merge(&dash::dash(), Vec3::ZERO);
    dark.merge(&ribs::ribs(), Vec3::ZERO);
    dark
}

#[cfg(test)]
mod tests_dash;
