//! Whether a planet lives, and how far evolution got there.
//!
//! Each liquid of the planet gives a suitability, combined as independent chances, then scaled by
//! the needs every life shares: energy, air, a calm star and a turning day. Abiogenesis is a
//! Poisson process: life starts after a wait drawn at a rate that grows with the suitability and
//! the speed of the chemistry, and only if that wait is shorter than the age of the star. The rung
//! then follows the time life has had since, at the speed of its chemistry.

mod chance;
mod tune;

pub use chance::chance;

use super::{Rung, Solvent, World, chem};
use crate::universe::hash::{Rng, child};
use chance::{barren, lifetime, rate};
use tune::{BARRIER, RUNG_GYR};

#[cfg(test)]
pub(super) mod tests;
#[cfg(test)]
mod tests_chance;
#[cfg(test)]
mod tests_worlds;

/// Child of the planet seed the odds draws come from.
const STREAM: u64 = 0x4F_44_44_53;

/// Kelvin of the home surface, where the chemistry of life runs at speed 1.
const HOME_KELVIN: f32 = 288.0;

/// The rungs, from the lowest.
const LADDER: [Rung; 5] = [
    Rung::Mats,
    Rung::Soft,
    Rung::Shelled,
    Rung::Skeletal,
    Rung::Old,
];

/// Returns `x` clamped to 0 to 1 and eased at both ends.
fn ease(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Returns the share of the needs every life shares that `world` meets with `solvent`, 0 to 1.
///
/// Energy: starlight, with vents as a floor of 30 percent under a dim sky, and the heat of the
/// melt itself for silicate life. Air: thin air keeps 40 percent. A calm star: red dwarfs under
/// 3,900 K flare, down to 40 percent at 3,000 K and cooler. A turning day: the habitable zone of a
/// cool star lies so close that tides lock the planet's day to its year, one face always lit and
/// the other always dark, which halves the score under 3,200 K and fades out by 4,200 K. Tides
/// grow fast with a smaller star and a closer orbit, so locking is likely for habitable-zone
/// planets of M dwarfs: https://arxiv.org/abs/1708.02981
fn needs(world: &World, solvent: Solvent) -> f32 {
    let energy = match solvent {
        Solvent::Silicate => 1.0,
        _ => 0.3 + 0.7 * (world.flux / 0.1).min(1.0),
    };
    let air = 0.4 + 0.6 * (world.pressure / 0.3).min(1.0);
    let calm = 0.4 + 0.6 * ease((world.star - 3000.0) / 900.0);
    let turning = 1.0 - 0.5 * ease((4200.0 - world.star) / 1000.0);
    energy * air * calm * turning
}

/// Returns the solvent life on `world` would use and how well the planet suits life, 0 to 1, or
/// nothing when no liquid suits it.
fn suitability(world: &World) -> Option<(Solvent, f32)> {
    let (solvent, _) = chem::solvent(world)?;
    Some((solvent, chem::wet(world) * needs(world, solvent)))
}

/// Returns how fast the chemistry of life runs at `kelvin`, relative to water at 288 K, after an
/// Arrhenius law with the softened barrier of [`BARRIER`]. Nothing runs faster than at 288 K: the
/// heat that speeds reactions also breaks long molecules.
fn speed(kelvin: f32) -> f32 {
    (-BARRIER * (1.0 / kelvin.max(1.0) - 1.0 / HOME_KELVIN))
        .exp()
        .min(1.0)
}

/// Returns how many rungs a harsh climate takes away: thin air, and heat for water life.
fn harshness(world: &World, solvent: Solvent) -> usize {
    let thin = world.pressure < 0.1;
    let hot = solvent == Solvent::Water && world.temperature > 330.0;
    usize::from(thin) + usize::from(hot)
}

/// Returns the rung life on `world` with `solvent` reached after `years` billion years.
///
/// Each 1.3 billion years at the chemical speed of the planet climb one rung, harshness takes
/// rungs away, and molten silicate life never gets past mats.
fn ladder(world: &World, solvent: Solvent, years: f32) -> Rung {
    let climbed = (years * speed(world.temperature) / RUNG_GYR) as usize;
    let rung = LADDER[climbed
        .saturating_sub(harshness(world, solvent))
        .min(LADDER.len() - 1)];
    match solvent {
        Solvent::Silicate => Rung::Mats,
        _ => rung,
    }
}

/// Returns how far life got on a planet of `world` whose life rolls from `seed`, or nothing on a dead world.
///
/// The star is a seeded share of its [`lifetime`] old. Gas giants and airless worlds never live,
/// nor does a world no liquid suits.
pub fn rung(world: &World, seed: u64) -> Option<Rung> {
    if barren(world) {
        return None;
    }
    let (solvent, suits) = suitability(world)?;
    let mut rng = Rng::new(child(seed, STREAM));
    let age = rng.unit() as f32 * lifetime(world);
    let wait = -(1.0 - rng.unit() as f32).ln() / rate(world, suits);
    (wait < age).then(|| ladder(world, solvent, age - wait))
}
