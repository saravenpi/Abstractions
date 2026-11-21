//! The ribs of the cabin: frames that stand against the walls of the tub at even steps along
//! the mouth, from the floor up to just under the lip. They follow the width of the tub and
//! the height of the roof at their station, so they follow the hull wherever it is drawn.

use bevy::prelude::*;

use crate::ship::cockpit::FLOOR;
use crate::ship::mesh::shell::{HALF, MOUTH, lip, mouth, station};
use crate::ship::solid::Solid;

/// Number of ribs along each wall.
const COUNT: usize = 4;

/// Thickness of a rib along the hull, how far it stands out of the wall, and how far it goes
/// into it, in metres.
const RIB: (f32, f32, f32) = (0.05, 0.04, 0.012);

/// How far under the lip the top of a rib stops, in metres.
const BELOW: f32 = 0.02;

/// Returns the half width of the tub at `z`, between the widths of the mouth.
fn half_at(z: f32) -> f32 {
    let stations = mouth();
    let i = usize::from(z > stations[1][0]);
    let t = (z - stations[i][0]) / (stations[i + 1][0] - stations[i][0]);
    HALF[i] + (HALF[i + 1] - HALF[i]) * t
}

/// Returns the ribs of both walls, `COUNT` on each, at even steps between the ends of the mouth.
pub fn ribs() -> Solid {
    let mut ribs = Solid::default();
    for i in 1..=COUNT {
        let z = MOUTH.0 + (MOUTH.1 - MOUTH.0) * i as f32 / (COUNT + 1) as f32;
        let half = half_at(z);
        let top = lip(station(z), half).y - BELOW;
        ribs.slab(
            Vec3::new(half - RIB.1, FLOOR, z - RIB.0 * 0.5),
            Vec3::new(half + RIB.2, top, z + RIB.0 * 0.5),
        );
    }
    ribs.mirrored()
}
