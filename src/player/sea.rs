//! The liquid of a planet as a body meets it: one measure for wading, swimming, diving and what is heard.

use bevy::prelude::*;

use crate::space::{Anchor, altitude};
use crate::world::Globe;

/// Returns the height in metres of render position `pos` above the liquid surface of `globe`, negative under it.
///
/// On a dry world it is infinite: no basin, however deep, is under a sea that is not there. Every rule
/// that asks how deep something is in the water reads this, so none of them has to know about dry worlds.
pub fn above_sea(anchor: &Anchor, globe: &Globe, pos: Vec3) -> f32 {
    altitude(anchor, globe, pos) - globe.sea_level()
}
