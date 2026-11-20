//! The groups of rows of the card of a planet: size and gravity, air and heat, day and year, water
//! and life, and the rarer facts behind the More row.

use super::detail::Row;
use super::feel::{day_feel, gravity_feel, heat, versus};
use super::giant::{company, flying, ground, surface, wet};
use super::words::{flux, temperature, tilt, year, zone};
use crate::saves::play_time;
use crate::ui::Icon;
use crate::ui::units::{distance, gravity_ratio, gravity_si};
use crate::universe::{PlanetSpec, STANDARD_GRAVITY};

/// Returns one measure of a planet.
pub fn stat(icon: Icon, label: &'static str, value: String, note: String) -> Row {
    Row::Stat {
        icon,
        label,
        value,
        note,
    }
}

/// Returns the group of the radius and the pull of `planet`, against `home`. `known` is true
/// when the record of the planet is: only then does the group tell a gas giant from a planet
/// with ground.
pub fn size(planet: &PlanetSpec, home: &PlanetSpec, known: bool) -> Vec<Row> {
    let pull = planet.gravity / STANDARD_GRAVITY;
    let meaning = match known {
        true => ground(planet),
        false => Row::Meaning(gravity_feel(pull).to_string()),
    };
    vec![
        Row::Heading("Size and gravity"),
        stat(
            Icon::Radius,
            "Radius",
            distance(planet.radius),
            versus(planet.radius, home.radius),
        ),
        stat(
            Icon::Gravity,
            "Gravity",
            gravity_ratio(planet.gravity),
            gravity_si(planet.gravity),
        ),
        meaning,
    ]
}

/// Returns the group of the air and the temperature of `planet`, and what its air does for the
/// wings of the player.
pub fn climate(planet: &PlanetSpec) -> Vec<Row> {
    let kelvin = planet.record.temperature;
    let heat = stat(
        Icon::Temperature,
        "Temperature",
        temperature(kelvin),
        heat(kelvin).to_string(),
    );
    let rows = [Row::Heading("Air and heat"), surface(planet), heat];
    rows.into_iter().chain(flying(planet)).collect()
}

/// Returns the group of the day and the year of `planet`, against `home`. The year of a moon is
/// its turn around its planet, which has no comparison with home.
pub fn time(planet: &PlanetSpec, home: &PlanetSpec) -> Vec<Row> {
    let (turn, orbit) = match planet.id.is_moon() {
        true => ("Orbit", String::new()),
        false => ("Year", versus(planet.orbit_seconds, home.orbit_seconds)),
    };
    let day = planet.day_seconds.abs();
    let ratio = day / home.day_seconds.abs();
    let mean = ratio
        .is_finite()
        .then(|| Row::Meaning(day_feel(ratio).to_string()));
    let rows = [
        Row::Heading("Day and year"),
        stat(
            Icon::Day,
            "Day",
            play_time(day),
            versus(day, home.day_seconds.abs()),
        ),
        stat(Icon::Year, turn, year(planet.orbit_seconds), orbit),
    ];
    rows.into_iter().chain(mean).collect()
}

/// Returns the heading of the group of the water and the life of `planet`, then its water, for
/// a planet with ground. The line on life goes after it.
pub fn wet_group(planet: &PlanetSpec) -> Vec<Row> {
    [Row::Heading("Water and life")]
        .into_iter()
        .chain(wet(planet))
        .collect()
}

/// Returns the More row and the rarer facts behind it: the light, the zone, the tilt, the moons
/// and the rings.
pub fn rarer(planet: &PlanetSpec) -> Vec<Row> {
    let record = &planet.record;
    let facts = [
        Row::More,
        Row::Fact("Light", flux(record.flux)),
        Row::Fact("Zone", zone(record.zone).to_string()),
        Row::Fact("Tilt", tilt(record.tilt)),
    ];
    facts.into_iter().chain(company(planet)).collect()
}
