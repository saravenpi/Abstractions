//! Mixes: the gases of one atmosphere with their shares, and what follows from them.

use super::gas::Gas;
use super::tune::{GAS_CONSTANT, OXYGEN};

/// One gas of a mix.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Part {
    /// The gas.
    pub gas: Gas,
    /// Its share of the molecules, from 0 to 1.
    pub share: f64,
}

/// The composition of an atmosphere: up to four gases, the most abundant first.
///
/// The shares add up to 1. A mix of fewer gases ends with parts of share 0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mix {
    /// The gases, by falling share.
    parts: [Part; 4],
}

impl Mix {
    /// The air of the Earth, which the home planet breathes: 28.96 g/mol.
    pub const HOME: Self = Self {
        parts: [
            Part::new(Gas::Nitrogen, 0.7808),
            Part::new(Gas::Oxygen, OXYGEN),
            Part::new(Gas::Argon, 0.0093),
            Part::new(Gas::CarbonDioxide, 0.0004),
        ],
    };

    /// Builds a mix from gases and amounts in any unit, of which one at least is above 0.
    ///
    /// The amounts are scaled to shares that add up to 1 and sorted, the largest first.
    pub fn of(recipe: [(Gas, f64); 4]) -> Self {
        let total: f64 = recipe.iter().map(|(_, amount)| amount.max(0.0)).sum();
        let mut parts = recipe.map(|(gas, amount)| Part::new(gas, amount.max(0.0) / total));
        parts.sort_by(|a, b| b.share.total_cmp(&a.share));
        Self { parts }
    }

    /// Returns the gases that are there, the most abundant first.
    pub fn parts(&self) -> impl Iterator<Item = Part> + '_ {
        self.parts.iter().copied().filter(|part| part.share > 0.0)
    }

    /// Returns the most abundant gas.
    pub fn main_gas(&self) -> Gas {
        self.parts[0].gas
    }

    /// Returns the share of `gas` among the molecules, from 0 to 1.
    pub fn share(&self, gas: Gas) -> f64 {
        let found = self.parts.iter().filter(|part| part.gas == gas);
        found.map(|part| part.share).sum()
    }

    /// Returns the mean mass of one mole of the mix in kilograms.
    pub fn molar(&self) -> f64 {
        self.parts().map(|part| part.share * part.gas.molar()).sum()
    }

    /// Returns the gas constant over the heat capacity of the mix: the exponent that links the
    /// temperature of rising air to its pressure, 0.286 for the air of the Earth.
    pub fn kappa(&self) -> f64 {
        let heat: f64 = self.parts().map(|part| part.share * part.gas.heat()).sum();
        GAS_CONSTANT / heat
    }

    /// Returns the mix after life that splits water has worked on it.
    ///
    /// Oxygen takes the place of the carbon dioxide and holds the share it has on the Earth;
    /// the three most abundant other gases keep their proportions in what is left.
    pub fn breathed(&self) -> Self {
        let spent = |part: &Part| matches!(part.gas, Gas::CarbonDioxide | Gas::Oxygen);
        let inert = self.parts.iter().filter(|part| !spent(part));
        let rest: f64 = inert.clone().map(|part| part.share).sum();
        let mut recipe = [(Gas::Oxygen, 0.0); 4];
        recipe[0].1 = OXYGEN;
        for (slot, part) in recipe[1..].iter_mut().zip(inert) {
            *slot = (
                part.gas,
                part.share * (1.0 - OXYGEN) / rest.max(f64::MIN_POSITIVE),
            );
        }
        Self::of(recipe)
    }
}

impl Part {
    /// Builds a part of a mix.
    const fn new(gas: Gas, share: f64) -> Self {
        Self { gas, share }
    }
}
