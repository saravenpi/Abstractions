//! The frame of the canopy: the hoop at its back, under the glass, and a slim rail along its
//! foot on each side and round the foot of the windscreen. Nothing crosses the glass in front of
//! the pilot or over the head: the view is free.

use bevy::prelude::*;

use super::dome::arch;
use super::frame::{CANOPY, CANOPY_ENDS, GLAZED, SUNK};
use crate::ship::solid::Solid;

/// Width of the hoop along the glass and its depth under it, and the width and the height of a
/// rail, in metres.
const BAR: (f32, f32, f32, f32) = (0.04, 0.02, 0.035, 0.03);

/// Adds the hoop under the glass at the last glazed station.
fn hoop(solid: &mut Solid) {
    let points = arch(CANOPY[GLAZED - 1]);
    let axis = Vec3::new(0.0, 0.5, points[0].z);
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let inward = (axis - (a + b) * 0.5).normalize_or_zero();
        let wide = Vec3::NEG_Z * BAR.0;
        solid.plate([a, b, b + wide, a + wide], inward * BAR.1);
    }
}

/// Adds the rail of one `side` of the canopy, -1 for the left and 1 for the right: a strip on the
/// skin of the hull just outside the foot of the glass, from the foot of the windscreen back to
/// the hoop.
fn rail(solid: &mut Solid, side: f32) {
    let foot = |at: [f32; 4]| {
        let low = arch(at)[0];
        Vec3::new(low.x * side, low.y, low.z)
    };
    let mut feet = vec![CANOPY_ENDS.0];
    feet.extend(CANOPY[..GLAZED].iter().map(|&at| foot(at)));
    for pair in feet.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let out = Vec3::X * side * BAR.2;
        solid.plate([a, b, b + out, a + out], Vec3::Y * (SUNK + BAR.3));
    }
}

/// Returns the frame of the canopy.
pub fn frame() -> Solid {
    let mut frame = Solid::default();
    hoop(&mut frame);
    rail(&mut frame, -1.0);
    rail(&mut frame, 1.0);
    frame
}
