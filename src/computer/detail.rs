//! The card of the row under the cursor: the name of the place, what it is, and its facts in
//! small groups. A planet shows four groups of two stats, each with an icon, a label, a value
//! with its unit, a comparison with home and a plain meaning, and keeps its rarer facts behind a
//! More row.

use super::atlas::Atlas;
use super::facts::{galaxy, planet, star, uncharted};
use super::route::Target;
use super::rows::Line;
use super::words::temperature;
use crate::ui::Icon;
use crate::universe::{PlanetRecord, PlanetSpec, Universe};

/// What a row says in place of the class and the temperature while the record is unknown.
pub const UNCHARTED: &str = "uncharted";

/// One row of a card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    /// The heading of the group of facts under it, with the icon of the group.
    Heading(Icon, &'static str),
    /// A sentence on its own, such as what to do to learn more.
    Note(&'static str),
    /// One measure of a planet: an icon, a label, a value with its unit and a few words, such
    /// as the comparison with home or what the value means for the player.
    Stat {
        /// The icon of the measure.
        icon: Icon,
        /// What is measured.
        label: &'static str,
        /// The value, always with its unit.
        value: String,
        /// The comparison with home or one to three words of meaning, or nothing.
        note: String,
    },
    /// What the group above means for the player, in one plain line.
    Meaning(String),
    /// The row that shows or hides the rarer facts. Every row after it is one of them.
    More,
}

/// Returns the rows of a card as they are shown: all of them when `open`, and otherwise those
/// up to and with the [`Row::More`] row.
pub fn visible(rows: &[Row], open: bool) -> &[Row] {
    match (open, rows.iter().position(|row| matches!(row, Row::More))) {
        (false, Some(at)) => &rows[..=at],
        _ => rows,
    }
}

/// The card of a place: its name, one line on what it is and where, and its rows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Card {
    /// The name of the place.
    pub title: String,
    /// What the place is and what it belongs to.
    pub subtitle: String,
    /// The facts, grouped under their headings.
    pub rows: Vec<Row>,
}

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
        (Target::Body(_), _) => Some("Not visited. Fly there to learn its air, water and climate."),
        (Target::Star(_), false) => Some("Not visited. Fly there to learn its planets."),
        (Target::Galaxy(_), false) => Some("Not visited. Fly there to learn its systems."),
        _ => None,
    }
}

/// Returns `words` with its first letter in capitals.
fn capital(words: &str) -> String {
    let mut letters = words.chars();
    letters.next().map_or(String::new(), |first| {
        first.to_uppercase().chain(letters).collect()
    })
}

/// Returns the subtitle of a planet or a moon: its class when its record is known, what it is
/// and what it turns around.
fn subtitle(universe: &Universe, body: &PlanetSpec, known: bool) -> String {
    let id = body.id;
    let (noun, around) = match id.is_moon() {
        true => ("moon", universe.planet(id.planet()).name),
        false => ("planet", Target::Star(id.star).name(universe)),
    };
    let kind = match (known, body.record.class.label()) {
        (false, _) => format!("{UNCHARTED} {noun}"),
        (true, giant) if giant.ends_with("giant") => giant.to_string(),
        (true, class) => format!("{class} {noun}"),
    };
    capital(&format!("{kind} of {around}"))
}

/// Returns the card of `line`: the whole record of `world` for a planet or a moon whose record
/// is known, its size and what to do to learn more for one without, and otherwise the facts
/// of the star or the galaxy.
pub fn card(line: &Line, world: Option<&PlanetSpec>, universe: &Universe) -> Card {
    let home = || universe.planet(universe.home());
    let (subtitle, rows) = match (line.target, world) {
        (Target::Body(_), Some(body)) => (subtitle(universe, body, true), planet(body, &home())),
        (Target::Body(id), None) => {
            let body = universe.planet(id);
            (
                subtitle(universe, &body, false),
                uncharted(line, &body, &home()),
            )
        }
        (Target::Star(id), _) => star(line, universe, id),
        (Target::Galaxy(id), _) => galaxy(line, universe, id),
    };
    Card {
        title: line.title.clone(),
        subtitle,
        rows,
    }
}
