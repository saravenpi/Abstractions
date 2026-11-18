//! What a flyover still misses: ground that is not yet as detailed as it would be had the focus
//! stood still where it is, as far as its pace leaves time to build detail.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::super::Chunks;
use super::super::cut::visible;
use super::super::near::Around;
use super::super::pace::Pace;
use super::super::plan::ring;
use super::super::reach::Reach;
use super::super::sight::{Height, veil};
use crate::world::globe::ChunkId;
use crate::world::globe::quadtree::{View, leaf_at, parent};
use crate::world::{CHUNK_SIZE, Terrain};

/// Share of its width a node reaches from its middle, as the quadtree view rounds it.
const HALF_DIAGONAL: f64 = 0.75;

/// A piece of ground that is not settled yet.
#[derive(Debug, Clone, Copy)]
pub struct Gap {
    /// The node or leaf that lacks something.
    pub id: ChunkId,
    /// True when only the finest terrain level or the ground cover is missing.
    pub fine: bool,
}

/// Where the flyover looks from.
pub struct Eye<'a> {
    /// Focus position in the planet frame.
    pub focus: DVec3,
    /// Unit direction of travel.
    pub heading: DVec3,
    /// Height of the focus over the ground or the sea under it.
    pub clearance: f64,
    /// The view of the planet through the chart of the focus face.
    pub terrain: &'a Terrain,
    /// The render distance.
    pub reach: Reach,
    /// What the speed of the focus leaves time to build.
    pub pace: Pace,
}

impl Eye<'_> {
    /// Returns the quadtree view of a focus that stands still here, with the detail radius its
    /// pace leaves.
    pub fn view(&self) -> View {
        let globe = self.terrain.globe();
        let mut view = View {
            focus: self.focus,
            radius: globe.radius(),
            deepest: globe.leaf_level(),
            reach: 0.0,
            budget: 0,
            split: self.reach.split(),
        };
        let rings = self.pace.rings(self.reach.rings);
        view.reach = f64::from(rings + 1) * view.arc(view.deepest);
        view
    }

    /// Returns the distance in metres from the focus to the nearest part of `id`, about.
    pub fn distance(&self, id: ChunkId) -> f64 {
        let view = self.view();
        (view.distance(id) - view.arc(id.level) * HALF_DIAGONAL).max(0.0)
    }

    /// Returns true when `id` lies under the focus or within sixty degrees of its heading.
    pub fn ahead(&self, id: ChunkId) -> bool {
        let up = self.focus.normalize();
        let to = id.dir(0.5, 0.5) * self.terrain.globe().radius() - self.focus;
        let flat = to - up * to.dot(up);
        self.distance(id) <= 0.0 || flat.dot(self.heading) > 0.5 * flat.length()
    }

    /// Returns the distance in metres at which the fog hides everything from the focus.
    pub fn fog_end(&self) -> f64 {
        let globe = self.terrain.globe();
        let height = Height {
            altitude: self.focus.length() - globe.radius(),
            clearance: self.clearance,
            radius: globe.radius(),
        };
        veil(f64::from(self.reach.sight(globe)), &height)
    }
}

/// Returns the leaf a standing focus would have detail on at chunk coordinate `coord`.
fn leaf(eye: &Eye, coord: IVec2) -> ChunkId {
    let at = (coord.as_vec2() + 0.5) * CHUNK_SIZE;
    let dir = eye.terrain.embed(at.x, 0.0, at.y).normalize();
    leaf_at(dir, eye.terrain.globe().leaf_level())
}

/// Returns what leaf `id` lacks for a standing focus: its chunk, or only its fine detail. A
/// leaf the quadtree of a standing focus would not reach lacks nothing, and neither does one
/// that only lacks what the pace of the focus leaves no time to build.
fn lack(chunks: &Chunks, eye: &Eye, around: &Around, id: ChunkId) -> Option<Gap> {
    if !parent(id).is_some_and(|above| eye.view().splits(above)) {
        return None;
    }
    let Some(l) = chunks.loaded.get(&id).filter(|l| l.shown) else {
        return Some(Gap { id, fine: false });
    };
    let rank = around.rank(id);
    let rings = eye.reach.levels(eye.terrain.globe());
    let calm = eye.pace.covers();
    let coarse = calm && l.detail.lod > rings.lod(rank);
    let bald = calm && rank <= eye.reach.cover() && l.detail.turf.grown().is_none();
    (coarse || bald).then_some(Gap { id, fine: true })
}

/// Returns everything `chunks` lacks compared with a world settled around a standing focus.
pub fn gaps(chunks: &Chunks, eye: &Eye) -> Vec<Gap> {
    let view = eye.view();
    let cut = visible(&chunks.tree, &|id| chunks.nodes.has(id));
    let coarse = cut.into_iter().filter(|id| view.splits(*id));
    let mut all: Vec<Gap> = coarse.map(|id| Gap { id, fine: false }).collect();
    if let Some(around) = Around::new(eye.terrain, eye.focus) {
        let wanted = ring(around.center, eye.pace.rings(eye.reach.rings));
        let leaves = wanted.into_iter().map(|coord| leaf(eye, coord));
        all.extend(leaves.filter_map(|id| lack(chunks, eye, &around, id)));
    }
    all
}
