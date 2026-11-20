//! The gas a craft flies through, which colours its glow.

use bevy::prelude::*;

use super::tune::METHANE_BELOW;
use crate::render::{LocalSystem, Vantage};
use crate::universe::{PlanetClass, PlanetRecord, PlanetSpec};

/// The main gas of an atmosphere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Gas {
    /// Nitrogen and oxygen: pink-orange, then white.
    NitrogenOxygen,
    /// Carbon dioxide: blue-white, then violet.
    CarbonDioxide,
    /// Hydrogen and helium: pale pink, then magenta.
    Hydrogen,
    /// Methane: teal, then blue.
    Methane,
    /// Sulphur compounds: yellow.
    Sulphur,
    /// Not known: the colour of a hot black body.
    #[default]
    Unknown,
}

impl Gas {
    /// Returns the main gas of the air of a planet.
    ///
    /// The universe has no composition for a planet, so the class stands for it: a giant is
    /// hydrogen, a world of molten rock has sulphur in its air, a dry world has the carbon
    /// dioxide of its volcanoes, a cold frozen world has methane, and the wet worlds have
    /// nitrogen and oxygen. A planet without air has no gas.
    pub fn of(record: &PlanetRecord) -> Self {
        match record.class {
            PlanetClass::GasGiant => Self::Hydrogen,
            PlanetClass::Lava => Self::Sulphur,
            PlanetClass::Desert => Self::CarbonDioxide,
            PlanetClass::Ice if record.temperature < METHANE_BELOW => Self::Methane,
            PlanetClass::Ice | PlanetClass::Ocean | PlanetClass::Island => Self::NitrogenOxygen,
            PlanetClass::Barren => Self::Unknown,
        }
    }

    /// Returns the linear colour of the glow of a `heat` from 0 to 1: the cool end of the gas
    /// at the start and the hot end at the full glow.
    pub fn color(self, heat: f32) -> Vec3 {
        let heat = heat.clamp(0.0, 1.0);
        let (cool, hot) = match self {
            Self::NitrogenOxygen => (Vec3::new(1.0, 0.3, 0.16), Vec3::new(1.0, 0.85, 0.92)),
            Self::CarbonDioxide => (Vec3::new(0.45, 0.62, 1.0), Vec3::new(0.7, 0.38, 1.0)),
            Self::Hydrogen => (Vec3::new(1.0, 0.58, 0.76), Vec3::new(1.0, 0.28, 0.82)),
            Self::Methane => (Vec3::new(0.18, 0.8, 0.78), Vec3::new(0.3, 0.52, 1.0)),
            Self::Sulphur => (Vec3::new(1.0, 0.78, 0.12), Vec3::new(1.0, 0.96, 0.62)),
            Self::Unknown => return black_body(heat),
        };
        cool.lerp(hot, heat)
    }
}

/// Returns the linear colour of a hot body at a `heat` from 0 to 1: dull orange, then yellow
/// white, then blue white.
fn black_body(heat: f32) -> Vec3 {
    let (red, white, blue) = (
        Vec3::new(1.0, 0.25, 0.05),
        Vec3::new(1.0, 0.7, 0.4),
        Vec3::new(0.85, 0.92, 1.2),
    );
    match heat < 0.5 {
        true => red.lerp(white, heat * 2.0),
        false => white.lerp(blue, heat * 2.0 - 1.0),
    }
}

impl LocalSystem {
    /// Returns the planet or moon under the camera of `vantage`, when a world exists.
    pub fn below(&self, vantage: &Vantage) -> Option<&PlanetSpec> {
        let near = self.0.as_ref()?;
        let index = u8::try_from(vantage.body).ok()?;
        match vantage.moon {
            0 => near.planet(index),
            moon => near.body(index, moon),
        }
    }
}
