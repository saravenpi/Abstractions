//! The view a selection is made for: where the focus is and how far detail must reach.

use std::f64::consts::FRAC_PI_2;

use bevy::math::DVec3;

use super::{MERGE, SPLIT};
use crate::world::globe::{ChunkId, Face};

/// Share of its width that a node reaches from its middle towards a corner, rounded up.
const HALF_DIAGONAL: f64 = 0.75;

/// What the quadtree is selected for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    /// Focus position in metres from the planet centre, in the planet frame.
    pub focus: DVec3,
    /// Sea level radius of the planet in metres.
    pub radius: f64,
    /// Deepest level a node may have.
    pub deepest: u8,
    /// Distance from the focus, in metres, within which every node is split down to `deepest`.
    pub reach: f64,
    /// Most splits one selection may ask for.
    pub budget: usize,
    /// How many times its width away a node splits: [`SPLIT`] at the default render distance.
    pub split: f64,
}

/// Returns the node of `level` that holds unit direction `dir`.
pub fn leaf_at(dir: DVec3, level: u8) -> ChunkId {
    let (face, u, v) = Face::locate(dir);
    let n = (1u64 << level) as f64;
    let cell = |c: f64| ((c + 1.0) * 0.5 * n).floor().clamp(0.0, n - 1.0) as u32;
    ChunkId {
        face,
        level,
        x: cell(u),
        y: cell(v),
    }
}

impl View {
    /// Returns the width in metres of a node of `level`, measured through the face centre.
    pub fn arc(&self, level: u8) -> f64 {
        self.radius * FRAC_PI_2 / (1u64 << level) as f64
    }

    /// Returns the distance in metres from the focus to the middle of `id` at sea level.
    pub fn distance(&self, id: ChunkId) -> f64 {
        (id.dir(0.5, 0.5) * self.radius).distance(self.focus)
    }

    /// Returns the distance under which `id` splits.
    fn range(&self, id: ChunkId) -> f64 {
        let arc = self.arc(id.level);
        (arc * self.split).max(self.reach + arc * HALF_DIAGONAL)
    }

    /// Returns true when the leaf `id` should split.
    pub fn splits(&self, id: ChunkId) -> bool {
        id.level < self.deepest && self.distance(id) < self.range(id)
    }

    /// Returns true when the split node `id` should merge.
    pub fn merges(&self, id: ChunkId) -> bool {
        self.distance(id) > self.range(id) * (MERGE / SPLIT)
    }
}
