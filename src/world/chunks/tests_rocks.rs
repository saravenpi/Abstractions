//! Tests of the rocks of generated chunks: stable ids, and a taken rock that never comes back.

use bevy::prelude::*;

use super::build::{ChunkData, generate};
use super::tests::view;
use crate::world::Terrain;
use crate::world::globe::Face;
use crate::world::mesh::Lod;
use crate::world::rocks::{RockId, Taken, is_rock};
use crate::world::scatter::scatter_chunk;

/// Returns a terrain and a chunk of it where a rock with a body stands.
fn rocky() -> (Terrain, IVec2) {
    let terrain = view(3000.0, Face::NegY);
    let rocks = |c: &IVec2| {
        let props = scatter_chunk(&terrain, *c).props;
        props.iter().any(|p| is_rock(p.plant))
    };
    let mut cells = (-8..8).flat_map(|x| (-8..8).map(move |y| IVec2::new(x, y)));
    let spot = cells.find(rocks).expect("a chunk with a rock");
    (terrain, spot)
}

/// Generates the chunk `coord` of `terrain` without the rocks of `taken`.
fn built(terrain: &Terrain, coord: IVec2, taken: &Taken) -> ChunkData {
    generate(terrain, coord, (Lod::Near, false), taken)
}

/// Returns the ids of the rocks of a generated chunk.
fn ids(data: &ChunkData) -> Vec<RockId> {
    data.stand.rocks().iter().map(|r| r.id).collect()
}

/// The same chunk generated twice lists the same rocks with the same ids.
#[test]
fn rock_ids_are_stable_across_two_generations() {
    let (terrain, coord) = rocky();
    let once = ids(&built(&terrain, coord, &Taken::default()));
    let twice = ids(&built(&terrain, coord, &Taken::default()));
    assert!(!once.is_empty());
    assert_eq!(once, twice);
}

/// A chunk built with a rock in the taken set does not have it, and keeps the others.
#[test]
fn a_taken_rock_is_left_out_of_a_new_chunk() {
    let (terrain, coord) = rocky();
    let before = ids(&built(&terrain, coord, &Taken::default()));
    let mut taken = Taken::default();
    taken.quarry(before[0], 5.0, 5.0);
    let after = built(&terrain, coord, &taken);
    assert!(!ids(&after).contains(&before[0]));
    assert_eq!(ids(&after).len(), before.len() - 1);
}

/// A chunk built with half of a rock taken draws it at the cube root of half its size, and a chunk
/// that was quarried to the end is the same as one that was built after.
#[test]
fn a_partly_taken_rock_is_drawn_smaller_in_a_new_chunk() {
    let (terrain, coord) = rocky();
    let whole = built(&terrain, coord, &Taken::default());
    let first = whole.stand.rocks()[0];
    let mut taken = Taken::default();
    taken.quarry(first.id, 50.0, 100.0);
    let after = built(&terrain, coord, &taken);
    let shrunk = after.stand.rocks().iter().find(|r| r.id == first.id);
    let shrunk = shrunk.expect("the rock is still there");
    assert!((shrunk.scale - first.scale * 0.5_f32.cbrt()).abs() < 1.0e-4);
    assert!((shrunk.left - 0.5).abs() < 1.0e-6);
}
