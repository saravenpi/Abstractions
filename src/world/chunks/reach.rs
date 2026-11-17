//! The render distance: everything streaming scales from the one setting.

use bevy::prelude::*;

use super::FRAME_BUDGET;
use super::near::leaf_arc;
use crate::world::Globe;
use crate::world::mesh::Rings;

/// Detail radius in chunks used when the settings are missing: the Medium step.
pub const DEFAULT_RINGS: u32 = 4;

/// Shortest and longest detail radius in chunks the setting may ask for.
pub const RING_LIMITS: (u32, u32) = (2, 24);

/// The named steps of the render distance and their detail radius in chunks.
pub const STEPS: [(&str, u32); 5] = [
    ("Low", 3),
    ("Medium", 4),
    ("High", 8),
    ("Ultra", 16),
    ("Extreme", 24),
];

/// Distance in metres within which the near sea is opaque, so a chunk that draws the bare sea bed
/// must end closer than this.
const SEA_COVER: f64 = 1270.0;

/// How many chunk widths the far corner of a chunk may lie from the focus per ring of distance,
/// where three cube faces meet and the chart is skewed the most.
const SKEW: f64 = 1.65;

/// Share of the detail radius the fog lets the camera see, so that chunks appear inside the fog
/// even where the chart squeezes them the most.
const FOG_SHARE: f64 = 0.78;

/// What the render distance setting asks of the streamer. The streamer reads this resource, which
/// follows the setting; a run without settings uses [`DEFAULT_RINGS`].
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reach {
    /// Detail radius in chunks around the focus.
    pub rings: i32,
}

impl Reach {
    /// Returns the reach for the render distance `setting`, a detail radius in chunks, kept within
    /// [`RING_LIMITS`]; without a setting it is [`DEFAULT_RINGS`].
    pub fn of(setting: Option<u32>) -> Self {
        let rings = setting.unwrap_or(DEFAULT_RINGS);
        Self {
            rings: rings.clamp(RING_LIMITS.0, RING_LIMITS.1) as i32,
        }
    }

    /// Returns how many times its width away a coarse node splits: farther at long settings, so
    /// distant terrain is sharper too.
    pub fn split(&self) -> f64 {
        2.0 + f64::from((self.rings - 4).max(0)) * 0.04
    }

    /// Returns the most quadtree nodes kept alive, hidden parents included: room for the detail
    /// disc down to the deepest level and for the rings of every coarser level.
    pub fn node_cap(&self) -> usize {
        let disc = f64::from(self.rings + 3).powi(2) * 6.4;
        let levels = 20.0 * self.split().powi(2) * 11.0;
        (disc + levels) as usize
    }

    /// Returns the most detail chunk tasks running or waiting at the same time.
    pub fn tasks(&self) -> usize {
        8 + self.rings as usize
    }

    /// Returns the most finished detail chunks turned into entities in one frame of play, the
    /// time budget allowing.
    pub fn per_frame(&self) -> usize {
        FRAME_BUDGET + self.rings as usize / 8
    }

    /// Returns the last ring of chunks that shows its grass and flowers.
    pub fn cover(&self) -> i32 {
        2 + self.rings / 4
    }

    /// Returns the terrain levels by ring on `globe`: finer rings at long settings, and the flat
    /// sea drawn into every chunk the near sea may not cover.
    pub fn levels(&self, globe: &Globe) -> Rings {
        let bare = (SEA_COVER / (SKEW * leaf_arc(globe))) as i32 - 2;
        let mid = (2 + self.rings / 4).min(bare);
        Rings {
            near: (1 + self.rings / 12).min(mid),
            mid,
            bare,
        }
    }

    /// Returns the distance in metres the fog lets the camera see on the ground of `globe`.
    pub fn sight(&self, globe: &Globe) -> f32 {
        (f64::from(self.rings) * leaf_arc(globe) * FOG_SHARE) as f32
    }
}
