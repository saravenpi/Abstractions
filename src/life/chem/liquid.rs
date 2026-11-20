//! Liquids life can use: where each one is liquid, the band of it where life does best, and how
//! much of it a planet holds.
//!
//! Phase data are at one home atmosphere, and boiling points follow the pressure by the
//! Clausius-Clapeyron relation. Hydrocarbons are a methane and ethane mix: it freezes with ethane
//! at 90 K and stays liquid until the ethane boils, at 185 K.

use super::super::{Solvent, World};

/// Share of the best score a solvent gets from damp ground, with no standing liquid.
const DAMP: f32 = 0.2;

/// Share of the surface under liquid from which more standing liquid adds nothing.
const FULL: f32 = 0.3;

/// Share of the odds of life kept by molten silicate, the most speculative chemistry: no known
/// molecule long enough to carry a code survives 1,500 K, so lava life stays a rare film.
const SPECULATIVE: f32 = 0.01;

/// A solvent as life sees it.
struct Liquid {
    /// Which solvent.
    solvent: Solvent,
    /// Share of the odds a chemistry this well known keeps: 1 for the solvents whose chemistry is
    /// worked out in a lab, less for speculative ones.
    known: f32,
    /// Kelvin at which it freezes.
    melts: f32,
    /// Kelvin at which it boils under one home atmosphere.
    boils: f32,
    /// Heat of vaporisation over the gas constant, in kelvin. Zero for molten rock, whose range
    /// does not follow the pressure.
    latent: f32,
    /// Kelvin band where life does best.
    band: (f32, f32),
}

/// Every solvent life can use, with its phase data and its best band.
const LIQUIDS: [Liquid; 4] = [
    Liquid {
        solvent: Solvent::Water,
        known: 1.0,
        melts: 273.15,
        boils: 373.15,
        latent: 4890.0,
        band: (273.15, 313.0),
    },
    Liquid {
        solvent: Solvent::Ammonia,
        known: 1.0,
        melts: 195.4,
        boils: 239.8,
        latent: 2808.0,
        band: (198.0, 225.0),
    },
    Liquid {
        solvent: Solvent::Hydrocarbon,
        known: 1.0,
        melts: 90.4,
        boils: 184.6,
        latent: 1767.0,
        band: (90.4, 120.0),
    },
    Liquid {
        solvent: Solvent::Silicate,
        known: SPECULATIVE,
        melts: 1300.0,
        boils: 3500.0,
        latent: 0.0,
        band: (1500.0, 2200.0),
    },
];

/// Returns the kelvin at which `liquid` boils under `pressure` home atmospheres.
fn boiling(liquid: &Liquid, pressure: f32) -> f32 {
    match liquid.latent > 0.0 {
        true => 1.0 / (1.0 / liquid.boils - pressure.max(1e-3).ln() / liquid.latent).max(1e-6),
        false => liquid.boils,
    }
}

/// Returns how well `kelvin` under `pressure` suits life in `liquid`: 0 where it is not liquid, 1
/// in its best band, easing down to 0 toward the freezing and boiling points.
fn fit(liquid: &Liquid, kelvin: f32, pressure: f32) -> f32 {
    let top = boiling(liquid, pressure);
    let ease = |x: f32| {
        let x = x.clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    };
    let (low, high) = (liquid.band.0, liquid.band.1.min(top));
    if kelvin < liquid.melts || kelvin >= top {
        0.0
    } else if kelvin < low {
        ease((kelvin - liquid.melts) / (low - liquid.melts))
    } else if kelvin > high {
        ease((top - kelvin) / (top - high))
    } else {
        1.0
    }
}

/// Returns how much of `solvent` life finds on `world`: nothing on a planet that holds none of
/// it, a little from ground that is only damp, and more from standing liquid, in full from 30
/// percent of the surface. It reads the share of the surface under each liquid from the planet
/// record.
fn amount(solvent: Solvent, world: &World) -> f32 {
    let share = world.liquids[solvent as usize];
    match share > 0.0 {
        true => DAMP + (1.0 - DAMP) * (share / FULL).min(1.0),
        false => 0.0,
    }
}

/// Returns how well `solvent` on `world` suits life, from 0 to 1: liquid at its temperature and
/// pressure, near the solvent's best band, in a large amount, and of a known chemistry.
pub fn suitability(world: &World, solvent: Solvent) -> f32 {
    LIQUIDS
        .iter()
        .find(|l| l.solvent == solvent)
        .map_or(0.0, |l| {
            l.known * fit(l, world.temperature, world.pressure) * amount(solvent, world)
        })
}

/// Returns the solvent that best suits life on `world`, with its suitability, or nothing when no
/// solvent is liquid there.
pub fn solvent(world: &World) -> Option<(Solvent, f32)> {
    LIQUIDS
        .iter()
        .map(|l| (l.solvent, suitability(world, l.solvent)))
        .filter(|(_, s)| *s > 0.0)
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

/// Returns how well the liquids of `world` together suit life, as independent chances:
/// `1 - product(1 - suitability)`.
pub fn wet(world: &World) -> f32 {
    1.0 - LIQUIDS
        .iter()
        .map(|l| 1.0 - suitability(world, l.solvent))
        .product::<f32>()
}
