//! The list of the guide: every species of the planet, plants first by growth architecture, then
//! animals from the bottom of the food web up. A species not yet seen shows as unknown.

use super::seen::Seen;
use super::words::kind;
use crate::life::{Guild, Life, Species};

/// What an unseen species is called.
pub const UNKNOWN: &str = "Unknown species";

/// What the guide says on a planet without life.
pub const LIFELESS: &str = "No life found on this planet";

/// One row of the list.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// Index of the species in [`Life::species`].
    pub index: u16,
    /// Its name, or [`UNKNOWN`].
    pub name: String,
    /// What it is: its growth for a plant, its guild for an animal, empty while unseen.
    pub kind: &'static str,
    /// Whether the player has seen it.
    pub seen: bool,
}

/// The list of the planet and how much of it was seen.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Book {
    /// Every species, in list order.
    pub entries: Vec<Entry>,
    /// How many were seen.
    pub seen: usize,
}

impl Book {
    /// Returns the count line, such as "12 of 31 seen", or [`LIFELESS`] for an empty list.
    pub fn tally(&self) -> String {
        match self.entries.len() {
            0 => LIFELESS.to_string(),
            n => format!("{} of {n} seen", self.seen),
        }
    }
}

/// Returns the place of `species` in the list: plants by growth architecture, then animals by
/// trophic level, ties kept in web order.
fn order(species: &Species) -> (bool, u8, i32, u16) {
    let animal = species.role.guild != Guild::Producer;
    let archetype = if animal { 0 } else { species.archetype };
    let level = (species.role.level * 100.0) as i32;
    (animal, archetype, level, species.id.index)
}

/// Returns the list row of `species`.
fn entry(species: &Species, seen: &Seen) -> Entry {
    let known = seen.has(species.id);
    let name = if known { &species.name } else { UNKNOWN };
    Entry {
        index: species.id.index,
        name: name.to_string(),
        kind: match known {
            true => kind(species.role.guild, species.archetype),
            false => "",
        },
        seen: known,
    }
}

/// Returns the list of `life`, empty on a dead planet.
pub fn book(life: Option<&Life>, seen: &Seen) -> Book {
    let Some(life) = life else {
        return Book::default();
    };
    let mut species: Vec<&Species> = life.species.iter().collect();
    species.sort_by_key(|s| order(s));
    Book {
        entries: species.into_iter().map(|s| entry(s, seen)).collect(),
        seen: seen.count(life),
    }
}
