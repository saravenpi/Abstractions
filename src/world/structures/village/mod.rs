//! Villages: a plaza, bending streets, varied houses on foundation platforms, props and fields.

mod center;
mod detail;
mod dressing;
mod fields;
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

/// Returns the world points a village covers: center, lots, street and footpath samples every 8 m, ends and fields.
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
    out.extend(paths::samples(&lay.paths));
    out
}

/// What the ground tests measure on one village, in world space.
#[cfg(test)]
pub struct Probe {
    /// Leg feet of the benches a tree ring would have on this plaza.
    pub benches: Vec<Vec3>,
    /// Vertices of the plaza, the streets and the footpaths.
    pub roads: Vec<Vec3>,
    /// Lowest vertex of every column of the crop rows, hedges and fences.
    pub crops: Vec<Vec3>,
    /// For each door path, its distance to the nearest other lot minus the radius of that lot.
    pub doors: Vec<f32>,
}

/// Returns, for each door path, its distance to the nearest other lot minus the radius of that lot.
#[cfg(test)]
fn door_margins(lay: &layout::Layout) -> Vec<f32> {
    let margin = |i: usize, (a, c): (Vec2, Vec2)| {
        let others = lay.lots.iter().enumerate().filter(|(j, _)| *j != i);
        let free = others.map(|(_, l)| {
            let k = (l.pos - a).dot(c - a) / (c - a).length_squared().max(1e-6);
            a.lerp(c, k.clamp(0.0, 1.0)).distance(l.pos) - l.house.radius()
        });
        free.fold(f32::MAX, f32::min)
    };
    let doors = lay.paths[..lay.lots.len()].iter().enumerate();
    doors.map(|(i, seg)| margin(i, *seg)).collect()
}

/// Draws the ground-hugging pieces of the village at `site` apart and returns their world positions.
#[cfg(test)]
pub fn probe(terrain: &Terrain, site: Site) -> Probe {
    let lay = layout::build(terrain, site);
    let fresh = || Build::new(terrain, site.origin, site);
    let world = |b: &Build| -> Vec<Vec3> {
        let verts = super::audit::positions(&b.mesh.clone().into_mesh());
        let shift = Vec3::new(site.origin.x, 0.0, site.origin.y);
        verts.into_iter().map(|p| p + shift).collect()
    };
    let (mut seats, mut roads, mut crops) = (fresh(), fresh(), fresh());
    streets::surface(&mut roads, &lay);
    for f in &lay.fields {
        crops.stand(f.pos, f.yaw);
        fields::crops(&mut crops, f);
    }
    let mut low = world(&crops);
    low.sort_by(|a, b| {
        a.x.total_cmp(&b.x)
            .then(a.z.total_cmp(&b.z))
            .then(a.y.total_cmp(&b.y))
    });
    low.dedup_by(|a, b| a.xz() == b.xz());
    Probe {
        benches: center::bench_feet(&mut seats),
        roads: world(&roads),
        crops: low,
        doors: door_margins(&lay),
    }
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
