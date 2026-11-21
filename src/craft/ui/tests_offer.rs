//! Tests of the workshop: every intent shows three tool cards or says what the pack lacks, a swap
//! changes the numbers, and the bars read the numbers.

use super::bench::{Bench, INTENTS, Pane};
use super::offer::offer;
use super::scene::{Ground, Scene, scene, status};
use super::sheet::{Sheet, gauges, sheet};
use crate::craft::build::Intent;
use crate::craft::matter::Class;
use crate::craft::pack::{Burden, Pack};
use crate::craft::stage::{earth, pack, tissue};

/// Returns the scene of `pack` with the workshop asked for `intent`, on Earth.
fn shown(pack: &Pack, intent: Intent) -> Scene {
    let bench = Bench {
        pane: Pane::Workshop,
        intent,
        ..Bench::default()
    };
    let ground = Ground {
        place: earth(),
        life: None,
        burden: Burden::of(pack.mass(), 9.81),
    };
    scene(&bench, pack, &ground)
}

/// Every intent gets three cards from the staged pack, one focused, each with a name, a reason,
/// the parts it takes and four bars inside their range.
#[test]
fn every_intent_shows_three_cards() {
    for intent in INTENTS {
        let tools = shown(&pack(), intent).tools.expect("three cards");
        assert_eq!(tools.len(), 3, "{intent:?}");
        assert_eq!(tools.iter().filter(|t| t.focused).count(), 1);
        for tool in &tools {
            assert!(
                !tool.name.is_empty() && !tool.reason.words.is_empty(),
                "{tool:?}"
            );
            assert!(tool.parts.starts_with("Uses "), "{tool:?}");
            assert!(tool.shape.is_some());
            assert!(tool.gauges.iter().all(|g| (0.0..=1.0).contains(&g.share)));
        }
    }
}

/// An empty pack says, for every intent, how to get stone; a pack of hide alone says that nothing
/// holds an edge when asked to cut.
#[test]
fn an_intent_with_no_card_says_why() {
    for intent in INTENTS {
        let why = shown(&Pack::default(), intent).tools.expect_err("no card");
        assert!(why.contains("press H"), "{intent:?}: {why}");
    }
    let mut hide = Pack::default();
    hide.stow(tissue(Class::Hide, 2, 1.0), f32::INFINITY);
    let why = shown(&hide, Intent::Cut).tools.expect_err("no edge");
    assert!(why.contains("edge"), "{why}");
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

/// The bars of a tool read its reach, mass, depth and swing against their full scale, with the
/// word for what its head does.
#[test]
fn the_bars_read_the_numbers() {
    let numbers = Sheet {
        mass: 1.5,
        reach: 0.8,
        speed: 10.0,
        depth: 1.0,
        life: 100.0,
    };
    let bars = gauges(&numbers, Intent::Cut);
    let labels: Vec<&str> = bars.iter().map(|g| g.label).collect();
    assert_eq!(labels, ["Reach", "Mass", "Edge", "Swing"]);
    assert!(
        bars.iter().all(|g| (g.share - 0.5).abs() < 1e-6),
        "{bars:?}"
    );
    assert_eq!(bars[0].value, "0.80 m");
    assert_eq!(bars[2].value, "50 %");
    assert_eq!(gauges(&numbers, Intent::Crush)[2].label, "Impact");
}

/// The load of an empty pack reads zero, never a negative zero.
#[test]
fn an_empty_pack_weighs_zero_without_a_sign() {
    let words = status(&Burden::of(Pack::default().mass(), 9.81));
    assert!(words.starts_with("0.0 of "), "{words}");
    assert!(!words.contains('-'), "{words}");
}
