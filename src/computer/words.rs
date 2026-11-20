//! The planet record in plain words: temperature, air, water, light, tilt and year.

use crate::saves::play_time;
use crate::universe::{NO_AIR, PlanetClass, PlanetRecord, Water, Zone};

use super::format::short;

/// Kelvin at zero degrees Celsius.
pub(super) const FREEZING: f64 = 273.15;

/// Pressures, the home planet having 1, under which the air is none, thin and like home.
/// From the last one up the air is thick.
pub const AIR: [(f64, &str); 3] = [(NO_AIR, "none"), (0.5, "thin"), (1.5, "like home")];

/// Seconds from which a year is counted in days of play instead of hours and minutes.
pub const LONG_YEAR: f64 = 172_800.0;

/// Seconds in one day of play.
const DAY: f64 = 86_400.0;

/// Formats a temperature in kelvin as whole degrees Celsius, such as `15 C` or `-120 C`.
pub fn temperature(kelvin: f64) -> String {
    format!("{} C", (kelvin - FREEZING).round() as i64)
}

/// Returns the word for the state water is in.
pub const fn state(water: Water) -> &'static str {
    match water {
        Water::None => "none",
        Water::Ice => "ice",
        Water::Liquid => "liquid",
        Water::Steam => "steam",
    }
}

/// Formats the water of a planet: its state, with the share of the surface under the oceans
/// when it is liquid. On a lava world the share is the molten rock, whatever the water does.
pub fn water(record: &PlanetRecord) -> String {
    let share = (record.ocean.clamp(0.0, 1.0) * 100.0).round() as i64;
    let word = state(record.water);
    match (record.class, record.water) {
        (PlanetClass::Lava, _) => format!("{word}, molten rock {share} %"),
        (_, Water::Liquid) => format!("{word}, oceans {share} %"),
        _ => word.to_string(),
    }
}

/// Formats the starlight a planet receives against the home planet, such as `1.00 x home`.
pub fn flux(flux: f64) -> String {
    format!("{} x home", short(flux.max(0.0)))
}

/// Returns where an orbit lies against the habitable zone, in plain words.
pub const fn zone(zone: Zone) -> &'static str {
    match zone {
        Zone::Hot => "too hot for water",
        Zone::Habitable => "in the habitable zone",
        Zone::Cold => "too cold for water",
    }
}

/// Formats the tilt of a spin axis, given in radians, as whole degrees.
pub fn tilt(radians: f64) -> String {
    format!("{} deg", radians.to_degrees().round() as i64)
}

/// Formats the length of a year: hours and minutes of play, or whole days from two days up.
pub fn year(seconds: f64) -> String {
    if seconds < LONG_YEAR {
        play_time(seconds)
    } else {
        format!("{} days", (seconds / DAY).round() as i64)
    }
}
