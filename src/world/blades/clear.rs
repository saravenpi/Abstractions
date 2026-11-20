//! The ground blades leave bare: around the trunks, stems and rocks of the big props, and on the patches
//! grazers cropped.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Cropped;
use super::tune::{ROOM, TILE};
use crate::world::CHUNK_SIZE;
use crate::world::canopy::Canopy;
use crate::world::chunks::Sited;

/// Returns true when planet position `p` stands on ground a prop of `near` or a cropped patch of `cropped`
/// takes: within the footprint of a trunk, a stem or a rock, or within a grazed patch.
pub fn taken(p: DVec3, near: &[(DVec3, f32)], cropped: &Cropped, radius: f64) -> bool {
    let up = p.normalize();
    near.iter()
        .any(|(at, foot)| at.distance(p) < f64::from(foot + ROOM))
        || cropped
            .0
            .iter()
            .any(|(dir, r)| dir.distance(up) * radius < f64::from(*r))
}

/// Returns the footprints of `near` that can take ground inside `sphere`, a centre and a radius in metres
/// that hold every blade of a tile: the others are farther from every blade than their own reach.
pub fn reaching(near: &[(DVec3, f32)], (centre, radius): (DVec3, f64)) -> Vec<(DVec3, f32)> {
    near.iter()
        .filter(|(at, foot)| at.distance(centre) < radius + f64::from(foot + ROOM))
        .copied()
        .collect()
}

/// Returns the footprints of the big props of the chunks within reach of the tile at planet position
/// `centre`, as planet positions and radii, into `out`.
pub fn footprints(chunks: &Query<(&Sited, &Canopy)>, centre: DVec3, out: &mut Vec<(DVec3, f32)>) {
    out.clear();
    let reach = f64::from(CHUNK_SIZE + TILE);
    for (sited, canopy) in chunks
        .iter()
        .filter(|(s, _)| s.pos.distance(centre) < reach)
    {
        let feet = canopy.0.iter().filter(|c| c.foot > 0.0);
        out.extend(feet.map(|c| (sited.pos + sited.rot * c.at.as_dvec3(), c.foot)));
    }
}
