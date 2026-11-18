//! The readout of the row under the cursor: the whole record of a planet in plain words.

use super::atlas::Atlas;
use super::format::{distance, gravity};
use super::route::Target;
use super::rows::Line;
use super::words::{air, flux, temperature, tilt, water, year, zone};
use crate::saves::play_time;
use crate::universe::{PlanetRecord, PlanetSpec, Universe};

/// Most lines a readout has.
pub const LINES: usize = 6;

/// What a row says in place of the class and the temperature while the record is unknown.
pub const UNCHARTED: &str = "uncharted";

/// Returns the class and the temperature of a planet, or [`UNCHARTED`] without its record.
pub fn brief(record: Option<&PlanetRecord>) -> String {
    match record {
        Some(record) => format!(
            "{} - {}",
            record.class.label(),
            temperature(record.temperature)
        ),
        None => UNCHARTED.to_string(),
    }
}

/// Returns true when the record of `target` is known: it is a planet whose system was
/// visited, the rule the planet count of a star follows.
pub fn charted(atlas: &Atlas, target: Target) -> bool {
    matches!(target, Target::Body(id) if atlas.visited(Target::Star(id.star)))
}

/// Returns, for each row, the planet whose record the computer knows, and nothing for a
/// star, a galaxy or a planet of a system that was not visited.
pub fn worlds(universe: &Universe, atlas: &Atlas, lines: &[Line]) -> Vec<Option<PlanetSpec>> {
    let world = |line: &Line| match line.target {
        Target::Body(id) if charted(atlas, line.target) => Some(universe.planet(id)),
        _ => None,
    };
    lines.iter().map(world).collect()
}

/// Returns the lines of the readout of `line`: the record of `world` when the row is a planet
/// with a known record, and the facts of the row otherwise.
pub fn readout(line: &Line, world: Option<&PlanetSpec>) -> Vec<String> {
    let Some(planet) = world else {
        return vec![line.facts().to_string()];
    };
    let record = &planet.record;
    let (radius, pull) = (distance(planet.radius), gravity(planet.gravity));
    let (day, orbit) = (play_time(planet.day_seconds), year(planet.orbit_seconds));
    vec![
        format!("{} - tilt {}", brief(Some(record)), tilt(record.tilt)),
        format!("air {}", air(record.pressure)),
        format!("water {}", water(record)),
        format!("light {}, zone {}", flux(record.flux), zone(record.zone)),
        format!("radius {radius} - gravity {pull}"),
        format!("day {day} - year {orbit}"),
    ]
}
