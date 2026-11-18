//! The readout of the row under the cursor: the whole record of a planet or a moon in plain
//! words.

use super::atlas::Atlas;
use super::format::{distance, gravity};
use super::giant::{company, surface};
use super::route::Target;
use super::rows::Line;
use super::words::{flux, temperature, tilt, year, zone};
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

/// Returns true when the record of `target` is known: it is a planet or a moon whose system
/// was visited, the rule the planet count of a star follows.
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

/// Returns what to do to learn more about a row the computer knows little about.
pub fn advice(target: Target, visited: bool) -> Option<&'static str> {
    match (target, visited) {
        (Target::Body(_), _) => Some("Visit this system to chart its planets"),
        (Target::Star(_), false) => Some("Visit this system to chart its planets"),
        (Target::Galaxy(_), false) => Some("Visit this galaxy to chart its systems"),
        _ => None,
    }
}

/// Returns the lines of the readout of `line`: the record of `world` when the row is a planet
/// or a moon with a known record, and otherwise the facts of the row and what to do to learn
/// more. The year of a moon is its turn around its planet.
pub fn readout(line: &Line, world: Option<&PlanetSpec>) -> Vec<String> {
    let Some(planet) = world else {
        let known = std::iter::once(line.facts().to_string());
        let hint = advice(line.target, line.visited).map(str::to_string);
        return known.chain(hint).collect();
    };
    let record = &planet.record;
    let (radius, pull) = (distance(planet.radius), gravity(planet.gravity));
    let (day, orbit) = (
        play_time(planet.day_seconds.abs()),
        year(planet.orbit_seconds),
    );
    let turn = if planet.id.is_moon() { "orbit" } else { "year" };
    let [ground, sea] = surface(planet);
    vec![
        format!("{} - tilt {}", brief(Some(record)), tilt(record.tilt)),
        ground,
        sea,
        format!("light {}, zone {}", flux(record.flux), zone(record.zone)),
        format!("radius {radius} - gravity {pull}{}", company(planet)),
        format!("day {day} - {turn} {orbit}"),
    ]
}
