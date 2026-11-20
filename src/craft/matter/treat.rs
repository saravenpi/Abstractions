//! Treatments: what a planet lets you do to a tissue, each one a trade-off, never a pure gain.

use serde::{Deserialize, Serialize};

use super::Material;
use super::tune::{FIRE_KELVIN, FREEZE_STIFFNESS, FREEZE_TOUGHNESS, FROST_KELVIN, SUN_FLUX};
use crate::universe::{NO_AIR, PlanetClass, PlanetRecord};

/// One way to change a tissue with what the planet offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Treatment {
    /// Drying in strong sunlight: stiffer and stronger, a little less tough.
    SunDry,
    /// Hardening over heat: a little harder, much weaker and less tough.
    FireHarden,
    /// Freezing through: stiffer, and more brittle.
    FreezeSet,
}

impl Treatment {
    /// Every treatment, in bit order.
    pub const ALL: [Treatment; 3] = [
        Treatment::SunDry,
        Treatment::FireHarden,
        Treatment::FreezeSet,
    ];

    /// Returns true when the planet of `record` offers what the treatment needs: strong starlight and
    /// air to carry the vapour off, ground hot enough to char, or cold enough to freeze through.
    pub fn possible(self, record: &PlanetRecord) -> bool {
        match self {
            Treatment::SunDry => record.flux >= SUN_FLUX && record.pressure >= NO_AIR,
            Treatment::FireHarden => {
                record.class == PlanetClass::Lava || record.temperature >= FIRE_KELVIN
            }
            Treatment::FreezeSet => record.temperature <= FROST_KELVIN,
        }
    }

    /// Returns `m` after the treatment.
    ///
    /// Sun-drying: stiffness ×2.5 and toughness ×0.8 (the plan's slice), strength ×1.7, as antler goes
    /// from 108 to 188 MPa when dry (https://en.wikipedia.org/wiki/Antler). Fire-hardening: slightly
    /// harder, strength −30 %, work of fracture −36 % so K_IC = √(E·G) ×0.8
    /// (https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=EXT_ID:27194289%20AND%20SRC:MED&resultType=core&format=json).
    pub fn apply(self, m: Material) -> Material {
        match self {
            Treatment::SunDry => Material {
                stiffness: m.stiffness * 2.5,
                toughness: m.toughness * 0.8,
                strength: m.strength * 1.7,
                ..m
            },
            Treatment::FireHarden => Material {
                hardness: (m.hardness * 1.1).min(10.0),
                strength: m.strength * 0.7,
                toughness: m.toughness * 0.64f32.sqrt(),
                ..m
            },
            Treatment::FreezeSet => Material {
                stiffness: m.stiffness * FREEZE_STIFFNESS,
                toughness: m.toughness * FREEZE_TOUGHNESS,
                ..m
            },
        }
    }
}

/// The treatments a piece went through, one bit each in [`Treatment::ALL`] order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Treatments(pub u8);

impl Treatments {
    /// Returns these treatments with `treatment` added. A treatment counts once however often it is done.
    pub fn with(self, treatment: Treatment) -> Self {
        Self(self.0 | 1 << treatment as u8)
    }

    /// Returns `m` after every treatment in the set, in bit order.
    pub fn apply(self, m: Material) -> Material {
        Treatment::ALL
            .iter()
            .filter(|&&t| self.0 & 1 << t as u8 != 0)
            .fold(m, |m, t| t.apply(m))
    }
}
