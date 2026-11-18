//! Tests that no structure stands in a river bed: a bed is shallow water to every placer.

use bevy::prelude::*;

use super::rules::fit;
use super::site::Kind;
use crate::world::scatter::tests_river::{river_chunks, rivers};

/// On chunks a river runs through, no kind of structure fits with its centre on a river bed point, nor with
/// the point of the outer ring of its footprint that the surface check skips lying on one.
#[test]
fn nothing_stands_in_a_river() {
    let found = river_chunks(&rivers(), 6);
    assert!(found.len() >= 3, "only {} river chunks", found.len());
    let kinds = Kind::ALL.into_iter().filter(|kind| *kind != Kind::Village);
    for kind in kinds {
        let edge = Vec2::X * kind.spec().radius * 0.9;
        for (tile, beds) in &found {
            let sites = beds.iter().flat_map(|b| [*b, *b - edge]);
            let fits = sites.filter(|p| fit(&tile.terrain, kind, *p).is_some());
            assert_eq!(fits.count(), 0, "a {kind:?} in a river");
        }
    }
}
