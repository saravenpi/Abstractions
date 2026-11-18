//! The lines of a readout that change with the kind of world: the ground, the air and the
//! water of a solid planet, or the cloud deck, the moons and the rings of a gas giant.

use super::format::{distance, short};
use super::words::{air, water};
use crate::universe::{PlanetClass, PlanetSpec, Rings};

/// Formats a number of moons, such as `no moons`, `1 moon` or `4 moons`.
pub fn moons(count: u8) -> String {
    match count {
        0 => "no moons".to_string(),
        1 => "1 moon".to_string(),
        n => format!("{n} moons"),
    }
}

/// Formats the rings of a planet by their inner and outer radius, or says there are none.
pub fn rings(rings: Option<Rings>) -> String {
    rings.map_or_else(
        || "no rings".to_string(),
        |rings| {
            format!(
                "rings {} to {}",
                distance(rings.inner),
                distance(rings.outer)
            )
        },
    )
}

/// Returns the second and the third line of the readout of `planet`. A gas giant has no
/// ground to land on: its air is measured at the cloud deck, and its moons and rings take
/// the place of the water.
pub fn surface(planet: &PlanetSpec) -> [String; 2] {
    let record = &planet.record;
    match record.class {
        PlanetClass::GasGiant => [
            format!(
                "no ground to land on - deck pressure {}",
                short(record.pressure.max(0.0))
            ),
            format!("{} - {}", moons(planet.moons), rings(planet.rings)),
        ],
        _ => [
            format!("air {}", air(record.pressure)),
            format!("water {}", water(record)),
        ],
    }
}

/// Returns what ends the size line of a solid planet: its moons and its rings when it has
/// any, and nothing for a gas giant, whose readout already says them.
pub fn company(planet: &PlanetSpec) -> String {
    if planet.record.class == PlanetClass::GasGiant {
        return String::new();
    }
    let count = (planet.moons > 0).then(|| format!(" - {}", moons(planet.moons)));
    let ring = planet.rings.map(|_| " - rings".to_string());
    count.into_iter().chain(ring).collect()
}
