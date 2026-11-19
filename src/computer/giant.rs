//! The rows of a card that change with the kind of world: the air and the water of a solid
//! planet, or the cloud deck of a gas giant with no ground, and the moons and the rings.

use super::detail::Row;
use super::format::{distance, short};
use super::words::{air, water};
use crate::universe::{PlanetClass, PlanetSpec, Rings};

/// Formats the rings of a planet by their inner and outer radius.
pub fn rings(rings: Rings) -> String {
    format!("{} to {}", distance(rings.inner), distance(rings.outer))
}

/// Returns the surface rows of `planet`. A gas giant has no ground to land on: its air is
/// measured at the cloud deck.
pub fn surface(planet: &PlanetSpec) -> [Row; 2] {
    let record = &planet.record;
    match record.class {
        PlanetClass::GasGiant => [
            Row::Fact("Ground", "none to land on".to_string()),
            Row::Fact("Deck pressure", short(record.pressure.max(0.0))),
        ],
        _ => [
            Row::Fact("Air", air(record.pressure)),
            Row::Fact("Water", water(record)),
        ],
    }
}

/// Returns the rows that count the moons and give the rings of `planet`, for those it has. A
/// gas giant always says how many moons it has.
pub fn company(planet: &PlanetSpec) -> Vec<Row> {
    let giant = planet.record.class == PlanetClass::GasGiant;
    let count = (giant || planet.moons > 0).then(|| Row::Fact("Moons", planet.moons.to_string()));
    let ring = planet.rings.map(|ring| Row::Fact("Rings", rings(ring)));
    count.into_iter().chain(ring).collect()
}
