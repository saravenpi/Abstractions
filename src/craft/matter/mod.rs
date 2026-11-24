//! Matter: the engineering numbers of tissues and stones, and the treatments that trade them.
//!
//! Each tissue class is anchored on a measured material (see [`Class::anchor`]) and scaled by how far
//! a species' tissue sits from the home values of its kind in [`crate::life::tissue::HOME`]. Units are
//! the ones engineers use, so damage, wear and swing can follow real formulas.

mod class;
mod mineral;
mod stuff;
mod tissue;
mod treat;
mod tune;
mod wood;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_treat;

pub use class::Class;
pub use mineral::Mineral;
pub use stuff::{Grade, Stuff, UNIT_MASS};
pub use tissue::material;
pub use treat::{Treatment, Treatments};
pub use wood::{ChargeKind, PlaceableKind, WoodKind, bullet_mass};

/// The mechanical numbers of one material.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Material {
    /// Density in kilograms per cubic metre.
    pub density: f32,
    /// Young's modulus E in gigapascals.
    pub stiffness: f32,
    /// Tensile or bending strength in megapascals.
    pub strength: f32,
    /// Fracture toughness K_IC in megapascal root metres.
    pub toughness: f32,
    /// Mohs scratch hardness, 0 to 10.
    pub hardness: f32,
    /// Thermal conductivity in watts per metre kelvin.
    pub conductivity: f32,
}

impl Material {
    /// Fracture toughness under which a material counts as brittle, in MPa·m½: glass sits at 0.7 to 0.8.
    pub const BRITTLE: f32 = 1.0;

    /// Returns the work of fracture G = K_IC² / E in joules per square metre: the energy a crack
    /// needs to open one square metre of new surface (linear elastic fracture mechanics,
    /// https://en.wikipedia.org/wiki/Fracture_toughness).
    pub fn work(&self) -> f32 {
        let k = self.toughness * 1.0e6;
        k * k / (self.stiffness * 1.0e9)
    }

    /// Returns the strain energy one cubic metre stores when it breaks, σ² / 2E, in joules per
    /// cubic metre: what a blunt blow must put into a layer to crack it through.
    pub fn resilience(&self) -> f32 {
        let s = self.strength * 1.0e6;
        s * s / (2.0 * self.stiffness * 1.0e9)
    }

    /// Returns true when the material shatters rather than bends.
    pub fn brittle(&self) -> bool {
        self.toughness < Self::BRITTLE
    }
}
