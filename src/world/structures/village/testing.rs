//! Test-only entry points into a village: its ground pieces, its lots and single houses drawn apart.

use bevy::prelude::*;

use super::lots::{House, Lot};
use super::{center, fields, houses as dwellings, layout, lots, streets, style};
use crate::world::Terrain;
use crate::world::mesh::Frame;
use crate::world::structures::audit::positions;
use crate::world::structures::brush::Build;
use crate::world::structures::carry::Target;
use crate::world::structures::site::Site;

/// What the ground tests measure on one village, in world space.
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
pub fn probe(terrain: &Terrain, site: Site) -> Probe {
    let lay = layout::build(terrain, site);
    let fresh = || Build::new(terrain, site.origin, site);
    let world = |b: &Build| -> Vec<Vec3> {
        let verts = positions(&b.mesh.clone().into_mesh());
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
pub fn houses(terrain: &Terrain, site: Site) -> Vec<(House, Vec2)> {
    layout::build(terrain, site)
        .lots
        .iter()
        .map(|l| (l.house, l.pos))
        .collect()
}

/// Draws one house of `house` kind in the style of `biome`, on the current site.
pub fn draw_house(b: &mut Build, biome: crate::world::Biome, house: House) {
    let lot = lots::Lot {
        pos: b.site.origin,
        yaw: 0.3,
        floor: b.site.floor,
        depth: 2.0,
        house,
        variant: 6,
    };
    dwellings::draw(b, &style::style_for(biome, 6), &lot);
}

/// Draws a plaza well in the style of `biome` on the current site.
pub fn draw_well(b: &mut Build, biome: crate::world::Biome) {
    center::draw(b, layout::Centre::Well, &style::style_for(biome, 6));
}

/// Every house kind.
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

/// Returns the lots of the village at `site`, laid out on `terrain`.
pub fn plots(terrain: &Terrain, site: Site) -> Vec<Lot> {
    layout::build(terrain, site).lots
}

/// Draws the house of `lot` in the style of the village of the current site.
pub fn draw_lot(b: &mut Build, lot: &Lot) {
    let at = b.site.origin;
    let look = style::style_for(b.terrain.biome(at.x, at.y), b.site.variant);
    dwellings::draw(b, &look, lot);
}

/// Draws the plaza, streets and footpaths of the village at `site` on `charts`, the chart of the chunk then
/// the metric view around the village, and returns their vertices in `frame`.
pub fn ground(charts: (&Terrain, &Terrain), site: Site, frame: &Frame) -> Vec<Vec3> {
    let mut b = Build::on(charts, site.origin, site);
    streets::surface(&mut b, &layout::build(charts.1, site));
    positions(&b.finish(Target::Frame(*frame)).mesh)
}
