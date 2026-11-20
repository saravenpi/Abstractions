//! The rows of the card of each kind of place: the record of a planet or a moon in groups, the
//! size of one not yet charted, and the facts of a star and of a galaxy. How far a place is
//! shows in its row of the list, not on its card.

use super::detail::{Row, advice};
use super::groups::{climate, rarer, size, time, wet_group};
use super::route::Target;
use super::rows::Line;
use super::words::{flux, temperature};
use crate::ui::units::distance;
use crate::universe::{GalaxyId, PlanetSpec, StarId, Universe};

/// Returns the rows of the record of a planet or a moon whose record is known, against the home
/// planet `home`: four groups, size and gravity, air and heat, day and year, water and life,
/// then the More row and the rarer facts behind it. The line on life goes at the end of the
/// last group, where the screen puts it. How far the place is shows in its row of the list.
pub fn planet(planet: &PlanetSpec, home: &PlanetSpec) -> Vec<Row> {
    let mut rows = size(planet, home, true);
    rows.extend(climate(planet));
    rows.extend(time(planet, home));
    rows.extend(wet_group(planet));
    rows.extend(rarer(planet));
    rows
}

/// Returns the rows of a planet or a moon whose record is not known: its size and pull, and what
/// to do to learn more.
pub fn uncharted(line: &Line, planet: &PlanetSpec, home: &PlanetSpec) -> Vec<Row> {
    let hint = advice(line.target, line.visited).map(Row::Note);
    size(planet, home, false).into_iter().chain(hint).collect()
}

/// Returns the subtitle and the rows of the star `id`: its heat, light and size, and its planets
/// once its system is visited.
pub fn star(line: &Line, universe: &Universe, id: StarId) -> (String, Vec<Row>) {
    let galaxy = Target::Galaxy(id.galaxy).name(universe);
    let Some(spec) = universe.star(id) else {
        return (format!("Star in {galaxy}"), Vec::new());
    };
    let planets = match line.visited {
        true => universe.system(id).planets.len().to_string(),
        false => "unknown".to_string(),
    };
    let rows = [
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

/// Returns the subtitle and the rows of the galaxy `id`: its size and its arms.
pub fn galaxy(line: &Line, universe: &Universe, id: GalaxyId) -> (String, Vec<Row>) {
    let Some(spec) = universe.galaxy(id) else {
        return ("Galaxy".to_string(), Vec::new());
    };
    let kind = format!("{:?}", spec.kind);
    let arms = (spec.arms > 0).then(|| Row::Fact("Arms", spec.arms.to_string()));
    let size = [
        Row::Heading("Galaxy"),
        Row::Fact("Radius", distance(spec.radius)),
    ];
    let hint = advice(line.target, line.visited).map(Row::Note);
    let rows = size.into_iter().chain(arms).chain(hint).collect();
    (format!("{kind} galaxy"), rows)
}
