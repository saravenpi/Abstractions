//! The built floors of a village: house floors and platforms, the plaza, the streets and the footpaths.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::kinds::House;
use super::layout::{self, Layout, PLAZA};
use super::lots::{Lot, ROAD_HALF};
use super::style::Flavor;
use crate::world::Terrain;
use crate::world::mesh::Frame;
use crate::world::structures::pads::{FloorKind, Pad, Slab, Strip};
use crate::world::structures::site::Site;

/// Half width of a footpath in meters.
const PATH_HALF: f32 = 0.45;

/// Height of the upper floor of the two-storey house above its ground floor.
const UPPER: f32 = 2.7;

/// Returns what the platforms and what the streets of a village of `flavor` are made of.
fn materials(flavor: Flavor) -> (FloorKind, FloorKind) {
    match flavor {
        Flavor::Timber => (FloorKind::Stone, FloorKind::Earth),
        Flavor::Taiga | Flavor::Cherry => (FloorKind::Stone, FloorKind::Stone),
        Flavor::Adobe => (FloorKind::Sand, FloorKind::Sand),
        Flavor::Stilt => (FloorKind::Wood, FloorKind::Wood),
    }
}

/// Returns the floors of the house on `lot`: the planks inside the walls, on both storeys where there are
/// two, then the platform around them.
fn house(view: &Terrain, lot: &Lot, base: FloorKind) -> Vec<Pad> {
    let s = lot.house.shell();
    let (inner, at) = (s.inner(), (lot.pos, lot.yaw, lot.floor));
    let planks = |y: f32| -> Slab {
        (
            false,
            (-inner).into(),
            inner.into(),
            [y; 2],
            FloorKind::Wood,
        )
    };
    let (x, front) = (s.hx + 0.5, s.hz + s.front + 0.5);
    let platform: Slab = (false, [-x, -s.hz - 0.5], [x, front], [0.0; 2], base);
    let mut slabs = Vec::new();
    if lot.house != House::Stall {
        slabs.push(planks(0.0));
    }
    if lot.house == House::TwoStory {
        slabs.push(planks(UPPER));
    }
    slabs.push(platform);
    slabs.iter().map(|slab| Pad::new(view, at, slab)).collect()
}

/// Returns the strips of the streets and footpaths of `lay`, each on the ground of `view`.
fn strips(view: &Terrain, lay: &Layout, kind: FloorKind) -> Vec<Strip> {
    let ground = |p: Vec2| view.embed(p.x, view.height(p.x, p.y), p.y);
    let strip = |(a, c): (Vec2, Vec2), half: f32| Strip {
        from: ground(a),
        to: ground(c),
        up: view.up(a.x, a.y),
        half,
        kind,
    };
    let roads = lay.streets.iter().flat_map(|s| s.windows(2));
    let roads = roads.map(|w| strip((w[0], w[1]), ROAD_HALF));
    let paths = lay.paths.iter().map(|seg| strip(*seg, PATH_HALF));
    roads.chain(paths).collect()
}

/// Returns the built floors of the village at `site`, laid out on `view`, the metric view around its center:
/// the pads of every house and of the plaza, and the strips of the streets and footpaths.
pub fn floors(view: &Terrain, site: &Site) -> (Vec<Pad>, Vec<Strip>) {
    let lay = layout::build(view, *site);
    let (base, road) = materials(lay.style.flavor);
    let mut pads: Vec<Pad> = lay
        .lots
        .iter()
        .flat_map(|lot| house(view, lot, base))
        .collect();
    let plaza: Slab = (true, [0.0; 2], [PLAZA, 0.0], [-1.5, 1.5], road);
    pads.push(Pad::new(view, (site.origin, site.yaw, site.floor), &plaza));
    (pads, strips(view, &lay, road))
}

/// A walled house of a village in planet positions: the middle of its ground floor, the middle
/// of its doorway on the threshold, and the unit direction out through the door.
pub type Entry = (DVec3, DVec3, DVec3);

/// Returns every walled house of the village at `site`, laid out on `view`, the metric view
/// around its center: what a tour or a test needs to stand at a door or under a roof.
pub fn entries(view: &Terrain, site: &Site) -> Vec<Entry> {
    let lay = layout::build(view, *site);
    let walled = lay.lots.into_iter().filter(|lot| lot.house != House::Stall);
    walled
        .map(|lot| {
            let s = lot.house.shell();
            let frame = Frame::at(view, lot.pos);
            let rot = frame.rot * DQuat::from_rotation_y(f64::from(lot.yaw));
            let at = |x: f32, z: f32| frame.origin + rot * Vec3::new(x, lot.floor, z).as_dvec3();
            (at(0.0, 0.0), at(s.door, s.hz), rot * DVec3::Z)
        })
        .collect()
}
