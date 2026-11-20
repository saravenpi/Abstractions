//! Carriers: the metal protein that carries oxygen decides the colour of the blood.
//!
//! On Earth iron in haem is red, copper in haemocyanin blue when it carries oxygen and clear when it
//! does not, iron in chlorocruorin green, iron in haemerythrin violet and vanadium in the blood of
//! sea squirts yellow-green. Off Earth only the iron and copper cases are known; the other rules
//! below are our guess, picked so that the carrier follows the chemistry of the planet.

use super::super::{Chemistry, Energy, Life, Plan, Rung, Solvent, Species, Structure};
use super::tune::DRIED;

/// What carries the oxygen in a body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Carrier {
    /// Iron in haem: red.
    Haem,
    /// Copper in haemocyanin: blue with oxygen, pale without.
    Haemocyanin,
    /// Iron in chlorocruorin: green.
    Chlorocruorin,
    /// Iron in haemerythrin: violet with oxygen, pale without.
    Haemerythrin,
    /// Vanadium in vanabins: yellow-green.
    Vanadium,
    /// A molten silicate with no carrier: it glows as the life of the planet does.
    Melt,
}

/// The blood of one species: its carrier, and the colour of the fluid in linear RGB.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blood {
    /// What carries the oxygen.
    pub carrier: Carrier,
    /// Colour with oxygen: from an artery, and of a spill in the air.
    pub oxygenated: [f32; 3],
    /// Colour without oxygen: from a vein.
    pub spent: [f32; 3],
    /// True when the heart drives it through closed vessels, false when it bathes the organs.
    pub closed: bool,
}

impl Carrier {
    /// Returns the colour with oxygen and without, in linear RGB, with `glow` the colour of the
    /// bioluminescence of the planet.
    fn colours(self, glow: [f32; 3]) -> ([f32; 3], [f32; 3]) {
        match self {
            Carrier::Haem => ([0.46, 0.004, 0.004], [0.13, 0.004, 0.008]),
            Carrier::Haemocyanin => ([0.06, 0.16, 0.58], [0.51, 0.58, 0.61]),
            Carrier::Chlorocruorin => ([0.19, 0.46, 0.045], [0.4, 0.58, 0.19]),
            Carrier::Haemerythrin => ([0.31, 0.045, 0.31], [0.64, 0.58, 0.64]),
            Carrier::Vanadium => ([0.4, 0.51, 0.045], [0.1, 0.3, 0.1]),
            Carrier::Melt => (glow, glow.map(|c| c * 0.5)),
        }
    }

    /// Returns the carrier of the second kind of body on a planet whose first kind carries `self`:
    /// the one an open circulation uses.
    fn other(self) -> Carrier {
        match self {
            Carrier::Haem | Carrier::Haemerythrin | Carrier::Chlorocruorin => Carrier::Haemocyanin,
            Carrier::Haemocyanin => Carrier::Haem,
            Carrier::Vanadium => Carrier::Haemocyanin,
            Carrier::Melt => Carrier::Melt,
        }
    }
}

/// Returns the carrier of the animals with closed vessels on a planet of `chemistry`: molten silicate
/// carries none, a metal rich crust makes copper the easy metal, and water gives iron, in the
/// haemerythrin form where the energy is chemical and the air holds little oxygen. The cold
/// solvents give vanadium (ammonia) and chlorocruorin (hydrocarbon).
fn primary(chemistry: &Chemistry) -> Carrier {
    match (chemistry.solvent, chemistry.structure, chemistry.energy) {
        (Solvent::Silicate, _, _) => Carrier::Melt,
        (_, Structure::Metal, _) => Carrier::Haemocyanin,
        (Solvent::Ammonia, _, _) => Carrier::Vanadium,
        (Solvent::Hydrocarbon, _, _) => Carrier::Chlorocruorin,
        (Solvent::Water, _, Energy::Chemistry) => Carrier::Haemerythrin,
        (Solvent::Water, _, _) => Carrier::Haem,
    }
}

/// Returns the blood of a body of `plan` at `rung` on a planet of `chemistry`, or nothing when it has
/// no circulating fluid: the soft rungs, slimes, jellies, swarms and crystals. A skeleton has closed
/// vessels with the carrier of the planet, a segmented body an open circulation with the other one.
pub fn of(chemistry: &Chemistry, plan: Plan, rung: Rung) -> Option<Blood> {
    if matches!(rung, Rung::Mats | Rung::Soft) {
        return None;
    }
    let (carrier, closed) = match plan {
        Plan::Skeletal => (primary(chemistry), true),
        Plan::Segmented => (primary(chemistry).other(), false),
        _ => return None,
    };
    let (oxygenated, spent) = carrier.colours(chemistry.glow);
    Some(Blood {
        carrier,
        oxygenated,
        spent,
        closed,
    })
}

/// Returns the blood of the species `species` of `life`.
pub fn species(life: &Life, species: &Species) -> Option<Blood> {
    of(&life.chemistry, species.plan, life.rung)
}

impl Blood {
    /// Returns the colour of a drop that lies on the ground `fresh` from 1 (just fell) to 0
    /// (dried): the oxygenated colour darkening to [`DRIED`] of itself.
    pub fn stain(&self, fresh: f32) -> [f32; 3] {
        let f = fresh.clamp(0.0, 1.0);
        self.oxygenated.map(|c| c * (DRIED + (1.0 - DRIED) * f))
    }

    /// Returns the colour of a drop in the air: oxygenated from an artery, spent from a vein.
    pub fn drop(&self, artery: bool) -> [f32; 3] {
        if artery { self.oxygenated } else { self.spent }
    }
}
