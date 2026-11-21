//! Climate: the albedo, greenhouse, temperature and water of a planet, settled together.

use super::air;
use super::record::Water;

/// Kelvin under which water freezes.
pub const FREEZING: f64 = 273.0;

/// Kelvin of a black sphere receiving a flux of 1.
///
/// With the 0.3 albedo of the Earth it gives 254.7 K, and the 33 K of its air bring that to 288.
const BLACK_BODY: f64 = 278.5;

/// Share of the starlight bare rock sends back.
const ROCK: f64 = 0.2;

/// Share of the starlight open water sends back.
const SEA: f64 = 0.08;

/// Share of the starlight ice sends back.
const ICE: f64 = 0.55;

/// Share of the starlight clouds send back.
const CLOUD: f64 = 0.6;

/// Share of the starlight the cloud deck of a gas giant sends back: the Bond albedo of Jupiter.
const DECK: f64 = 0.34;

/// Water stock, from 0 to 1, under which a planet has no water at all.
const DRY: f64 = 0.1;

/// Water stock, from 0 to 1, from which the sea drowns nearly all the land.
const DROWNED: f64 = 0.55;

/// Kelvin the air of a cold planet builds up to hold, as far as [`thaw`] lets it.
const THAW: f64 = 280.0;

/// Share of the starlight an Earth sends back: the reference albedo the maximum greenhouse is
/// measured with, see [`thaw`].
const EARTH_ALBEDO: f64 = 0.3;

/// Carbon supply, from 0 to 1, from which a planet has all the carbon its thermostat can use.
const CARBON_FULL: f64 = 0.3;

/// Times the feedback between cover, albedo and temperature is run. Fixed, so the result is too.
const ROUNDS: usize = 4;

/// The climate of a planet: four values that each depend on the three others.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Climate {
    /// Share of the starlight sent back to space, from 0 to 1.
    pub albedo: f64,
    /// Warming by the air in kelvin.
    pub greenhouse: f64,
    /// Mean surface temperature in kelvin.
    pub temperature: f64,
    /// The state of water at the surface.
    pub water: Water,
}

/// Returns the share of the surface under liquid water for a water stock `wet` from 0 to 1.
///
/// From 0.1 to 0.55 the sea covers 40 to 75 percent of the planet, seas and land in balance.
/// Above, it covers 85 percent and more, an ocean world. No stock gives a share in between.
pub fn spread(wet: f64) -> f64 {
    if wet >= DROWNED {
        0.85 + 0.15 * (wet - DROWNED) / (1.0 - DROWNED)
    } else {
        0.4 + 0.35 * (wet.max(DRY) - DRY) / (DROWNED - DRY)
    }
}

/// Returns the state of water at `temperature` kelvin under `pressure`, for a water stock `wet`.
///
/// A stock under 0.1 is no water at all. Water freezes under 273 K. Above, it needs a pressure
/// of 0.006 to stay liquid, without which it is lost to space, and it boils into steam at a
/// temperature that rises with the pressure.
pub fn state(temperature: f64, pressure: f64, wet: f64) -> Water {
    if wet < DRY {
        Water::None
    } else if temperature < FREEZING {
        Water::Ice
    } else if pressure < air::TRIPLE_POINT {
        Water::None
    } else if temperature < air::boiling(pressure) {
        Water::Liquid
    } else {
        Water::Steam
    }
}

/// Returns the most in kelvin that the carbon cycle can add to the greenhouse of a cold wet rock,
/// for a star whose habitable zone ends at the flux `outer`.
///
/// The outer edge is by definition where the strongest greenhouse an air can build only just
/// brings water to freezing. So the cap is what a world of the albedo of the Earth needs there:
/// `273 K - 278.5 K * (outer * (1 - 0.3))^0.25`, 78 K for the Sun, whose edge is at a flux of
/// 0.344. A world farther out gets no more, and freezes. It never exceeds what the thickest air,
/// at the pressure cap, warms by in the grey model: [`air::greenhouse`] at 5 atmospheres, 109 K.
fn thaw(outer: f64) -> f64 {
    let edge = BLACK_BODY * (outer * (1.0 - EARTH_ALBEDO)).powf(0.25);
    (FREEZING - edge).min(air::greenhouse(air::PRESSURE_CAP))
}

