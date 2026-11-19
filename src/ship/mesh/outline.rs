//! The airframe as a few convex parts, each given by its corner points in ship-root space: the
//! fuselage, the canopy, the fin, and the wing and the winglet of each side. They come from the
//! same outlines the drawn parts are made of, so what the ship collides with is what is seen.

use bevy::prelude::*;

use super::frame::{
    CANOPY, CANOPY_ENDS, CANOPY_SIDES, FIN, FIN_DEPTH, FUSELAGE, SIDES, SPINE, plates, ring,
};

/// Returns the corner points of each convex part of the airframe in ship-root space: the
/// fuselage from the nose tip to the tail, the canopy bubble, the fin, then the wing and the
/// winglet of the left side and of the right side.
pub fn hulls() -> Vec<Vec<Vec3>> {
    let mut parts = vec![
        lathed(&FUSELAGE, (SPINE[0], SPINE[2]), SIDES),
        lathed(&CANOPY, CANOPY_ENDS, CANOPY_SIDES),
        corners((FIN, FIN_DEPTH)),
    ];
    for side in [-1.0, 1.0] {
        parts.extend(plates(side).map(corners));
    }
    parts
}

/// Returns the eight corners of a plate: its flat outline `base` and the same moved by `depth`.
fn corners((base, depth): ([Vec3; 4], Vec3)) -> Vec<Vec3> {
    base.into_iter().chain(base.map(|p| p + depth)).collect()
}

/// Returns the corners of a body lathed through `stations` with `sides` sides, and its two `ends`.
fn lathed(stations: &[[f32; 4]], ends: (Vec3, Vec3), sides: u32) -> Vec<Vec3> {
    let rings = stations
        .iter()
        .flat_map(|&station| (0..sides).map(move |i| ring(station, i, sides)));
    rings.chain([ends.0, ends.1]).collect()
}
