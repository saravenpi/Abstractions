//! The frame of the canopy: two thin bows under the glass, one at the windscreen and one behind
//! the pilot. They carry the glass and leave the view free: there is nothing along the roof.

use bevy::prelude::*;

use super::frame::CANOPY;
use super::shell::arch;
use crate::ship::solid::Solid;

/// Width of a bar along the glass and its depth under it, in metres.
const BAR: (f32, f32) = (0.02, 0.012);

/// Adds a bow under the glass at `station`, `along` z from it.
fn bow(solid: &mut Solid, station: [f32; 4], along: f32) {
    let points = arch(station);
    let axis = Vec3::new(0.0, station[3], station[0]);
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let inward = (axis - (a + b) * 0.5).normalize_or_zero();
        let wide = Vec3::Z * along;
        solid.plate([a, b, b + wide, a + wide], inward * BAR.1);
    }
}

/// Returns the frame of the canopy.
pub fn frame() -> Solid {
    let mut frame = Solid::default();
    bow(&mut frame, CANOPY[0], BAR.0);
    bow(&mut frame, CANOPY[1], -BAR.0);
    frame
}
