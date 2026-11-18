//! Tests of the selection along a path: the ground ahead of a moving focus is split before the
//! focus is over it, the most useful node first.

use bevy::math::DVec3;

use super::*;

/// Returns the path of a focus over a 6 km planet with seven levels and detail out to 370 m,
/// heading `lead` metres along +X from the top of the planet.
fn path(lead: f64) -> Path {
    let focus = DVec3::Y * 6000.0;
    let view = View {
        focus,
        radius: 6000.0,
        deepest: 7,
        reach: 370.0,
        budget: 4000,
        split: SPLIT,
    };
    Path {
        view,
        ahead: (focus + DVec3::X * lead).normalize() * 6000.0,
        reach: view.reach,
    }
}

/// Applies selections along `path` until nothing changes.
fn settle(tree: &mut Tree, path: &Path) {
    for _ in 0..10_000 {
        let changes = select_along(tree, path, &|_| true, 64);
        if changes == Changes::default() {
            return;
        }
        tree.apply(&changes);
    }
    panic!("the selection never settles");
}

/// Returns true when the leaf of `tree` over the point `metres` along +X from the top of the
/// planet is of the deepest level.
fn detailed(tree: &Tree, metres: f64) -> bool {
    let dir = (DVec3::Y * 6000.0 + DVec3::X * metres).normalize();
    let leaf = leaf_at(dir, 7);
    !tree.is_split(leaf) && parent(leaf).is_some_and(|above| tree.is_split(above))
}

/// A focus that stands still selects exactly what its view selects, node for node.
#[test]
fn standing_path_selects_like_the_view() {
    let still = Path::still(path(0.0).view);
    let (mut along, mut plain) = (Tree::default(), Tree::default());
    for _ in 0..200 {
        let (a, b) = (
            select_along(&along, &still, &|_| true, MAX_SPLITS),
            select(&plain, &still.view, &|_| true),
        );
        along.apply(&a);
        plain.apply(&b);
    }
    assert_eq!(along, plain);
    assert!(along.count() > 100);
    let node = leaf_at(DVec3::new(0.3, 0.9, 0.1).normalize(), 5);
    assert_eq!(still.near(node), still.view.distance(node));
}

/// The ground is split down to the leaves along the whole stretch to the point ahead and
/// around it, and no farther: what is far to the side or behind stays coarse.
#[test]
fn path_splits_the_ground_ahead() {
    let (mut moving, mut still) = (Tree::default(), Tree::default());
    settle(&mut moving, &path(1000.0));
    settle(&mut still, &path(0.0));
    assert!([0.0, 300.0, 700.0, 1000.0, 1250.0].map(|m| detailed(&moving, m)) == [true; 5]);
    assert!(!detailed(&moving, 1800.0) && !detailed(&moving, -800.0));
    assert!(detailed(&still, 250.0) && !detailed(&still, 700.0));
    for leaf in still.leaves() {
        let covered = moving.is_split(leaf) || moving.leaves().contains(&leaf);
        assert!(
            covered,
            "{leaf:?} is coarser around the path than around the focus"
        );
    }
    assert!(moving.count() > still.count());
}

/// Nodes are built in the order of their use: on the path before beside it, the one that
/// covers more of the screen before a smaller one as far away, and what lies behind last.
#[test]
fn cost_favours_what_is_ahead_and_large() {
    let path = path(1000.0);
    let at = |x: f64, z: f64, level: u8| {
        leaf_at(
            (DVec3::Y * 6000.0 + DVec3::new(x, 0.0, z)).normalize(),
            level,
        )
    };
    assert!(path.cost(at(600.0, 0.0, 7)) < path.cost(at(0.0, 600.0, 7)));
    assert!(path.cost(at(0.0, 600.0, 7)) < path.cost(at(-600.0, 0.0, 7)));
    assert!(path.cost(at(0.0, 900.0, 5)) < path.cost(at(0.0, 900.0, 7)));
    let still = Path::still(path.view);
    let (left, right) = (at(-600.0, 0.0, 7), at(600.0, 0.0, 7));
    assert!((still.cost(left) - still.cost(right)).abs() < 0.2);
}

/// A focus too fast to see the detail splits new ground less deep, and merges nothing that is
/// split already: its limit takes nothing away.
#[test]
fn narrow_reach_splits_less_and_merges_nothing() {
    let wide = path(1000.0);
    let narrow = Path {
        reach: 100.0,
        ..wide
    };
    let (mut fresh, mut kept) = (Tree::default(), Tree::default());
    settle(&mut fresh, &narrow);
    assert!(detailed(&fresh, 500.0) && !detailed(&fresh, 1450.0));
    settle(&mut kept, &wide);
    let before = kept.clone();
    settle(&mut kept, &narrow);
    assert_eq!(kept, before);
    assert!(fresh.count() < kept.count());
}
