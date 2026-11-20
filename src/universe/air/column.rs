//! The column of air over a planet: its scale height, its top and where its clouds sit.

use super::tune::{
    BASE_SHARE, GAS_CONSTANT, HOME_TEMPERATURE, HUMIDITY, LOOSEST, TOP_SCALES, TROPOPAUSE, TROUTON,
};
use super::{Mix, VAPOUR};
use crate::universe::record::{PlanetRecord, Water};
use crate::universe::spec::PlanetSpec;

/// Returns the scale height in metres of the air of a planet: the climb over which it thins
/// by a factor e.
///
/// It is `R T / (M g)`, with the mean surface temperature, the molar mass of the mix and the
/// surface `gravity` in metres per second squared: 8.4 km on the Earth, 11 km on Mars, 16 km
/// on Venus and 21 km on Titan. It never passes the `radius` in metres over [`LOOSEST`], the
/// point where a planet stops holding its air, which only a body far smaller than a real
/// planet reaches.
pub fn scale_height(record: &PlanetRecord, gravity: f64, radius: f64) -> f64 {
    let free = GAS_CONSTANT * record.temperature / (record.mix.molar() * gravity);
    free.min(radius / LOOSEST)
}

/// Returns the atmosphere top, cloud base and cloud top of a planet in metres above sea level,
/// from its record, its surface `gravity` in metres per second squared and its `radius` in metres.
///
/// The top is [`TOP_SCALES`] scale heights up. Clouds start where rising air first condenses,
/// see [`condensation`], at most half way to their top, and end where the weather layer does,
/// see [`weather_top`], at least one scale height up. On the Earth that is 101 km, 1.1 km and
/// 10.9 km.
pub fn heights(record: &PlanetRecord, gravity: f64, radius: f64) -> [f64; 3] {
    let scale = scale_height(record, gravity, radius);
    let top = weather_top(record, scale).max(scale);
    let base = condensation(record, scale).min(BASE_SHARE * top);
    [TOP_SCALES * scale, base, top]
}

/// Returns the density of the air at sea level against the air of the home planet at 288 K.
///
/// It is the ideal gas law: the pressure times the molar mass over the temperature, each
/// against the home value. Multiply by 1.2 kg/m3 for a density.
pub fn dense(record: &PlanetRecord) -> f64 {
    let heavy = record.mix.molar() / Mix::HOME.molar();
    record.pressure * heavy * (HOME_TEMPERATURE / record.temperature)
}

/// Returns the height in metres where the weather layer of a planet ends, 0 when its air is
/// too thin to have one.
///
/// Air rising from the ground cools as `T (p / p0)^kappa`. It reaches the pressure
/// [`TROPOPAUSE`] at `scale / kappa * (1 - (TROPOPAUSE / p0)^kappa)`, with `scale` the scale
/// height in metres.
fn weather_top(record: &PlanetRecord, scale: f64) -> f64 {
    let kappa = record.mix.kappa();
    let left = (TROPOPAUSE / record.pressure).min(1.0);
    scale / kappa * (1.0 - left.powf(kappa))
}

/// Returns the height in metres where air rising from the ground of a planet starts to
/// condense into cloud, infinite when it never does. `scale` is the scale height in metres.
///
/// The air at the ground holds vapour at a relative humidity that grows with the share of the
/// surface under liquid, see [`HUMIDITY`], which sets its dew point by the Clausius-Clapeyron
/// relation. Rising, the air cools by `kappa T / scale` per metre and its dew point falls by
/// `vapour * dew^2 / scale`: the cloud base is where they meet.
fn condensation(record: &PlanetRecord, scale: f64) -> f64 {
    let (heat, vapour) = (record.temperature, vapour(record));
    let humidity = HUMIDITY.0 + (HUMIDITY.1 - HUMIDITY.0) * record.ocean.clamp(0.0, 1.0);
    let dew = 1.0 / (1.0 / heat - vapour * humidity.ln());
    let closing = record.mix.kappa() * heat - vapour * dew * dew;
    match closing > 0.0 {
        true => scale * (heat - dew) / closing,
        false => f64::INFINITY,
    }
}

/// Returns the gas constant over the heat of vaporisation of what makes the clouds of a
/// planet, per kelvin.
///
/// Where water is liquid or steam the clouds are water. Elsewhere they are whatever liquid
/// boils near the temperature of the ground, methane on Titan, which Trouton's rule gives as
/// one over [`TROUTON`] times that temperature.
fn vapour(record: &PlanetRecord) -> f64 {
    match record.water {
        Water::Liquid | Water::Steam => VAPOUR,
        Water::None | Water::Ice => 1.0 / (TROUTON * record.temperature),
    }
}

impl PlanetSpec {
    /// Returns the scale height of the air in metres: the top of the atmosphere over
    /// [`TOP_SCALES`]. The density and the pressure of the air fall by a factor e over it.
    pub fn scale_height(&self) -> f64 {
        self.atmosphere_height / TOP_SCALES
    }
}
