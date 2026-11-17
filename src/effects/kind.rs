//! What a particle is, how it moves, and which kind the surroundings call for.

use crate::world::{Biome, Surface};

/// Height above sea level in meters, over every summit, above which the air holds no particle
/// and no creature.
pub const CEILING: f32 = 400.0;

/// The sort of thing a particle currently is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Parked and invisible, so the scene stays sparse.
    Hidden,
    /// Falling tree leaf.
    Leaf,
    /// Large tropical leaf.
    BigLeaf,
    /// Cherry petal.
    Petal,
    /// Snowflake.
    Snow,
    /// Wind blown sand and dust.
    Dust,
    /// Sea spray fleck.
    Spray,
    /// Rising bubble.
    Bubble,
    /// Drifting plankton speck.
    Plankton,
    /// Floating pollen speck.
    Pollen,
}

/// How a kind of particle moves and looks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Motion {
    /// Fall speed in meters per second, negative to rise.
    pub fall: f32,
    /// Share of the wind velocity the particle follows.
    pub wind: f32,
    /// Sideways flutter speed in meters per second.
    pub sway: f32,
    /// Flutter rate in radians per second.
    pub rate: f32,
    /// Tumble rate in radians per second, 0 to always face the camera.
    pub spin: f32,
    /// Edge length in meters.
    pub size: f32,
    /// Shortest and longest life in seconds.
    pub life: (f32, f32),
    /// True to spawn close to the ground instead of anywhere in the box.
    pub low: bool,
}

impl Kind {
    /// Returns the motion parameters of the kind.
    pub fn motion(self) -> Motion {
        let m = |fall, wind, sway, rate, spin, size, life| Motion {
            fall,
            wind,
            sway,
            rate,
            spin,
            size,
            life,
            low: false,
        };
        match self {
            Kind::Hidden => m(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, (1.0, 3.0)),
            Kind::Leaf => m(0.9, 0.5, 0.8, 2.2, 2.5, 0.12, (10.0, 16.0)),
            Kind::BigLeaf => m(0.7, 0.4, 0.6, 1.6, 1.5, 0.22, (12.0, 18.0)),
            Kind::Petal => m(0.35, 0.6, 0.5, 1.4, 1.8, 0.07, (14.0, 22.0)),
            Kind::Snow => m(0.9, 0.6, 0.15, 1.0, 0.0, 0.05, (12.0, 18.0)),
            Kind::Dust => Motion {
                low: true,
                ..m(0.05, 1.1, 0.3, 3.0, 0.0, 0.04, (4.0, 8.0))
            },
            Kind::Spray => Motion {
                low: true,
                ..m(0.2, 1.0, 0.2, 3.0, 0.0, 0.05, (3.0, 6.0))
            },
            Kind::Bubble => m(-0.6, 0.0, 0.15, 3.0, 0.0, 0.05, (6.0, 10.0)),
            Kind::Plankton => m(0.03, 0.0, 0.12, 0.8, 0.0, 0.03, (10.0, 16.0)),
            Kind::Pollen => m(0.05, 0.4, 0.3, 1.0, 0.0, 0.04, (8.0, 14.0)),
        }
    }

    /// Returns true for kinds drawn as a leaf shape instead of a dot.
    pub fn is_leafy(self) -> bool {
        matches!(self, Kind::Leaf | Kind::BigLeaf | Kind::Petal)
    }

    /// Returns true for kinds that live under water.
    pub fn is_submerged(self) -> bool {
        matches!(self, Kind::Bubble | Kind::Plankton)
    }
}

/// What surrounds the camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ambient {
    /// Biome under the camera.
    pub biome: Biome,
    /// Surface under the camera.
    pub surface: Surface,
    /// Camera height above sea level in meters, along the local vertical.
    pub height: f32,
    /// Daylight from 0 (night) to 1.
    pub daylight: f32,
    /// Wind strength from 0 to 1.
    pub wind: f32,
}
