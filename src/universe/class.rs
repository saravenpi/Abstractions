//! Classes: the kind of world a record describes, and the share of it under liquid.

use super::air::NO_AIR;
use super::climate::{self, Climate};
use super::record::{PlanetClass, PlanetRecord, Water};
use super::units::STARLIGHT;

/// Kelvin from which rock melts in the low ground, a round figure for the first minerals to go.
pub const MOLTEN: f64 = 1000.0;

/// Kelvin above [`MOLTEN`] at which molten rock covers the most ground it ever does.
const MOLTEN_SPAN: f64 = 1000.0;

/// Tidal heat flow, in units of flux, from which a body is volcanic all over: 1 watt per square
/// metre. Io, the most volcanic body known, gives off about 2.5.
pub const VOLCANIC: f64 = 1.0 / STARLIGHT;

/// Tidal heat flows, in [`VOLCANIC`], over which lava covers the most ground it ever does.
const ERUPTING: f64 = 4.0;

/// Smallest and largest share of a lava world under molten rock.
const POOLS: (f64, f64) = (0.1, 0.4);

/// Share of the surface under water from which a world of seas is an ocean world.
const OCEAN_SHARE: f64 = 0.85;

/// Returns the share of the surface under liquid, water or molten rock, from 0 to 1.
///
/// A body whose tides heat it past [`VOLCANIC`] has lava over 10 percent of its ground, growing
/// to 40 percent at 4 times that. Else a world without air has none. A world hot enough to melt
/// rock has pools over 10 percent of its ground, growing to 40 percent 1,000 K further up.
/// Liquid water spreads as its stock `wet` says, see [`climate::spread`]. Every other world is
/// dry.
pub fn liquid(pressure: f64, climate: &Climate, wet: f64, tide: f64) -> f64 {
    let pools = |heat: f64| POOLS.0 + (POOLS.1 - POOLS.0) * heat.clamp(0.0, 1.0);
    if tide >= VOLCANIC {
        pools((tide / VOLCANIC - 1.0) / (ERUPTING - 1.0))
    } else if pressure < NO_AIR {
        0.0
    } else if climate.temperature >= MOLTEN {
        pools((climate.temperature - MOLTEN) / MOLTEN_SPAN)
    } else if climate.water == Water::Liquid {
        climate::spread(wet)
    } else {
        0.0
    }
}

/// Returns the class of a planet from its record alone.
///
/// The first rule that holds decides: tides strong enough to keep volcanoes erupting all over
/// make a lava world, air or not, no air to speak of is barren, rock hot enough to melt is
/// lava, liquid water is an ocean world from 85 percent of the surface and islands under that,
/// frozen water is ice, and what is left, air without open water, is desert.
pub fn classify(record: &PlanetRecord) -> PlanetClass {
    if record.tide >= VOLCANIC {
        PlanetClass::Lava
    } else if record.pressure < NO_AIR {
        PlanetClass::Barren
    } else if record.temperature >= MOLTEN {
        PlanetClass::Lava
    } else if record.water == Water::Liquid && record.ocean >= OCEAN_SHARE {
        PlanetClass::Ocean
    } else if record.water == Water::Liquid {
        PlanetClass::Island
    } else if record.water == Water::Ice {
        PlanetClass::Ice
    } else {
        PlanetClass::Desert
    }
}
