//! The rows of a card that change with the kind of world: the air and the water of a solid
//! planet, or the cloud deck of a gas giant with no ground, and the moons and the rings.

use super::detail::Row;
use super::feel::lift;
use super::groups::{fact, stat};
use super::words::water;
use crate::ui::Icon;
use crate::ui::units::{distance, pressure_bar, pressure_ratio};
use crate::universe::{PlanetClass, PlanetSpec, Rings, STANDARD_GRAVITY};

/// What the card says of the ground of a gas giant.
pub const NO_GROUND: &str = "No ground: gas giant";

/// Formats the rings of a planet by their inner and outer radius.
pub fn rings(rings: Rings) -> String {
    format!("{} to {}", distance(rings.inner), distance(rings.outer))
}

/// Returns true for a gas giant, which has a cloud deck and no ground.
pub fn is_giant(planet: &PlanetSpec) -> bool {
    planet.record.class == PlanetClass::GasGiant
}

/// Returns the line that says what the pull means for a player: that a gas giant has no ground
/// to land on, and for a solid planet how the jumps feel.
pub fn ground(planet: &PlanetSpec) -> Row {
    match is_giant(planet) {
        true => Row::Meaning(NO_GROUND.to_string()),
        false => {
            let ratio = planet.gravity / STANDARD_GRAVITY;
            Row::Meaning(super::feel::gravity_feel(ratio).to_string())
        }
    }
}

/// Returns the row of the air of `planet`: its pressure and a word for it. A gas giant has no
/// ground: its air is measured at the cloud deck.
pub fn surface(planet: &PlanetSpec) -> Row {
    let pressure = planet.record.pressure;
    match is_giant(planet) {
        true => stat(
            Icon::Pressure,
            "Deck air",
            pressure_ratio(pressure),
            pressure_bar(pressure),
        ),
        false => stat(
            Icon::Air,
            "Air",
            pressure_ratio(pressure),
            pressure_bar(pressure),
        ),
    }
}

/// Returns the line that says what the air does for the wings of the player, for a planet with
/// ground.
pub fn flying(planet: &PlanetSpec) -> Option<Row> {
    let words = lift(planet.record.pressure);
    (!is_giant(planet)).then(|| Row::Meaning(words.to_string()))
}

/// Returns the row of the water of `planet`, for a planet with ground.
pub fn wet(planet: &PlanetSpec) -> Option<Row> {
    let value = water(&planet.record);
    (!is_giant(planet)).then(|| fact(Icon::Water, "Water", value))
}

/// Returns the rows that count the moons and give the rings of `planet`, for those it has. A
/// gas giant always says how many moons it has.
pub fn company(planet: &PlanetSpec) -> Vec<Row> {
    let count = (is_giant(planet) || planet.moons > 0)
        .then(|| fact(Icon::Moon, "Moons", planet.moons.to_string()));
    let ring = planet
        .rings
        .map(|ring| fact(Icon::Rings, "Rings", rings(ring)));
    count.into_iter().chain(ring).collect()
}
