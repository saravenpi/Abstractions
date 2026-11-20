//! Tests of what the plants of a canopy shed: only real leaves and real flowers.

use super::*;
use crate::life::Relation;
use crate::life::flora::fixture::{garden, of};

/// Returns the look of producer `i` of `life` at `stage`, grown.
fn grown(life: &crate::life::Life, i: usize, stage: Stage) -> Look {
    Look {
        dials: of(life, i, stage),
        variant: 1,
    }
}

/// A deciduous tree stands bare in winter and sheds nothing; in leaf it sheds leaves of its own colour,
/// and three times as many in autumn.
#[test]
fn a_bare_tree_sheds_no_leaves() {
    let life = garden(11, &[0], Vec::new());
    let bare = grown(&life, 0, Stage::Dormant);
    assert!(bare.dials.form.deciduous && bare.dials.paint.leaves.is_none());
    assert_eq!(shed(&bare, Stage::Dormant, true), None);
    let green = grown(&life, 0, Stage::Green);
    let leafy = shed(&green, Stage::Green, true).expect("a tree in leaf sheds");
    assert_eq!(leafy.leaf, green.dials.paint.leaves);
    assert_eq!(leafy.rate, 1.0);
    let turning = grown(&life, 0, Stage::Fruit);
    assert_eq!(
        shed(&turning, Stage::Fruit, true).map(|s| s.rate),
        Some(AUTUMN)
    );
}

/// Flowers come only from a pollinated plant in flower, and ground plants and crystals drop no leaves.
#[test]
fn flowers_only_where_pollinated() {
    let alone = garden(12, &[0, 2, 7], Vec::new());
    let tree = grown(&alone, 0, Stage::Bloom);
    assert_eq!(shed(&tree, Stage::Bloom, true).and_then(|s| s.flower), None);
    assert_eq!(
        shed(&grown(&alone, 1, Stage::Green), Stage::Green, false),
        None
    );
    assert_eq!(
        shed(&grown(&alone, 2, Stage::Green), Stage::Green, true),
        None
    );
    let visited = vec![Relation::Pollinates {
        animal: 9,
        plant: 1,
    }];
    let meadow = garden(12, &[0, 2], visited);
    let mat = grown(&meadow, 1, Stage::Bloom);
    let flowering = shed(&mat, Stage::Bloom, false).expect("a mat in flower sheds pollen");
    assert_eq!(
        (flowering.leaf, flowering.flower),
        (None, mat.dials.paint.flowers)
    );
}
