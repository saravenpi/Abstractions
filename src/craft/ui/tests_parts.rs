//! Tests that the focused tool card lights its parts in the pack: in the workshop pane the lit rows
//! are exactly the materials the card takes, with the kilograms it takes; in the pack pane only the
//! cursor row is lit.

use super::bench::{Bench, INTENTS, Pane};
use super::lines::What;
use super::offer::offer;
use super::scene::{Ground, scene};
use crate::craft::pack::{Burden, cost};
use crate::craft::stage::{earth, pack};

/// Returns the ground of the staged pack on Earth.
fn ground(mass: f32) -> Ground<'static> {
    Ground {
        place: earth(),
        life: None,
        burden: Burden::of(mass, 9.81),
    }
}

/// For every intent and every card, the rows the focused card lights are the stacks its build
/// takes from, each with the mass it takes.
#[test]
fn a_focused_card_lights_its_parts() {
    let carried = pack();
    let stock = carried.stock();
    for intent in INTENTS {
        for pick in 0..3 {
            let bench = Bench {
                pane: Pane::Workshop,
                intent,
                pick,
                ..Bench::default()
            };
            let chosen = offer(&bench, &stock, &earth()).expect("an offer").chosen;
            let taken = cost(&chosen.expect("a chosen card"), &stock);
            let shown = scene(&bench, &carried, &ground(carried.mass()));
            let lit: Vec<(usize, f32)> = shown
                .rows
                .iter()
                .filter(|row| row.lit)
                .map(|row| match row.line.what {
                    What::Stack(at) => (at, row.uses.expect("a lit row says what it gives")),
                    What::Item(_) => panic!("a tool lit as a part"),
                })
                .collect();
            assert!(!lit.is_empty(), "{intent:?} {pick}");
            for (at, kg) in &taken {
                assert!(lit.contains(&(*at, *kg)), "{intent:?} {pick}: {at} {kg}");
            }
            assert_eq!(lit.len(), taken.len(), "{intent:?} {pick}");
        }
    }
}

/// In the pack pane the cursor row alone is lit, and no row says what a card takes.
#[test]
fn the_pack_pane_lights_its_cursor() {
    let carried = pack();
    let bench = Bench {
        cursor: 2,
        ..Bench::default()
    };
    let shown = scene(&bench, &carried, &ground(carried.mass()));
    let lit: Vec<usize> = (0..shown.rows.len())
        .filter(|at| shown.rows[*at].lit)
        .collect();
    assert_eq!(lit, [2]);
    assert!(shown.rows.iter().all(|row| row.uses.is_none()));
}
