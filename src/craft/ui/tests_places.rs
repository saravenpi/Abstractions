//! Tests that a tool shows one picture wherever it shows: the look of its workshop card is the
//! look of its row in the pack, of its slot on the pack screen and of the item the hotbar of the
//! game draws.

use super::bench::{Bench, INTENTS, Pane};
use super::glyph::handle;
use super::lines::What;
use super::offer::offer;
use super::raster::raster;
use super::scene::{Ground, look, scene};
use crate::craft::build::{Intent, Look};
use crate::craft::pack::Burden;
use crate::craft::stage::{earth, pack};

/// Returns the ground of a pack of `mass` kilograms on Earth.
fn ground(mass: f32) -> Ground<'static> {
    Ground {
        place: earth(),
        life: None,
        burden: Burden::of(mass, 9.81),
    }
}

/// Returns the look the workshop card of the first proposal for `intent` shows, then the looks of
/// the tool once made: in its pack row, in its slot of the pack screen and on the item the hotbar
/// of the game draws.
fn places(intent: Intent) -> [Option<Look>; 4] {
    let mut carried = pack();
    let stock = carried.stock();
    let bench = Bench {
        pane: Pane::Workshop,
        intent,
        ..Bench::default()
    };
    let ground = ground(carried.mass());
    let card = scene(&bench, &carried, &ground).tools.expect("cards")[0].shape;
    let offered = offer(&bench, &stock, &earth()).ok();
    let chosen = offered.and_then(|o| o.chosen).expect("a card");
    assert_eq!(card, Some(look(&chosen, &stock)));
    let id = carried.make(&chosen, &stock).expect("made");
    let shown = scene(&bench, &carried, &ground);
    let row = shown
        .rows
        .iter()
        .find(|row| row.line.what == What::Item(id));
    let slot = carried.bar.slots.iter().position(|s| *s == Some(id));
    let item = carried.item(id).and_then(|item| item.shape());
    let slotted = slot.and_then(|at| shown.slots[at].shape);
    [card, row.and_then(|row| row.line.shape), slotted, item]
}

/// A tool made in the workshop has one look on its card, in its row of the pack and on both
/// hotbars, so one picture at each size, kept under one handle per size.
#[test]
fn a_tools_icon_is_identical_in_the_three_places() {
    for intent in INTENTS {
        let [card, row, slot, hud] = places(intent);
        assert!(card.is_some_and(|look| look.figure.is_some()), "{intent:?}");
        assert_eq!([row, slot, hud], [card; 3], "{intent:?}");
        let card = card.expect("a look");
        assert_eq!(raster(&card, 24), raster(&card, 24));
        assert_ne!(handle(&card, 24, 0), handle(&card, 32, 0));
        assert_ne!(handle(&card, 16, 0), handle(&card, 16, 1));
    }
}
