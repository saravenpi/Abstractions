//! Tests of the workshop: three proposals for each intent, a swap that changes the numbers, and
//! numbers read against the held tool.

use super::bench::{Bench, INTENTS};
use super::offer::{offer, title};
use super::sheet::{sheet, versus};
use crate::craft::stage::{earth, pack};

/// Every intent gets three proposals from the staged pack, each with a name and a reason.
#[test]
fn every_intent_gets_three_proposals() {
    let stock = pack().stock();
    for intent in INTENTS {
        let bench = Bench {
            intent,
            ..Bench::default()
        };
        let shown = offer(&bench, &stock, &earth()).expect("an offer");
        assert!(shown.chosen.is_some(), "{intent:?}");
        for proposal in &shown.three {
            assert!(!title(proposal, &stock).is_empty());
            assert!(!proposal.why.is_empty());
        }
    }
}

/// Swapping the head material makes another build with other numbers.
#[test]
fn a_swap_changes_the_numbers() {
    let stock = pack().stock();
    let plain = offer(&Bench::default(), &stock, &earth()).expect("an offer");
    let base = plain.chosen.expect("chosen");
    let changed = (1..stock.len()).find_map(|by| {
        let bench = Bench {
            swap: [0, by],
            ..Bench::default()
        };
        let swapped = offer(&bench, &stock, &earth()).ok()?.chosen?;
        Some(swapped).filter(|s| s.design.of[1] != base.design.of[1])
    });
    let swapped = changed.expect("a swap that can be made");
    let (a, b) = (
        sheet(&base.build, base.design.intent, &earth()),
        sheet(&swapped.build, swapped.design.intent, &earth()),
    );
    assert_ne!(a, b);
}

/// A number reads plain without a held tool, and with its signed difference against one.
#[test]
fn numbers_read_against_the_held_tool() {
    assert_eq!(versus(0.62, None, " kg", 2), "0.62 kg");
    assert_eq!(versus(0.62, Some(0.72), " kg", 2), "0.62 kg (-0.10)");
    assert_eq!(versus(14.0, Some(12.5), " m/s", 1), "14.0 m/s (+1.5)");
}
