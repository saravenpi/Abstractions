//! Villages: a plaza, bending streets, varied houses on foundation platforms, props and fields.

mod caps;
mod center;
mod detail;
mod dressing;
mod fields;
mod floors;
mod houses;
mod houses2;
mod houses3;
mod interior;
mod kinds;
mod layout;
mod lots;
mod paths;
mod place;
mod props;
mod shell;
mod streets;
pub mod style;
mod sundial;
#[cfg(test)]
mod testing;
mod trace;

use bevy::prelude::*;

use super::brush::Build;
use crate::world::Terrain;
use crate::world::structures::geo::transfer;
use crate::world::structures::site::Site;

pub use floors::floors;
#[cfg(test)]
pub use lots::{House, Lot};
pub use place::{REACH, SPAN, near_village, village_in_cell};
#[cfg(test)]
pub use testing::{ALL_HOUSES, draw_house, draw_lot, draw_well, ground, houses, plots, probe};

/// Draws the whole village of the current site into the build.
pub fn draw(b: &mut Build) {
    let lay = layout::build(b.terrain, b.site);
    center::draw(b, lay.centre, &lay.style);
    streets::draw(b, &lay);
    for lot in &lay.lots {
        houses::draw(b, &lay.style, lot);
    }
    for (i, field) in lay.fields.iter().enumerate() {
        fields::draw(b, field, i);
    }
}

/// Returns the points of the chart of `terrain` a village covers: center, lots, street and footpath samples
/// every 8 m, ends and fields. The village is laid out on the metric view around its center.
pub fn points(terrain: &Terrain, site: Site) -> Vec<Vec2> {
    let view = terrain.around(site.origin.x, site.origin.y);
    let lay = layout::build(&view, site);
    let mut out = vec![site.origin];
    out.extend(lay.lots.iter().map(|l| l.pos));
    out.extend(
        lay.streets
            .iter()
            .flat_map(|s| s.iter().step_by(4).copied()),
    );
    out.extend(lay.ends.iter().map(|e| e.pos));
    out.extend(lay.fields.iter().map(|f| f.pos));
    out.extend(paths::samples(&lay.paths));
    out.into_iter()
        .map(|p| transfer(&view, terrain, p))
        .collect()
}
