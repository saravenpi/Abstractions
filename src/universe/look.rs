//! How each class of planet looks: the colours shared by the ground, the sky and the far view.

use bevy::math::Vec3;

use super::record::PlanetClass;

/// The colours and cloud cover of a class of planet. Colours are linear RGB from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    /// Dominant colour of the dry ground seen from afar.
    pub land: Vec3,
    /// Colour of the liquid that fills the low ground, or of the dry basin where there is none.
    pub sea: Vec3,
    /// Tint of the light the air scatters: white leaves the sky as the star alone makes it.
    pub air: Vec3,
    /// Share of the sky covered by clouds, from 0 to 1.
    pub clouds: f32,
    /// Light the liquid gives off by itself, 0 for water and above 0 for molten rock.
    pub glow: f32,
}

/// Builds one look.
const fn look(land: [f32; 3], sea: [f32; 3], air: [f32; 3], clouds: f32, glow: f32) -> Look {
    Look {
        land: Vec3::new(land[0], land[1], land[2]),
        sea: Vec3::new(sea[0], sea[1], sea[2]),
        air: Vec3::new(air[0], air[1], air[2]),
        clouds,
        glow,
    }
}

/// The look of every class, in the order of [`PlanetClass::ALL`].
const LOOKS: [Look; 7] = [
    look(
        [0.10, 0.30, 0.12],
        [0.03, 0.16, 0.40],
        [1.0, 1.0, 1.0],
        0.7,
        0.0,
    ),
    look(
        [0.13, 0.33, 0.10],
        [0.05, 0.22, 0.42],
        [1.0, 1.0, 1.0],
        0.5,
        0.0,
    ),
    look(
        [0.62, 0.40, 0.17],
        [0.50, 0.33, 0.16],
        [1.0, 0.82, 0.62],
        0.1,
        0.0,
    ),
    look(
        [0.80, 0.86, 0.92],
        [0.55, 0.70, 0.84],
        [0.86, 0.94, 1.0],
        0.35,
        0.0,
    ),
    look(
        [0.07, 0.06, 0.06],
        [1.0, 0.27, 0.03],
        [1.0, 0.55, 0.38],
        0.25,
        4.0,
    ),
    look(
        [0.30, 0.29, 0.28],
        [0.20, 0.19, 0.19],
        [1.0, 1.0, 1.0],
        0.0,
        0.0,
    ),
    look(
        [0.72, 0.60, 0.44],
        [0.55, 0.42, 0.30],
        [1.0, 0.90, 0.74],
        1.0,
        0.0,
    ),
];

impl PlanetClass {
    /// Returns the look of the class.
    pub const fn look(self) -> Look {
        LOOKS[self as usize]
    }

    /// Returns the name of the class as the board computer shows it.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ocean => "ocean",
            Self::Island => "island",
            Self::Desert => "desert",
            Self::Ice => "ice",
            Self::Lava => "lava",
            Self::Barren => "barren",
            Self::GasGiant => "gas giant",
        }
    }
}
