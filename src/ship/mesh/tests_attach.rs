//! Attachment tests: the fixed parts of the ship make one piece, no part floats. A part is
//! attached to another when they share a corner, or when a corner of one is inside the other.
//! Every part that is closed is closed, the hull is open only at the mouth of the cockpit,
//! and no triangle is degenerate.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use super::parts::{Part, parts};
use super::shell::MOUTH;
use super::tests_graph::{Key, find, holds, key, triangles, volumes};

/// The fixed parts of the ship are one piece: connected by shared corners, or by corners sunk
/// into the part that carries them.
#[test]
fn nothing_floats() {
    let all = parts();
    let points: Vec<HashMap<Key, Vec3>> = all
        .iter()
        .map(|part| triangles(part).into_iter().map(|p| (key(p), p)).collect())
        .collect();
    let mut parent: Vec<usize> = (0..all.len()).collect();
    let mut shared: HashMap<Key, usize> = HashMap::new();
    for (i, set) in points.iter().enumerate() {
        for corner in set.keys() {
            let other = *shared.entry(*corner).or_insert(i);
            let (a, b) = (find(&mut parent, i), find(&mut parent, other));
            parent[a] = b;
        }
    }
    for (i, part) in all.iter().enumerate() {
        let carried = volumes(part);
        for (j, set) in points.iter().enumerate().filter(|(j, _)| *j != i) {
            if holds(&carried, set) {
                let (a, b) = (find(&mut parent, i), find(&mut parent, j));
                parent[a] = b;
            }
        }
    }
    let roots: HashSet<usize> = (0..all.len()).map(|i| find(&mut parent, i)).collect();
    let first = find(&mut parent, 0);
    let lost: Vec<_> = (0..all.len())
        .filter(|i| find(&mut parent, *i) != first)
        .map(|i| (all[i].group, all[i].paint))
        .collect();
    assert_eq!(roots.len(), 1, "these parts float: {lost:?}");
}

/// Returns the edges that belong to one triangle only, in the triangles of `parts`.
fn open_edges(parts: &[&Part]) -> Vec<(Vec3, Vec3)> {
    let mut count: HashMap<(Key, Key), (u32, Vec3, Vec3)> = HashMap::new();
    for tri in parts.iter().flat_map(|part| part.solid.tris.iter()) {
        for i in 0..3 {
            let (a, b) = (tri[i], tri[(i + 1) % 3]);
            let id = if key(a) < key(b) {
                (key(a), key(b))
            } else {
                (key(b), key(a))
            };
            count.entry(id).or_insert((0, a, b)).0 += 1;
        }
    }
    let single = count.into_values().filter(|(n, _, _)| *n == 1);
    single.map(|(_, a, b)| (a, b)).collect()
}

/// Every group of parts is a closed shape, with no missing face to see through, except the hull,
/// which is open only over the cockpit, and the glass, which is open under the roof.
#[test]
fn every_part_is_closed_but_the_mouth() {
    let all = parts();
    let mut groups: HashMap<&str, Vec<&Part>> = HashMap::new();
    for part in &all {
        groups.entry(part.group).or_default().push(part);
    }
    for (group, members) in &groups {
        let open = open_edges(members);
        match *group {
            "canopy" => assert!(!open.is_empty()),
            "body" | "nose" => continue,
            _ => assert!(
                open.is_empty(),
                "{group} has {} open edges: {open:?}",
                open.len()
            ),
        }
    }
    let hull = [&groups["body"][..], &groups["nose"][..]].concat();
    for (a, b) in open_edges(&hull) {
        for p in [a, b] {
            assert!(
                p.z > MOUTH.0 - 1e-3 && p.z < MOUTH.1 + 1e-3 && p.y > 0.3,
                "{p} is open"
            );
        }
    }
}

/// No triangle of any part is degenerate.
#[test]
fn no_triangle_is_degenerate() {
    for part in parts() {
        for tri in &part.solid.tris {
            let area = (tri[1] - tri[0]).cross(tri[2] - tri[0]).length();
            assert!(area > 1e-6, "{}: {tri:?}", part.group);
        }
    }
}
