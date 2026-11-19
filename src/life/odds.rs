//! Whether a planet lives, and how far evolution got there.
//!
//! The chance of life is a product of factors, a liquid that fits the temperature, air, water on
//! the ground and usable energy, scaled by loaded dice so that about one system in three holds a
//! living world. The rung then follows the age of the star and how harsh the climate is.

use super::{Rung, Solvent, World, chem};
use crate::universe::hash::{Rng, child};
use crate::universe::{GIANT_RADIUS, NO_AIR};

#[cfg(test)]
pub(super) mod tests;

/// Child of the planet seed the odds draws come from.
const STREAM: u64 = 0x4F_44_44_53;

/// Scale of the chance of life, tuned so that about one system in three holds a living world.
const LOADING: f32 = 0.17;

/// Oldest a star gets, in billions of years.
const OLDEST: f32 = 12.0;

/// Billions of years evolution takes to climb one rung.
const RUNG_GYR: f32 = 2.2;

/// The rungs, from the lowest.
const LADDER: [Rung; 5] = [
    Rung::Mats,
    Rung::Soft,
    Rung::Shelled,
    Rung::Skeletal,
    Rung::Old,
];

/// Returns the share of the chance of life the ground and the light allow on `world`, from 0 to 1.
///
/// Dry ground keeps 40 percent of the chance, and a dim sky 30 percent, since vents still feed life.
fn ground(world: &World) -> f32 {
    let wet = 0.4 + 0.6 * (world.ocean / 0.3).min(1.0);
    let light = 0.3 + 0.7 * (world.flux / 0.1).min(1.0);
    wet * light
}

/// Returns how many rungs a harsh climate takes away: thin air, heat for water life, and the slow
/// chemistry of the cold solvents.
fn harshness(world: &World, solvent: Solvent) -> usize {
    let thin = world.pressure < 0.1;
    let hot = solvent == Solvent::Water && world.temperature > 330.0;
    let slow = matches!(solvent, Solvent::Ammonia | Solvent::Hydrocarbon);
    usize::from(thin) + usize::from(hot) + usize::from(slow)
}

/// Returns the rung evolution reached on `world` with `solvent`, for a draw `age` from 0 to 1 of the star's age.
///
/// A star lives about 10 billion years at the home temperature and shorter the hotter it is, as
/// `(5780 / kelvin)^2.5`, and no star is older than 12. Each 2.2 billion years climb one rung,
/// harshness takes rungs away, and molten silicate life never gets past shells.
fn ladder(world: &World, solvent: Solvent, age: f32) -> Rung {
    let lifetime = (10.0 * (5780.0 / world.star.max(1000.0)).powf(2.5)).min(OLDEST);
    let climbed = (age * lifetime / RUNG_GYR) as usize;
    let rung = LADDER[climbed
        .saturating_sub(harshness(world, solvent))
        .min(LADDER.len() - 1)];
    match solvent {
        Solvent::Silicate => rung.min(Rung::Shelled),
        _ => rung,
    }
}

/// Returns how far life got on a planet of `world` whose life rolls from `seed`, or nothing on a dead world.
///
/// Gas giants and airless worlds never live, nor does a world no liquid fits.
pub fn rung(world: &World, seed: u64) -> Option<Rung> {
    if world.radius >= GIANT_RADIUS.0 as f32 || world.pressure < NO_AIR as f32 {
        return None;
    }
    let (solvent, fit) = chem::solvent(world)?;
    let mut rng = Rng::new(child(seed, STREAM));
    let chance = LOADING * fit.sqrt() * ground(world);
    let lives = (rng.unit() as f32) < chance;
    lives.then(|| ladder(world, solvent, rng.unit() as f32))
}
