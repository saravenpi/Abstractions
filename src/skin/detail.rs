//! The level of detail a body is built at, and the two chamfer sizes of the family.

use super::loft::Ring;

/// Chamfer of plates and markings, in meters.
pub const SMALL: f32 = 0.006;

/// Chamfer of blocks: gauntlets, greaves, boots, the visor box and the pack, in meters.
pub const LARGE: f32 = 0.015;

/// How rich a mesh is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Detail {
    /// Every part, chamfered: what is seen up close.
    Near,
    /// The same shapes with half the sides, no chamfer and no small part: what is seen from afar.
    Far,
}

impl Detail {
    /// Returns whether the small parts are built.
    pub fn near(self) -> bool {
        self == Detail::Near
    }

    /// Returns the outline a loft is built with at this level: half the sides and no cut corner from afar.
    pub fn ring(self, ring: Ring) -> Ring {
        match (self, ring) {
            (Detail::Near, ring) => ring,
            (Detail::Far, Ring::Round(sides, power)) => Ring::Round((sides / 2).max(4), power),
            (Detail::Far, Ring::Cut(_)) => Ring::Cut(0.0),
        }
    }

    /// Returns the chamfer a plate is built with at this level: none from afar.
    pub fn cut(self, cut: f32) -> f32 {
        match self {
            Detail::Near => cut,
            Detail::Far => 0.0,
        }
    }
}
