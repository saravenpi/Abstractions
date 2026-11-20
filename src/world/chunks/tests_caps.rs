//! Tests of the quadtree caps on deep planets: they scale with the depth of the tree, and an
//! Earth-sized planet settles within them in the time a world may take.

use super::reach::{MOST_LEVELS, Reach, STEPS};
use super::tests_reach::{EARTH, settle};
use crate::world::Globe;
use crate::world::globe::quadtree::{MAX_SPLITS, SHALLOW, splits};

/// The node cap follows the depth of the tree: lower on a shallow planet, never above the cap
/// of a twenty level tree, which is what every planet had before, and a deeper tree gets more
/// splits per selection and no fewer than a shallow one.
#[test]
fn the_caps_scale_with_the_depth_of_the_tree() {
    for (_, rings) in STEPS {
        let reach = Reach::of(Some(rings));
        let caps: Vec<usize> = (0..=MOST_LEVELS)
            .map(|depth| reach.node_cap(depth))
            .collect();
        assert!(caps.windows(2).all(|pair| pair[0] < pair[1]), "{caps:?}");
        assert_eq!(reach.node_cap(255), caps[usize::from(MOST_LEVELS)]);
        let (home, earth) = (reach.node_cap(7), reach.node_cap(17));
        assert!(home < earth && earth < caps[usize::from(MOST_LEVELS)]);
    }
    let steps: Vec<usize> = (0..=MOST_LEVELS).map(splits).collect();
    assert!(steps.windows(2).all(|pair| pair[0] <= pair[1]), "{steps:?}");
    assert_eq!(steps[usize::from(SHALLOW)], MAX_SPLITS);
    assert_eq!(steps[7], MAX_SPLITS);
    assert!(steps[17] > 2 * MAX_SPLITS, "{}", steps[17]);
}

/// An Earth-sized planet is 17 levels deep, settles at Medium within five minutes, and keeps
/// within the node cap of its depth.
#[test]
fn an_earth_sized_planet_settles_within_five_minutes() {
    let depth = Globe::new(7, EARTH, 9.81).deepest_level();
    assert_eq!(depth, 17);
    let load = settle(EARTH, 4);
    assert!(load.seconds < 300.0, "settled in {:.0} s", load.seconds);
    assert!(
        load.nodes <= Reach::of(Some(4)).node_cap(depth) + 4,
        "{}",
        load.nodes
    );
}
