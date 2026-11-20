//! Where the lamps stand on the dashboard, and how a reading lights one. A lamp that is out
//! shrinks to nothing.

use bevy::prelude::*;

use super::paint::TEXT;
use super::readings::Lamp;
use super::screens::{BLOCK, on_panel};
use super::tune::TEXEL;

/// Side of a lit lamp in metres, and the pixels of the block between the lamps and from the
/// left edge to the first, and the row they are on.
const LAMP: (f32, f32, f32, f32) = (0.014, 21.0, 8.5, 51.0);

/// Scale of a part that shows nothing: too small to see, and still a valid transform.
const NOTHING: f32 = 1.0e-4;

impl Lamp {
    /// Returns the colour of the lamp as a linear share of white: the lamps of the state of
    /// the ship are white, a warning is amber, a danger is red.
    pub fn tone(self) -> Vec3 {
        match self {
            Lamp::Assist | Lamp::Landed => Vec3::ONE,
            Lamp::Close | Lamp::Hot | Lamp::Frame => Vec3::new(1.0, 0.45, 0.04),
            Lamp::Failing => Vec3::new(1.0, 0.06, 0.04),
        }
    }

    /// Returns where the lamp stands on the raked panel, lit or dark: under its label in the
    /// block of figures on its side.
    pub fn place(self, lit: bool) -> Transform {
        let index = self as usize;
        let (side, slot) = (index / 3, (index % 3) as f32);
        let width = TEXT.0 as f32 * TEXEL;
        let left = if side == 0 { -BLOCK.0 - width } else { BLOCK.0 };
        let across = left + (LAMP.1 * slot + LAMP.2) * TEXEL;
        let size = if lit { LAMP.0 } else { NOTHING };
        let down = BLOCK.1 - LAMP.3 * TEXEL;
        on_panel(Vec2::new(across, down), 0.003).with_scale(Vec3::new(size, size, 1.0))
    }
}
