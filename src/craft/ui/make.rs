//! Making the focused tool and treating its head material: the two deeds of the workshop that
//! change what the pack holds, each leaving a note that says what happened.

use super::deeds::Deed;
use super::lines::treated;
use super::offer::{offer, title};
use crate::craft::build::Proposal;
use crate::craft::matter::{Stuff, Treatment};

/// Returns the note after a tool named `name` was made, given the hotbar slot it went into.
pub fn made_note(name: &str, slot: Option<usize>) -> String {
    match slot {
        Some(at) => format!(
            "Made a {name}. It is in slot {}: press {} to hold it.",
            at + 1,
            at + 1
        ),
        None => format!("Made a {name}. The six slots are full: press 1 to 6 to put it in one."),
    }
}

impl Deed<'_> {
    /// Returns the focused proposal after its swaps, if the pack can make it.
    fn chosen(&self) -> Option<Proposal> {
        let stock = self.pack.stock();
        offer(&self.bench, &stock, &self.here.place())
            .ok()
            .and_then(|o| o.chosen)
    }

    /// Makes the focused tool, puts it in a free hotbar slot and says where.
    pub fn make(&mut self) {
        let Some(chosen) = self.chosen() else {
            self.bench.note = "Nothing to make from what is carried.".to_string();
            return;
        };
        let stock = self.pack.stock();
        let Some(id) = self.pack.make(&chosen, &stock) else {
            self.bench.note = "Not enough material for that tool.".to_string();
            return;
        };
        let slot = self.pack.bar.slots.iter().position(|s| *s == Some(id));
        self.bench.note = made_note(&title(&chosen, &stock), slot);
        self.bench.made = Some(id);
        self.bench.swap = [0, 0];
    }

    /// Treats the head material of the focused tool with the first treatment the planet allows
    /// that it has not had, and says what happened.
    pub fn treat(&mut self) {
        let Some(chosen) = self.chosen() else {
            self.bench.note = "Nothing to treat: no tool can be made.".to_string();
            return;
        };
        let stock = self.pack.stock();
        let stuff: Stuff = stock[usize::from(chosen.design.of[1])].stuff;
        let Some(planet) = self.here.planet() else {
            self.bench.note = "No planet underfoot.".to_string();
            return;
        };
        let done = Treatment::ALL
            .into_iter()
            .filter(|t| t.possible(&planet.record))
            .find(|t| self.pack.treat(&stuff, *t));
        self.bench.note = match done {
            Some(t) => format!("Head material {}.", treated(t)),
            None => "No treatment left for this material on this world.".to_string(),
        };
        self.bench.swap = [0, 0];
    }
}
