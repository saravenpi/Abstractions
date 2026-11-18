//! The stretch a selection is made for: from the focus to the point it will reach in a moment,
//! so the ground ahead is split before the focus is over it.

use bevy::math::DVec3;

use super::view::View;
use crate::world::globe::ChunkId;

/// How many times farther a node behind a moving focus counts than one beside it.
const BEHIND: f64 = 2.0;

/// A view and the point its focus is heading for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Path {
    /// What the quadtree is selected for at the focus.
    pub view: View,
    /// Where the focus will be in a moment, in metres from the planet centre.
    pub ahead: DVec3,
    /// Distance from the stretch, in metres, within which new ground splits down to the deepest
    /// level. It is the reach of the view, or less for a focus too fast to see the detail; what
    /// is split already merges by the reach of the view alone.
    pub reach: f64,
}

impl Path {
    /// Returns the path of a focus that stands still: a selection around the focus alone.
    #[cfg(test)]
    pub fn still(view: View) -> Self {
        Self {
            ahead: view.focus,
            reach: view.reach,
            view,
        }
    }

    /// Returns the distance in metres from the middle of `id` at sea level to the nearest point
    /// of the stretch.
    pub fn near(&self, id: ChunkId) -> f64 {
        let at = id.dir(0.5, 0.5) * self.view.radius;
        let stretch = self.ahead - self.view.focus;
        let length = stretch.length_squared();
        if length <= 0.0 {
            return self.view.distance(id);
        }
        let t = ((at - self.view.focus).dot(stretch) / length).clamp(0.0, 1.0);
        at.distance(self.view.focus + stretch * t)
    }

    /// Returns true when the leaf `id` should split.
    pub fn splits(&self, id: ChunkId) -> bool {
        let narrow = View {
            reach: self.reach,
            ..self.view
        };
        narrow.splits_from(id, self.near(id))
    }

    /// Returns true when the split node `id` should merge.
    pub fn merges(&self, id: ChunkId) -> bool {
        self.view.merges_from(id, self.near(id))
    }

    /// Returns true when `id` lies behind a focus that moves.
    fn behind(&self, id: ChunkId) -> bool {
        let at = id.dir(0.5, 0.5) * self.view.radius;
        (at - self.view.focus).dot(self.ahead - self.view.focus) < 0.0
    }

    /// Returns the build cost of `id`: its distance from the stretch in widths of itself, so
    /// the node that covers the most screen for its distance comes first, and twice that
    /// behind a moving focus.
    pub fn cost(&self, id: ChunkId) -> f64 {
        let widths = self.near(id) / self.view.arc(id.level);
        widths * [1.0, BEHIND][usize::from(self.behind(id))]
    }
}
