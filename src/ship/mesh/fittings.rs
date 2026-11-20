//! What is bolted into the cockpit tub: the seat with its harness, the side consoles, the
//! ribs along the walls, the dashboard with its raked panel and the black face of the
//! instruments.
//!
//! The ribs follow the lips of the mouth and the walls of the tub, which `shell` and `tub`
//! own, and the rest stands on the places `cockpit::tune` gives: nothing here knows the size
//! of the hull.

mod cushions;
mod dash;
mod ribs;

use bevy::prelude::*;

use crate::ship::cockpit::{BACK, FLOOR};
use crate::ship::solid::Solid;

pub use cushions::seat;
pub use dash::faces;

/// Height of the top of the front of the side consoles, where the stick and the throttle
/// hinge, and of the back of them, where the gear lever and the warp switch are.
pub const CONSOLE: (f32, f32) = (0.31, 0.25);

/// Adds the box between `lo` and `hi` and its mirror image on the left of the hull.
pub(super) fn pair(solid: &mut Solid, lo: Vec3, hi: Vec3) {
    solid.slab(lo, hi);
    solid.slab(Vec3::new(-hi.x, lo.y, lo.z), Vec3::new(-lo.x, hi.y, hi.z));
}

/// Returns the dark furniture of the tub: the foot of the seat, the two side consoles with
/// their columns for the stick and the throttle and their plates, the ribs along the walls and
/// the dashboard.
pub fn furniture() -> Solid {
    let mut dark = Solid::default();
    dark.slab(Vec3::new(-0.14, FLOOR, -1.3), Vec3::new(0.14, 0.0, -1.0));
    pair(
        &mut dark,
        Vec3::new(0.22, FLOOR, -1.85),
        Vec3::new(0.36, CONSOLE.0, -1.6),
    );
    pair(
        &mut dark,
        Vec3::new(0.22, FLOOR, -1.6),
        Vec3::new(0.36, CONSOLE.1, BACK),
    );
    pair(
        &mut dark,
        Vec3::new(0.235, CONSOLE.1, -1.55),
        Vec3::new(0.345, CONSOLE.1 + 0.012, -1.1),
    );
    dark.merge(&dash::dash(), Vec3::ZERO);
    dark.merge(&ribs::ribs(), Vec3::ZERO);
    dark
}

#[cfg(test)]
mod tests_dash;
