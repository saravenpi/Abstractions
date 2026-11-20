//! Trunk flows: the rivers of the drainage grid, followed on the ground by flows that spring 2 km apart
//! along each trunk, so that a river runs from its source to the sea whatever the length of a flow.

use bevy::math::DVec3;

use super::Cell;
use super::cell::STEPS;
use super::channel::Flowing;
use super::trace::{Flow, trace};
use crate::world::SEA_LEVEL;
use crate::world::globe::Globe;
use crate::world::globe::drainage::tune::STATION;

/// Returns the springs of the trunk from drainage cell `from` to its next cell `to`, each with that next
/// cell's direction as its goal: one every [`STATION`] metres along the straight line between their middles.
fn stations(globe: &Globe, from: DVec3, to: DVec3) -> Vec<(DVec3, DVec3)> {
    let count = ((from.angle_between(to) * globe.radius()) / STATION)
        .ceil()
        .max(1.0) as u32;
    (0..count)
        .map(|k| {
            (
                from.lerp(to, (f64::from(k) + 0.5) / f64::from(count))
                    .normalize(),
                to,
            )
        })
        .collect()
}

/// Returns the trunk flows that spring in `cell`: on a planet with plates and a sea of water, the stations
/// of every trunk of the drainage grid around it that fall in `cell` and on ground above the sea.
pub fn trunks(globe: &Globe, cell: Cell, kind: Flowing) -> Vec<Flow> {
    let n = globe.cells();
    let middle = cell.dir([STEPS as f64 * 0.5; 2], n);
    if kind != Flowing::Water || globe.large_height(middle).is_none() {
        return Vec::new();
    }
    let drain = globe.drainage();
    let here = drain.find(middle);
    let mut near: Vec<usize> = std::iter::once(here)
        .chain(drain.grid().around(here))
        .collect();
    near.sort_unstable();
    near.dedup();
    near.into_iter()
        .filter(|&k| drain.is_river(k))
        .filter_map(|k| Some((drain.dir(k), drain.dir(drain.next(k)?))))
        .flat_map(|(from, to)| stations(globe, from, to))
        .filter(|(dir, _)| Cell::locate(*dir, n).0 == cell && globe.coarse(*dir) > SEA_LEVEL)
        .filter_map(|(dir, goal)| trace(globe, dir, kind, Some(goal)))
        .collect()
}
