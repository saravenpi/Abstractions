//! Tests of the rule of the ground: heaps of one material merge, a full cube refuses, and a take
//! gets a tool whole or the part of a heap asked.

use super::{CELL_MAX, MERGE, Pile, Placed};
use crate::ids::BodyId;
use crate::log::PlayerId;
use crate::wire::{HEAP, Lying, TOOL};

/// Returns a thing of `kind` made of `material`, of `mass` kilograms, at `x` metres along the
/// first axis of the default body.
pub(crate) fn thing(kind: u8, material: u8, mass: f32, x: f64) -> Lying {
    Lying {
        id: 0,
        by: PlayerId(1),
        tag: 1,
        body: BodyId::default(),
        at: [x, 6_400_000.0, 0.0],
        kind,
        material: vec![material; 4],
        mass,
        wear: 0.0,
        item: vec![9; 24],
        since: 10.0,
    }
}

/// Returns what `placed` names, or fails.
fn laid(placed: Placed) -> Lying {
    match placed {
        Placed::Laid(lying) | Placed::Joined(lying) => lying,
        Placed::Refused => panic!("refused"),
    }
}

/// Two heaps of one material put down together are one heap with both masses; a heap of another
/// material, or one farther than the merge distance, lies on its own.
#[test]
fn heaps_of_one_material_merge() {
    let mut pile = Pile::default();
    let first = laid(pile.lay(thing(HEAP, 1, 1.5, 0.0)));
    let joined = pile.lay(thing(HEAP, 1, 0.5, 0.5));
    let Placed::Joined(heap) = joined else {
        panic!("not joined: {joined:?}");
    };
    assert_eq!((heap.id, heap.mass), (first.id, 2.0));
    assert!(matches!(
        pile.lay(thing(HEAP, 2, 1.0, 0.5)),
        Placed::Laid(_)
    ));
    let far = thing(HEAP, 1, 1.0, MERGE + 0.5);
    assert!(matches!(pile.lay(far), Placed::Laid(_)));
    assert!(matches!(
        pile.lay(thing(TOOL, 1, 1.0, 0.0)),
        Placed::Laid(_)
    ));
    assert_eq!(pile.lying.len(), 4);
}

/// A full cube refuses a tool and a heap of a new material, and a heap of a material already in
/// the cube joins it wherever it lies there.
#[test]
fn a_full_cube_refuses_or_merges() {
    let mut pile = Pile::default();
    for n in 0..CELL_MAX {
        let spaced = thing(HEAP, n as u8, 1.0, n as f64 * 0.1 * MERGE);
        laid(pile.lay(spaced));
    }
    assert_eq!(pile.lay(thing(TOOL, 0, 1.0, 1.0)), Placed::Refused);
    assert_eq!(pile.lay(thing(HEAP, 200, 1.0, 1.0)), Placed::Refused);
    let far_in_cube = thing(HEAP, 0, 2.0, 30.0);
    let Placed::Joined(heap) = pile.lay(far_in_cube) else {
        panic!("not joined");
    };
    assert_eq!((heap.material[0], heap.mass), (0, 3.0));
    assert_eq!(pile.lying.len(), CELL_MAX);
}

/// A tool is taken whole whatever mass is asked; a heap gives the part asked and keeps the rest,
/// then goes when the rest is taken; a thing taken is gone for the next asker.
#[test]
fn a_take_gets_a_tool_whole_or_a_part_of_a_heap() {
    let mut pile = Pile::default();
    let tool = laid(pile.lay(thing(TOOL, 1, 2.0, 0.0)));
    let heap = laid(pile.lay(thing(HEAP, 1, 3.0, 10.0)));
    let took = pile.take(tool.id, 0.1).expect("the tool");
    assert_eq!((took.got.clone(), took.left), (tool.clone(), None));
    assert!(pile.take(tool.id, 1.0).is_none());
    let part = pile.take(heap.id, 1.0).expect("a part");
    assert_eq!(part.got.mass, 1.0);
    assert_eq!(part.left.map(|left| left.mass), Some(2.0));
    let rest = pile.take(heap.id, 5.0).expect("the rest");
    assert_eq!((rest.got.mass, rest.left), (2.0, None));
    assert!(pile.lying.is_empty());
    assert!(pile.take(heap.id, f32::NAN).is_none());
}

/// Ids start at 1 and grow, and a pile made again from its things gives the next one past the
/// highest.
#[test]
fn ids_grow_and_survive_a_new_pile() {
    let mut pile = Pile::default();
    let a = laid(pile.lay(thing(TOOL, 1, 1.0, 0.0)));
    let b = laid(pile.lay(thing(TOOL, 1, 1.0, 5.0)));
    assert_eq!((a.id, b.id), (1, 2));
    let mut again = Pile::new(pile.lying.clone());
    let c = laid(again.lay(thing(TOOL, 1, 1.0, 9.0)));
    assert_eq!(c.id, 3);
    again.forget(1);
    again.told(Lying { mass: 4.0, ..b });
    assert_eq!(again.lying.len(), 2);
    assert_eq!(again.get(2).map(|l| l.mass), Some(4.0));
}