/// Returns the albedo of a planet from what covered it one round before.
///
/// The ground is ice at 0.55, bare rock at 0.2, or seas at 0.08 between rock. Clouds at 0.6
/// hide a share of it: 0.1 over a dry world, 0.2 over ice, 0.7 over a boiled sea, and from 0.2
/// to 0.9 over seas as they warm from freezing to boiling, which shades a warm sea. Air thinner
/// than the air of the Earth carries clouds in proportion.
fn albedo(pressure: f64, wet: f64, was: &Climate) -> f64 {
    let (clouds, ground) = match was.water {
        Water::None => (0.1, ROCK),
        Water::Ice => (0.2, ICE),
        Water::Steam => (0.7, ROCK),
        Water::Liquid => {
            let span = air::boiling(pressure) - FREEZING;
            let warmth = ((was.temperature - FREEZING) / span).clamp(0.0, 1.0);
            let sea = spread(wet);
            (0.2 + 0.7 * warmth, sea * SEA + (1.0 - sea) * ROCK)
        }
    };
    let clouds = clouds * pressure.min(1.0);
    clouds * CLOUD + (1.0 - clouds) * ground
}

/// Runs one round of the feedback: cover gives albedo, albedo and air give heat, heat gives water.
///
/// The temperature is `278.5 K * (flux * (1 - albedo))^0.25` plus the greenhouse. The air of a
/// cold planet thickens in heat-trapping gas until the ground reaches 280 K, up to [`thaw`]: this
/// is what holds seas liquid out to the far edge of the habitable zone and not beyond it, where
/// no air warms more than that maximum greenhouse, not even a steady one. That thermostat is
/// the cycle of carbon between air, water and rock, so it needs water and works in proportion to the carbon supply `carbon`, in full from 0.3: a dry or carbon-poor
/// planet keeps its steady greenhouse. `flux` is the starlight and the flux at the outer edge of
/// the zone of the star. A planet whose sea has boiled gets the runaway greenhouse,
/// see [`air::runaway`].
fn step(flux: (f64, f64), pressure: f64, (wet, carbon): (f64, f64), was: &Climate) -> Climate {
    let (flux, outer) = flux;
    let albedo = albedo(pressure, wet, was);
    let bare = BLACK_BODY * (flux * (1.0 - albedo)).powf(0.25);
    let steady = air::greenhouse(pressure);
    let steady = if flux < outer {
        steady.min(thaw(outer))
    } else {
        steady
    };
    let cycle = if wet >= DRY {
        (carbon / CARBON_FULL).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let greenhouse = if was.water == Water::Steam {
        air::runaway(pressure)
    } else {
        let top = steady + (thaw(outer) - steady).max(0.0) * cycle;
        (THAW - bare).clamp(steady, top)
    };
    let temperature = bare + greenhouse;
    Climate {
        albedo,
        greenhouse,
        temperature,
        water: state(temperature, pressure, wet),
    }
}

/// Returns the climate of a planet receiving `flux` under `pressure`, around a star whose zone ends
/// at the flux `outer`, for a water stock `wet`
/// and a carbon supply `carbon`, both from 0 to 1.
///
/// It starts from bare rock and runs the ice and albedo feedback 4 times, a fixed count, so the
/// same planet always settles on the same climate.
pub fn settle(flux: f64, outer: f64, pressure: f64, wet: f64, carbon: f64) -> Climate {
    let bare = Climate {
        albedo: ROCK,
        greenhouse: 0.0,
        temperature: 0.0,
        water: Water::None,
    };
    (0..ROUNDS).fold(bare, |was, _| {
        step((flux, outer), pressure, (wet, carbon), &was)
    })
}

/// Returns the climate at the cloud deck of a gas giant receiving `flux`, for a water stock `wet`.
///
/// The deck sends back 34 percent of the light whatever lies under it, and the air above it is
/// at the pressure cap, so it warms by the steady greenhouse of 109 K with no feedback to run.
/// Water there is in the state [`state`] gives at that heat and pressure.
pub fn deck(flux: f64, wet: f64) -> Climate {
    let greenhouse = air::greenhouse(air::PRESSURE_CAP);
    let temperature = BLACK_BODY * (flux * (1.0 - DECK)).powf(0.25) + greenhouse;
    Climate {
        albedo: DECK,
        greenhouse,
        temperature,
        water: state(temperature, air::PRESSURE_CAP, wet),
    }
}
