//! The rows of the card of each kind of place: the record of a planet or a moon in groups,
//! the size of one not yet charted, and the facts of a star and of a galaxy.

use super::detail::{Row, advice};
use super::format::{distance, gravity};
use super::giant::{company, surface};
use super::route::Target;
use super::rows::Line;
use super::words::{flux, temperature, tilt, year, zone};
use crate::saves::play_time;
use crate::universe::{GalaxyId, PlanetSpec, StarId, Universe};

/// Returns the row that says how far the place of `line` is from the ship.
fn away(line: &Line) -> Row {
    Row::Fact("From you", distance(line.distance))
}

/// Returns the rows of the size and the pull of `body`, with its moons and rings.
fn body(planet: &PlanetSpec) -> Vec<Row> {
    let size = [
        Row::Heading("Body"),
        Row::Fact("Radius", distance(planet.radius)),
        Row::Fact("Gravity", gravity(planet.gravity)),
    ];
    size.into_iter().chain(company(planet)).collect()
}

/// Returns the rows of the record of a planet or a moon whose record is known: how far it is,
/// its surface, its climate, its body and its time. The year of a moon is its turn around
/// its planet.
pub fn planet(line: &Line, planet: &PlanetSpec) -> Vec<Row> {
    let record = &planet.record;
    let turn = if planet.id.is_moon() { "Orbit" } else { "Year" };
    let climate = [
        Row::Heading("Climate"),
        Row::Fact("Temperature", temperature(record.temperature)),
        Row::Fact("Light", flux(record.flux)),
        Row::Fact("Zone", zone(record.zone).to_string()),
        Row::Fact("Tilt", tilt(record.tilt)),
    ];
    let time = [
        Row::Heading("Time"),
        Row::Fact("Day", play_time(planet.day_seconds.abs())),
        Row::Fact(turn, year(planet.orbit_seconds)),
    ];
    let mut rows = vec![away(line), Row::Heading("Surface")];
    rows.extend(surface(planet));
    rows.extend(climate);
    rows.extend(body(planet));
    rows.extend(time);
    rows
}

/// Returns the rows of a planet or a moon whose record is not known: how far it is, its size
/// and what to do to learn more.
pub fn uncharted(line: &Line, planet: &PlanetSpec) -> Vec<Row> {
    let hint = advice(line.target, line.visited).map(Row::Note);
    let size = [
        Row::Heading("Body"),
        Row::Fact("Radius", distance(planet.radius)),
        Row::Fact("Gravity", gravity(planet.gravity)),
    ];
    let mut rows = vec![away(line)];
    rows.extend(size.into_iter().chain(hint));
    rows
}

/// Returns the subtitle and the rows of the star `id`: how far it is, its heat, light and
/// size, and its planets once its system is visited.
pub fn star(line: &Line, universe: &Universe, id: StarId) -> (String, Vec<Row>) {
    let galaxy = Target::Galaxy(id.galaxy).name(universe);
    let Some(spec) = universe.star(id) else {
        return (format!("Star in {galaxy}"), vec![away(line)]);
    };
    let planets = match line.visited {
        true => universe.system(id).planets.len().to_string(),
        false => "unknown".to_string(),
    };
    let rows = [
        away(line),
        Row::Heading("Star"),
        Row::Fact("Temperature", temperature(spec.temperature)),
        Row::Fact("Light", flux(spec.luminosity)),
        Row::Fact("Radius", distance(spec.radius)),
        Row::Heading("System"),
        Row::Fact("Planets", planets),
    ];
    let hint = advice(line.target, line.visited).map(Row::Note);
    let title = format!("Class {:?} star in {galaxy}", spec.class);
    (title, rows.into_iter().chain(hint).collect())
}

/// Returns the subtitle and the rows of the galaxy `id`: how far it is, its size and its arms.
pub fn galaxy(line: &Line, universe: &Universe, id: GalaxyId) -> (String, Vec<Row>) {
    let Some(spec) = universe.galaxy(id) else {
        return ("Galaxy".to_string(), vec![away(line)]);
    };
    let kind = format!("{:?}", spec.kind);
    let arms = (spec.arms > 0).then(|| Row::Fact("Arms", spec.arms.to_string()));
    let size = [
        away(line),
        Row::Heading("Galaxy"),
        Row::Fact("Radius", distance(spec.radius)),
    ];
    let hint = advice(line.target, line.visited).map(Row::Note);
    let rows = size.into_iter().chain(arms).chain(hint).collect();
    (format!("{kind} galaxy"), rows)
}
