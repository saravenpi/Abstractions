//! The absorption of sound by a gas, after ISO 9613-1:1993, "Acoustics: attenuation of sound
//! during propagation outdoors, part 1": the classical loss and the relaxation of oxygen and of
//! nitrogen, both moved by the water vapour of the humidity.
//!
//! The standard is written for the air of the Earth. Two terms reach past it for other worlds:
//! the classical loss follows `1 / (rho c^3)`, so the molar mass and the ratio of heats of the
//! gas, and carbon dioxide relaxes at a frequency that follows the pressure, about 240 Hz at the
//! surface of Mars, where the highs die within metres (Maurice et al., "In situ recording of Mars
//! soundscape", Nature 605, 2022, <https://doi.org/10.1038/s41586-022-04679-0>).

use super::tune::{BAND, CORNER_DB, HALVINGS};
use crate::universe::{Gas, Mix, PlanetRecord, Water, humidity};

/// Reference temperature of the standard in kelvin, 20 degrees.
const T0: f64 = 293.15;

/// Triple point of water in kelvin.
const T01: f64 = 273.16;

/// Shares of oxygen and of nitrogen in the air the standard is written for.
const EARTH: (f64, f64) = (0.2095, 0.7808);

/// Relaxation frequency of carbon dioxide at one atmosphere in hertz, and its most absorption
/// per wavelength in nepers.
const CARBON: (f64, f64) = (38_000.0, 0.08);

/// Decibels in a neper.
const DB: f64 = 8.686;

/// Coldest temperature in kelvin the formulas are taken at.
const COLDEST: f64 = 50.0;

/// What a gas is made of and its state, as the absorption needs it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mixture {
    /// Pressure in Earth atmospheres.
    pub pressure: f64,
    /// Temperature in kelvin.
    pub temperature: f64,
    /// Relative humidity of water vapour, from 0 to 1.
    pub humidity: f64,
    /// Shares of oxygen, nitrogen and carbon dioxide among the molecules.
    pub shares: [f64; 3],
    /// Classical loss against the air of home at the same pressure and temperature.
    pub classic: f64,
    /// Speed of sound in metres per second.
    pub sound: f64,
}

impl Mixture {
    /// Returns the gas of the air of `record` at a `pressure` in Earth atmospheres and a
    /// `temperature` in kelvin, where sound travels at `sound` metres per second. A world with no
    /// water has no vapour in its air.
    pub fn of(record: &PlanetRecord, (pressure, temperature): (f64, f64), sound: f32) -> Self {
        let mix = &record.mix;
        let ratio = |mix: &Mix| 1.0 / (1.0 - mix.kappa());
        let heavy = (mix.molar() / Mix::HOME.molar()).sqrt();
        Self {
            pressure,
            temperature,
            humidity: if record.water == Water::None {
                0.0
            } else {
                humidity(record)
            },
            shares: [Gas::Oxygen, Gas::Nitrogen, Gas::CarbonDioxide].map(|gas| mix.share(gas)),
            classic: heavy * (ratio(&Mix::HOME) / ratio(mix)).powf(1.5),
            sound: f64::from(sound),
        }
    }
}

/// The absorption of one gas, worked out once: the classical loss per hertz squared, and the
/// strength and the relaxation frequency of oxygen, nitrogen and carbon dioxide.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loss {
    /// Classical and rotational loss in decibels per metre per hertz squared.
    classic: f64,
    /// Strength and relaxation frequency in hertz of each relaxing gas.
    relax: [(f64, f64); 3],
}

impl Loss {
    /// Works out the absorption of `gas`.
    pub fn of(gas: &Mixture) -> Self {
        let p = gas.pressure.max(1.0e-4);
        let kelvin = gas.temperature.max(COLDEST);
        let t = kelvin / T0;
        let saturated = 10.0_f64.powf(-6.8346 * (T01 / kelvin).powf(1.261) + 4.6151);
        let h = (100.0 * gas.humidity.clamp(0.0, 1.0) * saturated / p).min(100.0);
        let oxygen = p * (24.0 + 4.04e4 * h * (0.02 + h) / (0.391 + h));
        let nitrogen =
            p / t.sqrt() * (9.0 + 280.0 * h * (-4.170 * (t.powf(-1.0 / 3.0) - 1.0)).exp());
        let warm = DB * t.powf(-2.5);
        let [o, n, c] = gas.shares;
        Self {
            classic: DB * 1.84e-11 / p * t.sqrt() * gas.classic,
            relax: [
                (
                    warm * 0.01275 * (-2239.1 / kelvin).exp() * o / EARTH.0,
                    oxygen,
                ),
                (
                    warm * 0.1068 * (-3352.0 / kelvin).exp() * n / EARTH.1,
                    nitrogen,
                ),
                (2.0 * DB * CARBON.1 * c / gas.sound.max(1.0), p * CARBON.0),
            ],
        }
    }

    /// Returns the decibels one metre of the gas takes from a tone of `hz` hertz.
    pub fn at(&self, hz: f64) -> f64 {
        let squared = hz * hz;
        let relax: f64 = self
            .relax
            .iter()
            .map(|(strength, fr)| strength / (fr + squared / fr))
            .sum();
        squared * (self.classic + relax)
    }

    /// Returns the frequency at which `metres` of the gas take [`CORNER_DB`], found by halving
    /// the band on a log scale: the corner of the one-pole low-pass that stands for it. A path
    /// that takes less than that from the top of the band gets the top of the band.
    pub fn corner(&self, metres: f64) -> f64 {
        let (mut low, mut high) = (BAND.0.ln(), BAND.1.ln());
        if self.at(BAND.1) * metres < CORNER_DB {
            return BAND.1;
        }
        for _ in 0..HALVINGS {
            let mid = 0.5 * (low + high);
            if self.at(mid.exp()) * metres < CORNER_DB {
                low = mid;
            } else {
                high = mid;
            }
        }
        (0.5 * (low + high)).exp()
    }
}
