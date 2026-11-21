//! The gas a craft flies through, which colours its glow.

use bevy::prelude::*;

use crate::life::{World, chem, odds};
use crate::render::{LocalSystem, Vantage};
use crate::universe::{self as air, Mix, PlanetSpec};

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
    /// Returns the gas that colours the glow in `mix`, from its most abundant gas: nitrogen,
    /// oxygen and the air of a living world glow like the Earth's, hydrogen and helium like a
    /// giant's, and water vapour or argon alone leave the glow a hot black body.
    pub fn of(mix: &Mix) -> Self {
        match mix.main_gas() {
            air::Gas::Nitrogen | air::Gas::Oxygen => Self::NitrogenOxygen,
            air::Gas::CarbonDioxide => Self::CarbonDioxide,
            air::Gas::Hydrogen | air::Gas::Helium => Self::Hydrogen,
            air::Gas::Methane => Self::Methane,
            air::Gas::SulphurDioxide => Self::Sulphur,
            air::Gas::Steam | air::Gas::Argon => Self::Unknown,
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

/// Second radiation constant of Planck's law, `h c / k`, in metre kelvin.
const PLANCK_C2: f64 = 1.438_777e-2;

/// Wavelengths in metres the red, green and blue of the screen stand for.
const PRIMARIES: [f64; 3] = [610.0e-9, 550.0e-9, 465.0e-9];

/// Returns the linear colour of a hot surface at `kelvin`, brightest channel 1: the radiance of
/// Planck's law at the wavelengths of [`PRIMARIES`], dull red at 1,200 K, orange at 2,500 K and
/// near white past 6,000 K. Nothing below 300 K.
pub fn incandescence(kelvin: f32) -> Vec3 {
    if kelvin < 300.0 {
        return Vec3::ZERO;
    }
    let radiance =
        |wave: f64| 1.0 / (wave.powi(5) * ((PLANCK_C2 / (wave * f64::from(kelvin))).exp() - 1.0));
    let [red, green, blue] = PRIMARIES.map(radiance);
    let top = red.max(green).max(blue);
    Vec3::new(
        (red / top) as f32,
        (green / top) as f32,
        (blue / top) as f32,
    )
}

impl LocalSystem {
    /// Returns the gas of the air under the camera of `vantage`, [`Gas::Unknown`] outside a world.
    ///
    /// Life that splits water fills the air with oxygen, see [`chem::air`], so a living world
    /// glows with the mix it breathes and a dead one with the mix it was born with.
    pub fn gas(&self, vantage: &Vantage) -> Gas {
        let (Some(planet), Some(near)) = (self.below(vantage), self.0.as_ref()) else {
            return Gas::Unknown;
        };
        let world = World::of(planet, &near.system.star);
        let mix = match odds::rung(&world, planet.seed) {
            Some(_) => chem::air(&chem::chemistry(&world, planet.seed), planet.record.mix),
            None => planet.record.mix,
        };
        Gas::of(&mix)
    }

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
