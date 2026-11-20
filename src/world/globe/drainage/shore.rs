//! The shores near mild latitudes: where a new game starts, taken from the drainage grid's land cells and
//! refined on the real ground.

use std::f64::consts::FRAC_PI_2;

use bevy::math::DVec3;

use super::Drainage;
use super::tune::{APART, BISECT, LATITUDE, MOUTH, STARTS, TRIES};
use crate::world::globe::Globe;
use crate::world::globe::roll::roll;

/// Returns the cells of land at a mild latitude, the best first: the shore before the inland, a river mouth
/// before a plain shore, and among equals the order the `seed` rolls.
fn ranked(drain: &Drainage, seed: u64) -> Vec<usize> {
    let mild = |cell: usize| {
        let degrees = drain.dir(cell).y.asin().to_degrees().abs();
        (LATITUDE.0..=LATITUDE.1).contains(&degrees)
    };
    let score = |cell: usize| {
        let coast = drain.sea_beside(cell).is_some();
        let mouth = coast && drain.is_river(cell);
        let rolled = roll(seed, [cell as i64, 0, 0], 0);
        rolled * if mouth { 1.0 - MOUTH } else { 1.0 } + if coast { 0.0 } else { 1.0 }
    };
    let mut cells: Vec<(f32, usize)> = (0..drain.grid.len())
        .filter(|&cell| drain.is_land(cell) && mild(cell))
        .map(|cell| (score(cell), cell))
        .collect();
    cells.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    cells.into_iter().map(|(_, cell)| cell).collect()
}

/// Returns the point of the real ground on the shore of drainage cell `cell`, on the land side: the land
/// middle of the cell and the sea middle beside it narrowed down by bisection on the coarse ground. A cell
/// that is not land on the real ground, and a coast cell whose sea is land on the real ground, give nothing:
/// the local ground draws the coast, and it can differ from the grid by whole hills. An inland cell gives
/// its middle.
fn shore(globe: &Globe, drain: &Drainage, cell: usize) -> Option<DVec3> {
    let land = |dir: DVec3| globe.coarse(dir) > globe.sea_level();
    let at = drain.dir(cell);
    let Some(sea) = drain.sea_beside(cell).map(|k| drain.dir(k)) else {
        return land(at).then_some(at);
    };
    let (mut on, mut off) = match (land(at), land(sea)) {
        (true, false) => (at, sea),
        (false, true) => (sea, at),
        _ => return None,
    };
    for _ in 0..BISECT {
        let mid = (on + off).normalize();
        match land(mid) {
            true => on = mid,
            false => off = mid,
        }
    }
    Some(on)
}

impl Globe {
    /// Returns the unit directions of up to [`STARTS`] places a new game may start at, the best first: shores
    /// of land at a mild latitude, far apart, taken from the drainage grid. A dry or a groundless planet
    /// gives none or inland ones. The same planet always gives the same places.
    pub fn landfalls(&self) -> Vec<DVec3> {
        if !self.has_ground() {
            return Vec::new();
        }
        let drain = self.drainage();
        drain.starts.get_or_init(|| picked(self, &drain)).clone()
    }
}

/// Returns the places a new game may start at on `globe`, see [`Globe::landfalls`].
fn picked(globe: &Globe, drain: &Drainage) -> Vec<DVec3> {
    let apart = APART * FRAC_PI_2 / f64::from(drain.grid.n);
    let mut picked: Vec<(usize, DVec3)> = Vec::new();
    let mut tried = 0;
    for cell in ranked(drain, globe.seed()) {
        let far =
            |(other, _): &(usize, DVec3)| drain.dir(*other).angle_between(drain.dir(cell)) > apart;
        if picked.len() == STARTS || tried == TRIES {
            break;
        }
        if picked.iter().all(far) {
            tried += 1;
            picked.extend(shore(globe, drain, cell).map(|dir| (cell, dir)));
        }
    }
    picked.into_iter().map(|(_, dir)| dir).collect()
}
