//! The roof each village style puts on a room.

use bevy::prelude::*;

use super::style::{Flavor, Style};
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::eaves::{Cap, Walls};

/// How far the eaves of timber and taiga roofs, and the slab of adobe roofs, reach outside the walls.
const OVER: f32 = 0.5;

/// How far the eaves of stilt house roofs reach outside the walls.
const STILT_OVER: f32 = 1.0;

/// How far the lower tier of a cherry roof reaches outside the walls.
const TIER_OVER: f32 = 1.2;

/// How far inside the walls the eaves of the upper tier of a cherry roof lie.
const TIER_INSET: f32 = 0.5;

impl Build<'_> {
    /// Adds the roof the village style asks for on `walls`, the outer wall faces at the height of the wall tops.
    /// `pick` in `[0, 1)` chooses between a gabled and a hipped roof where the style has both.
    pub fn cap_roof(&mut self, st: &Style, walls: Walls, rise: f32, pick: f32) {
        let cap = Cap {
            roof: st.roof[0],
            wall: st.wall[0],
            hip: pick >= 0.65,
            snow: st.flavor == Flavor::Taiga,
        };
        let wide = Vec3::new(OVER, 0.0, OVER);
        match st.flavor {
            Flavor::Adobe => self.flat_roof(walls.0 - wide, walls.1 + wide, Mat::deco(cap.roof)),
            Flavor::Stilt => {
                self.pitched_over(walls, STILT_OVER, rise * 1.2, &Cap { hip: true, ..cap })
            }
            Flavor::Cherry => self.tiers(walls, rise, &Cap { hip: true, ..cap }),
            Flavor::Taiga | Flavor::Timber => self.pitched_over(walls, OVER, rise, &cap),
        }
    }

    /// Adds a two-tier hipped roof with wide eaves. The eaves of the upper tier lie on the lower tier.
    fn tiers(&mut self, walls: Walls, rise: f32, cap: &Cap) {
        let low = rise * 0.7;
        self.pitched_over(walls, TIER_OVER, low, cap);
        let half = (walls.1.z - walls.0.z) * 0.5 + TIER_OVER;
        let lift = (TIER_OVER + TIER_INSET) * low / half;
        let inset = Vec3::new(TIER_INSET, 0.0, TIER_INSET);
        let (lo, hi) = (walls.0 + inset, walls.1 - inset);
        self.hip(
            lo + Vec3::Y * lift,
            hi + Vec3::Y * lift,
            rise * 0.8,
            Mat::deco(cap.roof),
        );
    }
}
