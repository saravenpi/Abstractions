//! The air a planet is born with: which gases its heat, its water and its pull leave it.

use super::gas::Gas;
use super::mix::Mix;
use super::tune::{
    ARGON, CARBON_FROST, CARBON_VAPOUR, DRY_NITROGEN, GAS_CONSTANT, GIANT_METHANE, HELIUM,
    HOME_ESCAPE, JEANS, METHANE, STEAM, SULPHUR, SULPHUR_NITROGEN, VOLCANIC, WET_CARBON,
};
use crate::universe::class::MOLTEN;
use crate::universe::hash::{self, Rng};
use crate::universe::record::{PlanetClass, PlanetRecord, Water};

/// Child of the record seed the draw of the mix comes from, so the draws of the record never shift.
const STREAM: u64 = 0x41_49_52;

/// Gases and amounts before the planet loses the ones it cannot hold.
type Recipe = [(Gas, f64); 4];

/// Returns the escape speed in metres per second of a body whose surface gravity and radius
/// are `gravity` and `size` times those of the home planet, which stands for the Earth.
pub fn escape(gravity: f64, size: f64) -> f64 {
    HOME_ESCAPE * (gravity * size).sqrt()
}

/// Returns the air of a planet without life, from its record, its escape speed in metres per
/// second, its inner heat `tide` in units of the starlight the home planet gets, and its seed.
///
/// The record picks the gases, see [`recipe`], and one draw places each share inside its
/// range. The planet then loses every gas whose molecules move too fast for its pull, see
/// [`holds`]; when it holds none, a trace of the heaviest is all its air.
pub fn born(record: &PlanetRecord, escape: f64, tide: f64, seed: u64) -> Mix {
    let roll = Rng::new(hash::child(seed, STREAM)).unit();
    let recipe = recipe(record, tide, roll);
    let held = |gas: Gas| holds(gas, record.temperature, escape);
    let kept = recipe.map(|(gas, amount)| (gas, [0.0, amount][usize::from(held(gas))]));
    if kept.iter().any(|(_, amount)| *amount > 0.0) {
        return Mix::of(kept);
    }
    let heavy = recipe
        .iter()
        .fold(recipe[0].0, |heavy, (gas, _)| heavy.max(*gas));
    let mut alone = [(heavy, 0.0); 4];
    alone[0].1 = 1.0;
    Mix::of(alone)
}

/// Returns true when a body with an escape speed of `escape` metres per second keeps `gas` at
/// `temperature` kelvin: its escape speed is over [`JEANS`] times the thermal speed
/// `sqrt(3 R T / M)` of the gas.
pub fn holds(gas: Gas, temperature: f64, escape: f64) -> bool {
    JEANS * (3.0 * GAS_CONSTANT * temperature / gas.molar()).sqrt() < escape
}

/// Returns the kelvin under which carbon dioxide freezes out of air of `pressure` home
/// atmospheres, by the Clausius-Clapeyron relation: 195 K under 1, 148 K under the 0.006 of Mars.
pub fn frost(pressure: f64) -> f64 {
    1.0 / (1.0 / CARBON_FROST - CARBON_VAPOUR * pressure.ln())
}

/// Returns the gases the record of a planet calls for, `roll` from 0 to 1 placing each share
/// inside its range. The first rule that holds decides:
///
/// - a gas giant is hydrogen and helium with a little methane;
/// - molten ground, or a moon kneaded by tides, breathes sulphur dioxide and carbon dioxide;
/// - where carbon dioxide freezes out, nitrogen and methane are left, as on Titan;
/// - liquid water locks carbon dioxide into rock and leaves nitrogen, as on the early Earth;
/// - boiled water stays in carbon dioxide air as steam;
/// - every other world keeps the carbon dioxide its volcanoes gave it, as Mars and Venus do.
fn recipe(record: &PlanetRecord, tide: f64, roll: f64) -> Recipe {
    let span = |(low, high): (f64, f64)| low + (high - low) * roll;
    let (argon, dry) = ((Gas::Argon, ARGON), (Gas::Nitrogen, span(DRY_NITROGEN)));
    if record.class == PlanetClass::GasGiant {
        let methane = (Gas::Methane, GIANT_METHANE);
        blend(Gas::Hydrogen, (Gas::Helium, span(HELIUM)), methane)
    } else if record.temperature >= MOLTEN || tide >= VOLCANIC {
        let sulphur = (Gas::SulphurDioxide, span(SULPHUR));
        blend(
            Gas::CarbonDioxide,
            sulphur,
            (Gas::Nitrogen, SULPHUR_NITROGEN),
        )
    } else if record.temperature < frost(record.pressure) {
        blend(Gas::Nitrogen, (Gas::Methane, span(METHANE)), argon)
    } else if record.water == Water::Liquid {
        blend(Gas::Nitrogen, (Gas::CarbonDioxide, span(WET_CARBON)), argon)
    } else if record.water == Water::Steam {
        blend(Gas::CarbonDioxide, (Gas::Steam, span(STEAM)), dry)
    } else {
        blend(Gas::CarbonDioxide, dry, argon)
    }
}

/// Returns a recipe of two gases with their shares and `rest`, which fills what they leave.
fn blend(rest: Gas, first: (Gas, f64), second: (Gas, f64)) -> Recipe {
    let left = 1.0 - first.1 - second.1;
    [(rest, left), first, second, (rest, 0.0)]
}
