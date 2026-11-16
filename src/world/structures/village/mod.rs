//! Villages: a plaza, bending streets, varied houses on foundation platforms, props and fields.

mod center;
mod detail;
mod dressing;
mod fields;
mod houses;
mod houses2;
mod houses3;
mod kinds;
mod layout;
mod lots;
mod place;
mod props;
mod shell;
mod streets;
pub mod style;
mod trace;

use bevy::prelude::*;

use super::brush::Build;
use crate::world::Terrain;
use crate::world::structures::site::Site;

#[cfg(test)]
pub use lots::House;
pub use place::{REACH, SPAN, near_village, village_in_cell};

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

/// Returns the world points a village covers: center, lots, street samples every 8 m, ends and fields.
pub fn points(terrain: &Terrain, site: Site) -> Vec<Vec2> {
    let lay = layout::build(terrain, site);
    let mut out = vec![site.origin];
    out.extend(lay.lots.iter().map(|l| l.pos));
    out.extend(
        lay.streets
            .iter()
            .flat_map(|s| s.iter().step_by(4).copied()),
    );
    out.extend(lay.ends.iter().map(|e| e.pos));
    out.extend(lay.fields.iter().map(|f| f.pos));
    out
}

/// Returns the house kind and position of every lot.
#[cfg(test)]
pub fn houses(terrain: &Terrain, site: Site) -> Vec<(House, Vec2)> {
    layout::build(terrain, site)
        .lots
        .iter()
        .map(|l| (l.house, l.pos))
        .collect()
}

/// Draws one house of `house` kind in the style of `biome`, on the current site.
#[cfg(test)]
pub fn draw_house(b: &mut Build, biome: crate::world::Biome, house: House) {
    let lot = lots::Lot {
        pos: b.site.origin,
        yaw: 0.3,
        floor: b.site.floor,
        depth: 2.0,
        house,
        variant: 6,
    };
    houses::draw(b, &style::style_for(biome, 6), &lot);
}

/// Every house kind.
#[cfg(test)]
pub const ALL_HOUSES: [House; 10] = [
    House::Cottage,
    House::Long,
    House::TwoStory,
    House::Porch,
    House::Smithy,
    House::Barn,
    House::Tower,
    House::Stall,
    House::Shed,
    House::Hall,
];
