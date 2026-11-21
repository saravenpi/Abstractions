//! The one reason a proposal is good or bad, and what the pack lacks when the workshop has nothing
//! to propose, in plain words.

use super::sheet::{blows, sheet};
use crate::craft::build::{EDGE_MIN, Intent, Place, Proposal, Stock, Why};
use crate::ui::units::fixed;

/// The one reason that matters about a proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct Reason {
    /// Whether the reason is a flaw.
    pub bad: bool,
    /// The reason in words, starting with a capital.
    pub words: String,
}

/// Returns `words` with its first letter in capitals.
fn capital(words: &str) -> String {
    let mut letters = words.chars();
    letters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(letters).collect()
    })
}

/// Returns the reason of `proposal`, the proposal at `pick`: its flaw when it has one, else what it
/// is best at, at `place`.
pub fn reason(proposal: &Proposal, pick: usize, place: &Place) -> Reason {
    let flaw = proposal
        .why
        .iter()
        .find(|why| !matches!(why, Why::WeakestJoint(_)));
    if let Some(flaw) = flaw {
        return Reason {
            bad: true,
            words: capital(&flaw.to_string()),
        };
    }
    let numbers = sheet(&proposal.build, proposal.design.intent, place);
    let weakest = proposal
        .why
        .last()
        .map_or(String::new(), |why| why.to_string());
    let words = match pick {
        0 => format!("Light at {} kg: quick to swing", fixed(numbers.mass, 2)),
        1 => format!(
            "Hits hardest: {} of 2 layers, shell then hide",
            fixed(numbers.depth, 1)
        ),
        _ => format!("Lasts {} blows, {weakest}", fixed(blows(numbers.life), 0)),
    };
    Reason { bad: false, words }
}

/// Returns what the pack lacks for `intent`, given `stock`, and how to get it.
pub fn missing(intent: Intent, stock: &[Stock]) -> String {
    if stock.is_empty() {
        return "The pack is empty. Look at a rock and press H to break stone off it, \
                or carve a kill for bone and hide."
            .to_string();
    }
    let sharp = intent != Intent::Crush;
    let hard = stock.iter().any(|s| s.material.hardness >= EDGE_MIN);
    match (sharp, hard) {
        (true, false) => "Nothing carried holds an edge or a point. Break flint, obsidian or \
                          basalt off a rock with H, or carve bone from a kill."
            .to_string(),
        _ => "Too little material for a head and a shaft. Break more stone off a rock with H, \
              or carve a kill."
            .to_string(),
    }
}
