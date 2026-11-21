//! Tests of the empty pack on screen: its card writes the mass carried as zero, never as a
//! negative zero.

use super::card::{Shown, pack_card};
use crate::craft::build::Place;
use crate::craft::pack::{Burden, Pack};
use crate::fauna::guide::card::Row;

/// The card of a pack that holds nothing says "0.0 of" the capacity: the sum of no masses is a
/// negative zero, and no row may show its sign.
#[test]
fn an_empty_pack_weighs_zero_without_a_sign() {
    let pack = Pack::default();
    let burden = Burden::of(pack.mass(), 9.81);
    let shown = Shown {
        pack: &pack,
        stock: &[],
        place: Place {
            gravity: 9.81,
            air: 1.225,
        },
        burden,
        held: None,
        treatments: Vec::new(),
    };
    let card = pack_card(None, &shown);
    let carried = card.rows.iter().find_map(|row| match row {
        Row::Fact("Carried", words) => Some(words.clone()),
        _ => None,
    });
    let carried = carried.expect("the card has a Carried row");
    assert!(carried.starts_with("0.0 of "), "{carried}");
    assert!(!carried.contains('-'), "{carried}");
}
