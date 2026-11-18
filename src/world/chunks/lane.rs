//! The lane: the detail chunks wanted around the stretch from the focus to the point it will
//! reach, and the order they are built in. A focus that stands still has a plain disc.

use bevy::prelude::*;

use crate::world::mesh::ring_distance;

/// How much of its distance along the path a chunk ahead adds to its cost: a chunk four chunks
/// ahead on the path is built before one three chunks to the side.
const ALONG: f32 = 0.5;

/// How many times farther a chunk behind a moving focus counts than one beside it.
const BEHIND: f32 = 2.0;

/// Share of the lead by which the disc behind a moving focus is no longer filled: a chunk that
/// will be out of range in half a second is not worth starting.
const WAKE: f32 = 0.25;

/// Steps per chunk the lead is rounded to, so a steady flight does not change the lane every
/// frame.
const GRAIN: f32 = 4.0;

/// The chunks wanted around a moving focus, in chunk coordinates of the chart of its face.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lane {
    /// Chunk that holds the focus.
    pub from: IVec2,
    /// From the focus to the point it will reach, in chunks.
    pub lead: Vec2,
    /// Detail radius in chunks around the stretch between them.
    pub radius: i32,
}

impl Lane {
    /// Returns the lane of a focus in chunk `from` that will be `lead` chunks away in a moment,
    /// with the detail `radius`. A lead of nothing gives the plain disc around `from`.
    pub fn new(from: IVec2, lead: Vec2, radius: i32) -> Self {
        Self {
            from,
            lead: (lead * GRAIN).round() / GRAIN,
            radius,
        }
    }

    /// Returns the offset of `cell` from the nearest point of the stretch, and how far along
    /// the stretch that point is, in chunks. The stretch runs on for `more` chunks past its end.
    fn nearest(&self, cell: IVec2, more: f32) -> (Vec2, f32) {
        let off = (cell - self.from).as_vec2();
        let length = self.lead.length();
        if length <= 0.0 {
            return (off, 0.0);
        }
        let along = (off.dot(self.lead) / length).clamp(0.0, length + more);
        (off - self.lead * (along / length), along)
    }

    /// Returns true when `cell` lies behind a focus that moves.
    fn behind(&self, cell: IVec2) -> bool {
        (cell - self.from).as_vec2().dot(self.lead) < 0.0
    }

    /// Returns true when `cell` lies within the radius plus `slack` chunks of the stretch.
    pub fn holds(&self, cell: IVec2, slack: i32) -> bool {
        let reach = (self.radius + slack) as f32;
        self.nearest(cell, 0.0).0.length_squared() <= reach * reach
    }

    /// Returns how far `cell` is from the line of flight in rings of chunks. The line runs on
    /// past the point ahead by as much as the lead, up to the radius: at speed a chunk ahead
    /// gets the level it will have when the focus flies by, and nothing changes for a focus
    /// that barely moves.
    pub fn rank(&self, cell: IVec2) -> i32 {
        let more = self.lead.length().min(self.radius as f32);
        let (off, _) = self.nearest(cell, more);
        off.abs().max_element().round() as i32
    }

    /// Returns the build cost of `cell` in chunks: least on the path just ahead of the focus,
    /// more to the sides and far ahead, most behind.
    pub fn cost(&self, cell: IVec2) -> f32 {
        let (off, along) = self.nearest(cell, 0.0);
        let behind = [1.0, BEHIND][usize::from(self.behind(cell))];
        (off.length() + ALONG * along) * behind
    }

    /// Returns true when `cell` is worth building: it is in the lane, and not in the part of
    /// the disc a moving focus is about to leave behind. The chunks that touch the focus
    /// always are.
    pub fn wants(&self, cell: IVec2) -> bool {
        let far = (cell - self.from).as_vec2().length();
        let wake = far > (self.radius + 1) as f32 - self.lead.length() * WAKE;
        let touches = ring_distance(cell, self.from) <= 1;
        self.holds(cell, 0) && (touches || !(self.behind(cell) && wake))
    }

    /// Returns every chunk worth building, cheapest first.
    pub fn cells(&self) -> Vec<IVec2> {
        let to = self.from.as_vec2() + self.lead;
        let low = self.from.min(to.floor().as_ivec2());
        let high = self.from.max(to.ceil().as_ivec2());
        let span = |a: i32, b: i32| a - self.radius..=b + self.radius;
        let mut all: Vec<IVec2> = span(low.x, high.x)
            .flat_map(|x| span(low.y, high.y).map(move |y| IVec2::new(x, y)))
            .filter(|cell| self.wants(*cell))
            .collect();
        all.sort_by(|a, b| {
            let order = self.cost(*a).total_cmp(&self.cost(*b));
            order.then_with(|| (a.x, a.y).cmp(&(b.x, b.y)))
        });
        all
    }
}
