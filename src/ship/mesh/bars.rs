//! The frame of the canopy: two thin bows under the glass, one in front of the pilot and one
//! behind, and two rails between them along the roof. They frame the view without blocking it.

use bevy::prelude::*;

use super::frame::CANOPY;
use super::shell::arch;
use crate::ship::solid::Solid;

/// Width of a bar along the glass and its depth under it, in metres.
const BAR: (f32, f32) = (0.03, 0.018);

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
    let (front, back) = (arch(CANOPY[0]), arch(CANOPY[1]));
    for (corner, side) in [(2, -1.0), (3, 1.0)] {
        let wide = Vec3::X * BAR.0 * side;
        let rail = [
            front[corner],
            back[corner],
            back[corner] + wide,
            front[corner] + wide,
        ];
        frame.plate(rail, Vec3::NEG_Y * BAR.1);
    }
    frame
}
