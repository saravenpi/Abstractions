//! Biochemistry: one roll per living planet.

use super::{Chemistry, Energy, Solvent, Structure, World};
use crate::universe::hash::{Rng, child};

#[cfg(test)]
mod tests;

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

/// Kelvin between which each solvent is liquid, water aside, whose top follows the pressure.
const AMMONIA: (f32, f32) = (195.0, 240.0);

/// Kelvin between which hydrocarbons are liquid.
const HYDROCARBON: (f32, f32) = (90.0, 190.0);

/// Kelvin between which molten silicate carries life.
const SILICATE: (f32, f32) = (1300.0, 3500.0);

/// Kelvin at which water freezes.
const FREEZING: f32 = 273.0;

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

/// Returns how well a temperature `t` fits a liquid range from `low` to `high`: 1 in the middle, 0 at
/// and past the edges.
pub fn fit(t: f32, low: f32, high: f32) -> f32 {
    let x = (2.0 * t - low - high) / (high - low);
    (1.0 - x.powi(4)).max(0.0)
}

/// Returns the kelvin at which water boils under `pressure` home atmospheres, by Clausius-Clapeyron from 373 K.
fn boiling(pressure: f32) -> f32 {
    1.0 / (1.0 / 373.15 - 2.045e-4 * pressure.max(1e-3).ln())
}

/// Returns the solvent that best fits the temperature and pressure of `world`, with its fit, or nothing when none does.
pub fn solvent(world: &World) -> Option<(Solvent, f32)> {
    let t = world.temperature;
    let fits = [
        (Solvent::Water, fit(t, FREEZING, boiling(world.pressure))),
        (Solvent::Ammonia, fit(t, AMMONIA.0, AMMONIA.1)),
        (Solvent::Hydrocarbon, fit(t, HYDROCARBON.0, HYDROCARBON.1)),
        (Solvent::Silicate, fit(t, SILICATE.0, SILICATE.1)),
    ];
    fits.into_iter()
        .filter(|(_, f)| *f > 0.0)
        .max_by(|a, b| a.1.total_cmp(&b.1))
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
