//! The line of the card on life, a stat with the leaf icon: the chance of life on a planet or a
//! moon the player has not flown down to, and what lives there on one they have. Both come from
//! the life model, on the same world it builds when the planet is entered.

use super::detail::Row;
use super::groups::stat;
use crate::life::{Rung, World, odds};
use crate::ui::Icon;
use crate::universe::{BodyId, PlanetSpec, Universe};

/// Returns what lives on a planet whose life reached `rung`, in plain words.
pub fn lives(rung: Option<Rung>) -> &'static str {
    match rung {
        None => "none",
        Some(Rung::Mats) => "microbial mats",
        Some(Rung::Soft) => "soft-bodied life",
        Some(Rung::Shelled) => "shelled life",
        Some(Rung::Skeletal) => "animals with skeletons",
        Some(Rung::Old) => "ancient life",
    }
}

/// Returns a chance from 0 to 1 in whole percent. Only a sure thing says 0 or 100.
pub fn percent(chance: f32) -> String {
    let shown = match chance {
        c if c <= 0.0 => 0,
        c if c >= 1.0 => 100,
        c => ((c * 100.0).round() as u32).clamp(1, 99),
    };
    format!("{shown} %")
}

/// Returns the world of `body` as the life model sees it, around the star of its system.
pub fn world(universe: &Universe, body: &PlanetSpec) -> World {
    let id = body.id.star;
    let star = universe
        .star(id)
        .unwrap_or_else(|| universe.system(id).star);
    World::of(body, &star)
}

/// Returns the line on life of `body`: what lives there once `landed`, and the chance of life
/// before.
pub fn row(universe: &Universe, body: &PlanetSpec, landed: bool) -> Row {
    let world = world(universe, body);
    match landed {
        true => {
            let living = lives(odds::rung(&world, body.seed)).to_string();
            stat(Icon::Life, "Life", living, String::new())
        }
        false => {
            let chance = percent(odds::chance(&world));
            stat(Icon::Life, "Chance of life", chance, String::new())
        }
    }
}

/// Returns what the line on life means for the player, in one plain sentence.
pub fn meaning(universe: &Universe, body: &PlanetSpec, landed: bool) -> Row {
    let world = world(universe, body);
    let words = match landed {
        true if odds::rung(&world, body.seed).is_none() => "Barren: nothing lives here",
        true => "Alive: go down and look",
        false => match odds::chance(&world) {
            c if c < 0.05 => "Life is very unlikely",
            c if c < 0.3 => "Life is unlikely",
            c if c < 0.6 => "Life is possible",
            _ => "Life is likely",
        },
    };
    Row::Meaning(words.to_string())
}

/// The line on life of the last body shown with the sentence under it, kept while the same body
/// stays selected.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Odds {
    /// The body, its seed and whether it was landed on, when the lines were worked out.
    key: Option<(BodyId, u64, bool)>,
    /// The lines worked out for it.
    rows: Option<[Row; 2]>,
    /// How many times the lines were worked out, for the cost tests.
    pub worked: usize,
}

impl Odds {
    /// Returns the line on life of `body` and its meaning, working them out again only for
    /// another body or after a landing.
    pub fn rows(&mut self, universe: &Universe, body: &PlanetSpec, landed: bool) -> [Row; 2] {
        let key = Some((body.id, body.seed, landed));
        match (&self.rows, self.key == key) {
            (Some(rows), true) => rows.clone(),
            _ => {
                let rows = [row(universe, body, landed), meaning(universe, body, landed)];
                (self.key, self.rows, self.worked) = (key, Some(rows.clone()), self.worked + 1);
                rows
            }
        }
    }
}
