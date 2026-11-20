//! Stuff: a compact reference to one carried material, small enough to store in an item.

use serde::{Deserialize, Serialize};

use super::tune::GRADE;
use super::{Class, Material, Mineral, Treatments, material};
use crate::life::Tissue;

/// Writes a 64-bit seed as decimal text and reads it back.
mod seed {
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    /// Writes `seed` as decimal text.
    pub fn serialize<S: Serializer>(seed: &u64, to: S) -> Result<S::Ok, S::Error> {
        to.collect_str(seed)
    }

    /// Reads a seed written by [`serialize`].
    pub fn deserialize<'de, D: Deserializer<'de>>(from: D) -> Result<u64, D::Error> {
        String::deserialize(from)?.parse().map_err(D::Error::custom)
    }
}

/// How cleanly the animal a tissue came from was killed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Grade {
    /// One hit on a weak point.
    Clean,
    /// Two or three hits.
    Good,
    /// More hits, or a crushing blow that bruised the tissue.
    Poor,
}

impl Grade {
    /// Returns the grade of a kill that took `hits` hits, `blunt` when a crushing blow landed.
    pub fn of(hits: u8, blunt: bool) -> Self {
        match (hits, blunt) {
            (_, true) => Grade::Poor,
            (0..=1, false) => Grade::Clean,
            (2..=3, false) => Grade::Good,
            _ => Grade::Poor,
        }
    }

    /// Returns the share of strength and toughness the tissue keeps, 0 to 1.
    pub fn quality(self) -> f32 {
        GRADE[self as usize]
    }
}

/// One material a player carries, by where it came from rather than by its numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stuff {
    /// A tissue harvested from an animal.
    Tissue {
        /// Terrain seed of the planet the species lives on, written as text: a TOML integer stops at
        /// i64::MAX.
        #[serde(with = "seed")]
        planet: u64,
        /// Index of the species in that planet's life.
        species: u16,
        /// Index of the tissue in the species' tissue list.
        tissue: u8,
        /// The class the harvested part gave.
        class: Class,
        /// How cleanly it was killed.
        grade: Grade,
        /// What was done to it since.
        treated: Treatments,
    },
    /// A stone picked up from the ground.
    Mineral(Mineral),
}

impl Stuff {
    /// Returns the numbers of this stuff, reading the species' `tissue` for a harvested tissue.
    /// Returns nothing when a tissue is needed and missing, or does not fit the class.
    pub fn material(&self, tissue: Option<&Tissue>) -> Option<Material> {
        match *self {
            Stuff::Mineral(mineral) => Some(mineral.material()),
            Stuff::Tissue {
                class,
                grade,
                treated,
                ..
            } => material(class, tissue?, grade).map(|m| treated.apply(m)),
        }
    }
}
