//! Standing blades: the tail fin and the fins at the wing tips. A blade is a thin section,
//! pointed at the front and cut off at the back, lofted up through a few stations.

use bevy::prelude::*;

use super::loft::{Section, solid};
use crate::ship::solid::Solid;

/// Stations of the tail fin from its root up: height, z of the leading edge and of the trailing
/// edge, and half thickness. The root is inside the fuselage and the spine, and thick, so that
/// the fin grows out of the back. The leading edge is swept 56 degrees and cranked, and the tip,
/// 2 m up, is raked back past the nozzles.
pub(super) const FIN: [[f32; 4]; 4] = [
    [0.45, 1.1, 3.05, 0.16],
    [0.85, 1.55, 3.4, 0.09],
    [1.35, 2.35, 3.62, 0.06],
    [2.0, 3.25, 3.95, 0.035],
];

/// Stations of the blade at a wing tip, like [`FIN`]: it starts under the wing end, holds it,
/// and stands 0.66 m above the top of it, well over the 0.35 m a body steps up, so it stops a
/// body on the wing.
pub(super) const WINGLET: [[f32; 4]; 3] = [
    [-0.36, 1.62, 2.4, 0.04],
    [-0.1, 1.58, 2.5, 0.055],
    [0.5, 2.25, 2.85, 0.03],
];

/// The section of a blade: the share of the chord from the leading edge, and the share of the
/// half thickness across, of each corner. The faces are flat over most of the chord, so that a
/// body that walks into a blade meets a wall, which it cannot slide along and round the tip.
const SECTION: [(f32, f32); 7] = [
    (0.0, 0.0),
    (0.2, 1.0),
    (0.8, 1.0),
    (1.0, 0.3),
    (1.0, -0.3),
    (0.8, -1.0),
    (0.2, -1.0),
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

/// How far a band stops short of the leading and of the trailing edge of its blade, in metres.
const SHORT: f32 = 0.05;

/// Returns a band on the two faces of the blade of `stations` at `x`, from the height `from` up
/// to `to`: a sleeve that stands `out` metres off the faces and stops short of the edges, for a
/// stripe.
pub fn band(x: f32, stations: &[[f32; 4]], (from, to): (f32, f32), out: f32) -> Solid {
    let at = |y: f32| {
        let i = stations
            .iter()
            .rposition(|s| s[0] <= y)
            .unwrap_or(0)
            .min(stations.len() - 2);
        let (a, b) = (stations[i], stations[i + 1]);
        let t = (y - a[0]) / (b[0] - a[0]);
        let [_, front, back, half] = std::array::from_fn(|k| a[k] + (b[k] - a[k]) * t);
        section(x, [y, front + SHORT, back - SHORT, half + out])
    };
    let mut band = Solid::default();
    solid(&mut band, &[at(from), at(to)], |_, _| 0);
    band
}
