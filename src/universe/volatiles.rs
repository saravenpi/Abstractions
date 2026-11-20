//! Volatiles: the water and carbon a planet receives, and the cold liquids that pool on it.

/// Flux under which water ice stays solid in the disc a planet forms from: the frost line.
///
/// A black body is at 170 K there. Every habitable zone edge of the star table lies at a flux
/// of 0.21 or more, so the frost line is always beyond it.
pub const FROST: f64 = 0.14;

/// How fast the supply of ices thins inside the frost line, as a power of the flux.
const THINNING: f64 = 0.2;

/// Mass of a star, in Suns, under which its bright youth dries the planets close to it.
///
/// A red dwarf takes up to a billion years to settle and is far brighter until then, so the
/// planets that end up in its habitable zone first spend that time in a runaway greenhouse and
/// lose their water to space: Luger and Barnes 2015, "Extreme Water Loss and Abiotic O2 Buildup
/// on Planets Throughout the Habitable Zones of M Dwarfs", <https://arxiv.org/abs/1411.7412>.
const PARCHING: f64 = 0.35;

/// A liquid that pools on cold worlds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lake {
    /// Kelvin at which it freezes.
    pub melts: f64,
    /// Kelvin at which it boils under one Earth atmosphere.
    pub boils: f64,
    /// Heat of vaporisation over the gas constant, in kelvin.
    pub latent: f64,
    /// Pressure in Earth atmospheres under which it has no liquid state.
    pub least: f64,
    /// Share of the worlds at the right heat and pressure that hold lakes of it.
    pub odds: f64,
}

/// Liquid ammonia: 195 to 240 K under one atmosphere, on one fitting world in four.
pub const AMMONIA: Lake = Lake {
    melts: 195.4,
    boils: 239.8,
    latent: 2808.0,
    least: 0.06,
    odds: 0.25,
};

/// A methane and ethane mix, as on Titan: 90 to 185 K under one atmosphere, on one fitting
/// world in three.
pub const HYDROCARBON: Lake = Lake {
    melts: 90.4,
    boils: 184.6,
    latent: 1767.0,
    least: 0.1,
    odds: 0.35,
};

/// Smallest and largest share of a surface under the lakes of a cold liquid.
pub const LAKES: (f64, f64) = (0.02, 0.3);

/// Returns the share of the ices of the outer disc that reaches a planet receiving `flux`.
///
/// A planet beyond the frost line forms among them and gets them all. Inside it they come by
/// impacts from further out, fewer the deeper in: `(0.14 / flux)^0.2`, two thirds for the Earth.
pub fn reach(flux: f64) -> f64 {
    (FROST / flux).powf(THINNING).min(1.0)
}

/// Returns the water stock of a planet, from 0 to 1, for a draw `roll` from 0 to 1.
///
/// The draw is scaled by the [`reach`] of ices at its `flux`. A star lighter than 0.35 Suns
/// (`suns` is its mass) also dries its close planets in proportion to its mass, in full where
/// the reach is under a half and not at all beyond the frost line.
pub fn water(roll: f64, flux: f64, suns: f64) -> f64 {
    let reach = reach(flux);
    let kept = (suns / PARCHING).min(1.0);
    let far = ((reach - 0.5) / 0.5).clamp(0.0, 1.0);
    roll * reach * (kept + (1.0 - kept) * far)
}

/// Returns the carbon supply of a planet, from 0 to 1, for a draw `roll` from 0 to 1: carbon
/// comes with the ices, so the draw is scaled by the square root of their [`reach`].
pub fn carbon(roll: f64, flux: f64) -> f64 {
    roll * reach(flux).sqrt()
}

/// Returns the share of the surface under lakes of `lake` at `temperature` kelvin under
/// `pressure`, for a draw `roll` from 0 to 1: 0 where it is not liquid and on the worlds that
/// hold none, else 2 to 30 percent.
///
/// The boiling point follows the pressure by the Clausius-Clapeyron relation.
pub fn lakes(lake: &Lake, temperature: f64, pressure: f64, roll: f64) -> f64 {
    if pressure < lake.least || roll >= lake.odds {
        return 0.0;
    }
    let top = 1.0 / (1.0 / lake.boils - pressure.ln() / lake.latent).max(1e-6);
    let liquid = lake.melts <= temperature && temperature < top;
    let rich = (roll / lake.odds).powi(2);
    f64::from(u8::from(liquid)) * (LAKES.0 + (LAKES.1 - LAKES.0) * rich)
}
