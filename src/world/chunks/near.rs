//! Chunk coordinates around the focus: every leaf seen through the chart of the focus face, and
//! the lane the focus follows through them.

use std::f64::consts::FRAC_PI_2;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::lane::Lane;
use super::pace::Pace;
use super::plan::needs_collider;
use super::sight::Height;
use crate::world::globe::quadtree::leaf_at;
use crate::world::globe::{Chart, ChunkId};
use crate::world::mesh::leaf_coord;
use crate::world::{CHUNK_SIZE, Globe, Terrain};

/// Where the focus is heading and how much detail the streamer builds on the way.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Course {
    /// Where the focus will be in a moment, in metres from the planet centre.
    pub ahead: DVec3,
    /// Detail radius in chunks of the render distance.
    pub rings: i32,
    /// What the speed of the focus leaves time for.
    pub pace: Pace,
}

/// The chart of the focus face, the chunk the focus is in and the lane it follows.
///
/// Leaves of other faces get the coordinate of their middle in this chart, continued past its
/// edge, so rings around the focus also run across a face edge. Near a cube corner two leaves
/// may share a coordinate and a coordinate may hold no leaf.
pub struct Around {
    terrain: Terrain,
    /// Chunk that holds the focus.
    pub center: IVec2,
    /// The leaf under the focus and the leaves that touch it, across face edges too.
    pub near: Vec<ChunkId>,
    /// Rings added to every chunk for the height of the focus above the ground, one per chunk
    /// width: seen from the air everything is farther.
    pub lift: i32,
    /// The chunks wanted from the focus to the point it will reach.
    pub lane: Lane,
    /// Chunks past the lane a loaded chunk is kept: one ring, and what a fast pace took off
    /// the radius, so nothing loaded vanishes for speed alone.
    keep: i32,
    /// What the speed of the focus leaves time for.
    pub pace: Pace,
}

impl Around {
    /// Returns the coordinates around a focus that stands still at planet position `focus`,
    /// seen through the face chart of `terrain`. Other charts have no leaves and give nothing.
    pub fn new(terrain: &Terrain, focus: DVec3) -> Option<Self> {
        let still = Course {
            ahead: focus,
            rings: 0,
            pace: Pace::Calm,
        };
        Self::along(terrain, focus, &still)
    }

    /// Returns the coordinates around a focus at planet position `focus` that follows `course`,
    /// seen through the face chart of `terrain`. Other charts have no leaves and give nothing.
    pub fn along(terrain: &Terrain, focus: DVec3, course: &Course) -> Option<Self> {
        if !matches!(terrain.chart(), Chart::Face { .. }) {
            return None;
        }
        let chunks = |p: Vec3| Vec2::new(p.x, p.z) / CHUNK_SIZE;
        let at = chunks(terrain.locate(focus));
        let (center, lead) = (
            at.floor().as_ivec2(),
            chunks(terrain.locate(course.ahead)) - at,
        );
        let under = leaf_at(focus.normalize_or(DVec3::Y), terrain.globe().leaf_level());
        let steps = (-1..=1).flat_map(|dx| (-1..=1).map(move |dy| (dx, dy)));
        let height = Height::of(terrain.globe(), focus);
        let radius = course.pace.rings(course.rings);
        Some(Self {
            lift: (height.clearance / leaf_arc(terrain.globe()))
                .floor()
                .clamp(0.0, 1.0e6) as i32,
            terrain: terrain.clone(),
            center,
            near: steps
                .filter_map(|(dx, dy)| under.neighbor(dx, dy))
                .collect(),
            lane: Lane::new(center, lead, radius),
            keep: course.rings - radius + 1,
            pace: course.pace,
        })
    }

    /// Returns the chunk coordinate of leaf `id` in the chart of the focus face.
    pub fn coord(&self, id: ChunkId) -> IVec2 {
        match self.terrain.chart() {
            Chart::Face { face, .. } if face == id.face => leaf_coord(id),
            chart => (chart.locate(id.dir(0.5, 0.5)) / CHUNK_SIZE)
                .floor()
                .as_ivec2(),
        }
    }

    /// Returns how far leaf `id` is from the line of flight in rings of chunks, the height of
    /// the focus above the ground included.
    pub fn rank(&self, id: ChunkId) -> i32 {
        self.lane.rank(self.coord(id)) + self.lift
    }

    /// Returns true when leaf `id` must have colliders: it is the leaf under the focus, touches
    /// it, or lies within the collider rings in the chart of the focus face.
    pub fn solid(&self, id: ChunkId) -> bool {
        self.near.contains(&id) || needs_collider(self.coord(id), self.center)
    }

    /// Returns true while the loaded chunk of leaf `id` stays: it is within a ring of the lane
    /// at the full radius of the render distance.
    pub fn keeps(&self, id: ChunkId) -> bool {
        self.lane.holds(self.coord(id), self.keep)
    }
}

/// Returns the width in metres of a detail chunk in the middle of a cube face of `globe`.
pub fn leaf_arc(globe: &Globe) -> f64 {
    globe.radius() * FRAC_PI_2 / f64::from(1u32 << globe.leaf_level())
}
