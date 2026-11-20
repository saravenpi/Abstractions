//! Biochemistry: one roll per living planet.

use super::{Chemistry, Energy, Solvent, Structure, World};
use crate::universe::Mix;
use crate::universe::hash::{Rng, child};

mod liquid;

pub use liquid::{solvent, suitability, wet};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_liquid;

/// Child of the planet seed the chemistry draws come from.
const STREAM: u64 = 0x43_48_45_4D;

/// Pigment colour at a star temperature, in kelvin, from the coolest star to the hottest.
const PIGMENT: [(f32, [f32; 3]); 8] = [
    (2000.0, [0.02, 0.01, 0.015]),
    (3000.0, [0.18, 0.03, 0.04]),
    (4000.0, [0.3, 0.18, 0.04]),
    (5000.0, [0.24, 0.4, 0.08]),
    (5800.0, [0.2, 0.5, 0.15]),
    (7000.0, [0.35, 0.62, 0.45]),
    (8000.0, [0.6, 0.75, 0.8]),
    (12_000.0, [0.82, 0.87, 0.95]),
];

/// Returns the linear RGB colour of the light gathering pigment of producers under a star at `kelvin`:
/// green under a yellow star, red to black under a red dwarf, pale under a hot blue star.
///
/// The colour eases between fixed stops, so a small change of temperature gives a small change of colour.
pub fn pigment(kelvin: f32) -> [f32; 3] {
    let last = PIGMENT.len() - 1;
    let above = PIGMENT
        .iter()
        .position(|(k, _)| *k > kelvin)
        .unwrap_or(last + 1);
    let (low, high) = (
        PIGMENT[above.saturating_sub(1).min(last)],
        PIGMENT[above.min(last)],
    );
    let span = (high.0 - low.0).max(1.0);
    let t = ((kelvin - low.0) / span).clamp(0.0, 1.0);
    let t = t * t * (3.0 - 2.0 * t);
    [0, 1, 2].map(|i| low.1[i] + (high.1[i] - low.1[i]) * t)
}

/// Returns the linear RGB glow colour of life in `solvent`.
fn glow(solvent: Solvent) -> [f32; 3] {
    match solvent {
        Solvent::Water => [0.2, 0.9, 0.7],
        Solvent::Ammonia => [0.6, 0.3, 1.0],
        Solvent::Hydrocarbon => [1.0, 0.6, 0.15],
        Solvent::Silicate => [1.0, 0.35, 0.1],
    }
}

/// Returns what hard tissues are made of at `kelvin`, for a seeded metal share `metal` from 0 to 1.
fn structure(kelvin: f32, metal: f64) -> Structure {
    if kelvin > 700.0 || (kelvin < 150.0 && metal < 0.5) {
        Structure::Silica
    } else if metal > 0.85 {
        Structure::Metal
    } else {
        Structure::Carbon
    }
}

/// Returns the biochemistry of a living planet of `world` whose life rolls from `seed`.
///
/// The solvent follows the temperature and pressure, the energy the starlight (vents under a dim
/// sky, heat in molten rock), the structure the temperature and a seeded metal share, and the glow
/// colour the solvent.
pub fn chemistry(world: &World, seed: u64) -> Chemistry {
    let mut rng = Rng::new(child(seed, STREAM));
    let solvent = solvent(world).map_or(Solvent::Water, |(s, _)| s);
    let vents = rng.unit() < 0.1 || world.flux < 0.05;
    let energy = match solvent {
        Solvent::Silicate => Energy::Heat,
        _ if vents => Energy::Chemistry,
        _ => Energy::Light,
    };
    Chemistry {
        solvent,
        energy,
        structure: structure(world.temperature, rng.unit()),
        pigment: pigment(world.star),
        glow: glow(solvent),
    }
}

/// The planet a species lives on, as the species generators read it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Setting {
    /// What the planet imposes on bodies.
    pub world: World,
    /// Its biochemistry.
    pub chemistry: Chemistry,
    /// How far evolution got.
    pub rung: super::Rung,
}

/// Returns the air of a planet whose life has `chemistry`, from the mix `born` it has without life.
///
/// Life that lives in water on starlight splits water and fills the air with oxygen, see
/// [`Mix::breathed`]. Any other life leaves the air as it found it.
pub fn air(chemistry: &Chemistry, born: Mix) -> Mix {
    match (chemistry.solvent, chemistry.energy) {
        (Solvent::Water, Energy::Light) => born.breathed(),
        _ => born,
    }
}
