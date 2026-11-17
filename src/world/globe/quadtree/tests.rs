//! Tests of the quadtree selection: exact cover, split budget, hysteresis and the ready gate.

use std::collections::HashSet;

use bevy::math::DVec3;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::*;

/// Returns a view from `focus` on a 6 km planet with seven levels and detail out to 370 m.
fn view(focus: DVec3) -> View {
    View {
        focus,
        radius: 6000.0,
        deepest: 7,
        reach: 370.0,
        budget: MAX_SPLITS,
        split: SPLIT,
    }
}

/// Applies selections until nothing changes and returns how many it took.
fn settle(tree: &mut Tree, view: &View) -> usize {
    for round in 0..10_000 {
        let changes = select(tree, view, &|_| true);
        if changes == Changes::default() {
            return round;
        }
        tree.apply(&changes);
    }
    panic!("the tree never settles");
}

/// Asserts that the leaves of `tree` cover the six faces with no hole and no overlap.
fn assert_exact_cover(tree: &Tree) {
    let leaves: HashSet<ChunkId> = tree.leaves().into_iter().collect();
    assert_eq!(leaves.len(), tree.leaves().len());
    let area: f64 = leaves
        .iter()
        .map(|id| 0.25f64.powi(i32::from(id.level)))
        .sum();
    assert_eq!(area, 6.0);
    for leaf in &leaves {
        let mut above = parent(*leaf);
        while let Some(id) = above {
            assert!(!leaves.contains(&id) && tree.is_split(id));
            above = parent(id);
        }
    }
}

/// Returns a random unit direction.
fn random_dir(rng: &mut ChaCha8Rng) -> DVec3 {
    loop {
        let v = DVec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        );
        if (0.01..1.0).contains(&v.length_squared()) {
            return v.normalize();
        }
    }
}

/// The leaves cover the sphere exactly once for 200 focus positions from the surface to 5 radii,
/// whether the tree is fresh or carried over from the previous focus.
#[test]
fn leaves_cover_the_sphere_exactly_once() {
    let mut rng = ChaCha8Rng::seed_from_u64(7);
    let mut carried = Tree::default();
    for k in 0..200 {
        let height = 6000.0 * (1.0 + 4.0 * (k as f64 / 199.0).powi(3));
        let v = view(random_dir(&mut rng) * height);
        settle(&mut carried, &v);
        assert_exact_cover(&carried);
        let mut fresh = Tree::default();
        settle(&mut fresh, &v);
        assert_exact_cover(&fresh);
        let deepest = fresh.leaves().iter().map(|id| id.level).max();
        assert!(deepest <= Some(v.deepest));
    }
}

/// On the surface the tree reaches the deepest level under the focus, which `leaf_at` names, and
/// stays a few hundred nodes.
#[test]
fn surface_focus_reaches_the_leaf_level() {
    let dir = DVec3::new(0.3, 0.8, -0.52).normalize();
    let v = view(dir * 6002.0);
    let mut tree = Tree::default();
    settle(&mut tree, &v);
    let under = leaf_at(dir, v.deepest);
    assert!(tree.leaves().contains(&under));
    assert_eq!(leaf_at(under.dir(0.5, 0.5), v.deepest), under);
    assert_eq!(parent(children(under)[3]), Some(under));
    assert!((100..700).contains(&tree.count()), "{} nodes", tree.count());
    let largest = View {
        focus: dir * 40_002.0,
        radius: 40_000.0,
        deepest: 10,
        reach: 9.0 * 61.4,
        budget: MAX_SPLITS,
        split: SPLIT,
    };
    let mut tree = Tree::default();
    settle(&mut tree, &largest);
    assert!(tree.count() < MAX_NODES, "{} nodes", tree.count());
}

/// One selection never asks for more splits than the budget, never more than four, and only
/// splits leaves that are ready.
#[test]
fn split_budget_is_respected() {
    for budget in [0, 1, 2, 4, 9] {
        let v = View {
            budget,
            ..view(DVec3::new(0.2, 0.9, 0.4).normalize() * 6000.0)
        };
        let mut tree = Tree::default();
        for _ in 0..40 {
            let changes = select(&tree, &v, &|_| true);
            assert!(changes.splits.len() <= budget.min(MAX_SPLITS));
            tree.apply(&changes);
        }
        assert_eq!(tree.count() > 6, budget > 0);
        assert!(select(&tree, &v, &|_| false).splits.is_empty());
    }
    let v = view(DVec3::Y * 6000.0);
    assert_eq!(
        select(&Tree::default(), &v, &|id| id.face == Face::PosY)
            .splits
            .len(),
        1
    );
}

/// A focus that moves 1 cm back and forth across a split boundary causes no split and no merge.
#[test]
fn oscillating_on_a_split_boundary_changes_nothing() {
    let along = |t: f64| {
        let start = DVec3::new(0.21, 0.93, 0.3).normalize();
        let side = start.cross(DVec3::X).normalize();
        (start * (t / 6000.0).cos() + side * (t / 6000.0).sin()) * 6002.0
    };
    for level in [3, 5, 6] {
        let node = leaf_at(along(0.0).normalize(), level);
        let wants = |t: f64| view(along(t)).splits(node);
        let (mut inside, mut outside) = (0.0, 6000.0);
        assert!(wants(inside) && !wants(outside));
        while outside - inside > 1e-6 {
            let mid = (inside + outside) * 0.5;
            if wants(mid) {
                inside = mid;
            } else {
                outside = mid;
            }
        }
        let mut tree = Tree::default();
        settle(&mut tree, &view(along(inside - 0.01)));
        assert!(tree.is_split(node));
        for step in 0..200 {
            let t = inside + [0.01, -0.01][step % 2];
            assert!(!wants(inside + 0.01) && wants(inside - 0.01));
            assert_eq!(
                select(&tree, &view(along(t)), &|_| true),
                Changes::default()
            );
        }
    }
}
