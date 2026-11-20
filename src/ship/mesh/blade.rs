//! Standing blades: the tail fin and the fins at the wing tips. A blade is a thin section,
//! pointed at the front and cut off at the back, lofted up through a few stations.

use bevy::prelude::*;

use super::loft::{Section, solid};
use crate::ship::solid::Solid;

/// Stations of the tail fin from its root up: height, z of the leading edge and of the trailing
/// edge, and half thickness. The root is inside the fuselage, and thick, so that the fin grows
/// out of the roof; the tip is 1.9 m up.
pub(super) const FIN: [[f32; 4]; 4] = [
    [0.45, 1.4, 3.15, 0.17],
    [0.78, 1.55, 3.2, 0.085],
    [1.25, 1.95, 3.3, 0.07],
    [1.9, 2.7, 3.4, 0.04],
];

/// Stations of the fin at a wing tip, like [`FIN`]: the fin starts under the wing end and
/// stands 0.41 m above the top of it, higher than a body steps up, so it stops a body on the wing.
pub(super) const WINGLET: [[f32; 4]; 3] = [
    [-0.32, 0.85, 2.25, 0.05],
    [-0.035, 1.075, 2.325, 0.05],
    [0.25, 1.3, 2.4, 0.04],
];

/// The section of a blade: the share of the chord from the leading edge, and the share of the
/// half thickness across, of each corner.
const SECTION: [(f32, f32); 5] = [
    (0.0, 0.0),
    (0.35, 1.0),
    (1.0, 0.3),
    (1.0, -0.3),
    (0.35, -1.0),
];

/// Returns the section of a blade standing at `x` metres from the middle, at a station.
fn section(x: f32, [y, front, back, half]: [f32; 4]) -> Section {
    let corner = |&(along, across): &(f32, f32)| {
        Vec3::new(x + across * half, y, front + along * (back - front))
    };
    SECTION.iter().map(corner).collect()
}

/// Returns the blade of `stations` standing at `x` metres from the middle.
pub fn blade(x: f32, stations: &[[f32; 4]]) -> Solid {
    let sections: Vec<Section> = stations.iter().map(|&at| section(x, at)).collect();
    let mut blade = Solid::default();
    solid(&mut blade, &sections, |_, _| 0);
    blade
}

/// Returns the corner points of the blade of `stations` at `x`, for its collider.
pub fn corners(x: f32, stations: &[[f32; 4]]) -> Vec<Vec3> {
    stations.iter().flat_map(|&at| section(x, at)).collect()
}
