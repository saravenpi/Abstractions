//! Tests of item recipes: their size on the wire and the stats they rebuild.

use abstractions_net::wire::{decode, encode};

use super::rig::{earth, stock};
use super::*;
use crate::craft::matter::Stuff;

/// A proposal stored as a recipe survives the wire byte for byte in at most 72 bytes (68 with three
/// tissues of one species), and rebuilds the same tool, its stats recomputed from the bytes.
#[test]
fn recipes_round_trip_small() {
    let carried = stock();
    let found = solve(Intent::Cut, &carried, &earth()).expect("a cut build");
    let mut recipe = found.durable.recipe(&carried);
    recipe.wear.hit();
    let bytes = encode(&recipe).expect("it encodes");
    let (back, used): (Recipe, usize) = decode(&bytes, 1024).expect("it decodes").expect("whole");
    assert_eq!((back.clone(), used), (recipe, bytes.len()));
    assert!(bytes.len() <= 72, "{} bytes", bytes.len());
    let lookup = |s: &Stuff| carried.iter().find(|c| c.stuff == *s).map(|c| c.material);
    let rebuilt = back.build(lookup).expect("its stuff is carried");
    assert_eq!(rebuilt, found.durable.build);
    assert_eq!(swing(&rebuilt, &earth()), found.durable.swing);
}
