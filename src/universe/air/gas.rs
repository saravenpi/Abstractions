//! Gases: the molecules an atmosphere is made of, with the mass and the heat capacity of each.

/// One gas of an atmosphere, from the lightest to the heaviest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Gas {
    /// Molecular hydrogen, the bulk of a gas giant.
    Hydrogen,
    /// Helium.
    Helium,
    /// Methane, kept only by cold worlds.
    Methane,
    /// Water vapour.
    Steam,
    /// Molecular nitrogen.
    Nitrogen,
    /// Molecular oxygen, made by life.
    Oxygen,
    /// Argon.
    Argon,
    /// Carbon dioxide, the air of dead rocky worlds.
    CarbonDioxide,
    /// Sulphur dioxide, breathed out by volcanoes.
    SulphurDioxide,
}

/// Kilograms per mole and joules per mole and kelvin at constant pressure near 300 K of every
/// gas, in the order of [`Gas::ALL`], from the NIST Chemistry WebBook.
const TABLE: [(f64, f64); 9] = [
    (0.002_016, 28.84),
    (0.004_003, 20.79),
    (0.016_04, 35.69),
    (0.018_02, 33.59),
    (0.028_01, 29.12),
    (0.032_00, 29.38),
    (0.039_95, 20.79),
    (0.044_01, 37.13),
    (0.064_07, 39.87),
];

impl Gas {
    /// Every gas, from the lightest to the heaviest.
    pub const ALL: [Gas; 9] = [
        Gas::Hydrogen,
        Gas::Helium,
        Gas::Methane,
        Gas::Steam,
        Gas::Nitrogen,
        Gas::Oxygen,
        Gas::Argon,
        Gas::CarbonDioxide,
        Gas::SulphurDioxide,
    ];

    /// Returns the mass of one mole in kilograms.
    pub fn molar(self) -> f64 {
        TABLE[self as usize].0
    }

    /// Returns the heat one mole takes per kelvin at constant pressure, in joules.
    pub fn heat(self) -> f64 {
        TABLE[self as usize].1
    }
}
