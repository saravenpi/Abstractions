//! Tests of shared edges: neighbour nodes agree bit for bit on the vertices they share.

use std::collections::HashSet;
use std::sync::Arc;

use super::tests_globe::{globe, view};
use super::*;
use crate::world::Globe;
use crate::world::globe::{ChunkId, Face};

/// Returns the span of node `id`: through the chunk path at the leaf level, the node path above.
fn span(globe: &Arc<Globe>, id: ChunkId, n: usize) -> Span {
    let terrain = view(globe, id.face);
    if id.level == globe.leaf_level() {
        Span::leaf(&terrain, leaf_coord(id), n)
    } else {
        Span::node(&terrain, id, n)
    }
}

/// Returns the bits of the planet position of every vertex on the border of node `id`.
fn border(globe: &Arc<Globe>, id: ChunkId, n: usize) -> HashSet<[u64; 3]> {
    let span = span(globe, id, n);
    let on_border = |i: usize, j: usize| i == 0 || j == 0 || i == n || j == n;
    let all = (0..=n).flat_map(|i| (0..=n).map(move |j| (i, j)));
    all.filter(|(i, j)| on_border(*i, *j))
        .map(|(i, j)| {
            span.point(i, j, span.height(i, j))
                .to_array()
                .map(f64::to_bits)
        })
        .collect()
}

/// Returns the bits of the vertices of node `id` along its edge towards step `(dx, dy)`.
fn edge(globe: &Arc<Globe>, id: ChunkId, n: usize, step: (i32, i32)) -> Vec<[u64; 3]> {
    let span = span(globe, id, n);
    let pick = |k: usize| match step {
        (1, _) => (n, k),
        (-1, _) => (0, k),
        (_, 1) => (k, n),
        _ => (k, 0),
    };
    (0..=n)
        .map(pick)
        .map(|(i, j)| {
            span.point(i, j, span.height(i, j))
                .to_array()
                .map(f64::to_bits)
        })
        .collect()
}

/// Asserts that every vertex of `id` along `step` is bit-equal to a border vertex of the
/// neighbour on that side.
fn assert_edge_shared(globe: &Arc<Globe>, id: ChunkId, step: (i32, i32)) {
    let n = GRID_POINTS - 1;
    let other = id
        .neighbor(step.0, step.1)
        .expect("a side neighbour always exists");
    let theirs = border(globe, other, n);
    let ours = edge(globe, id, n, step);
    assert_eq!(ours.len(), n + 1);
    assert!(
        ours.iter().all(|p| theirs.contains(p)),
        "{id:?} and {other:?} disagree on their shared edge"
    );
}

/// Neighbour leaves inside a face share bit-equal edge vertices.
#[test]
fn neighbour_leaves_share_bit_equal_edges_within_a_face() {
    let globe = globe();
    let level = globe.leaf_level();
    for face in Face::ALL {
        let id = ChunkId {
            face,
            level,
            x: 40,
            y: 77,
        };
        for step in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            assert_eq!(id.neighbor(step.0, step.1).map(|o| o.face), Some(face));
            assert_edge_shared(&globe, id, step);
        }
    }
}

/// Neighbours share bit-equal edge vertices across every edge of the cube, from both sides, at
/// the leaf level and at coarse levels, also for the nodes at the cube corners.
#[test]
fn neighbours_share_bit_equal_edges_across_face_edges() {
    let globe = globe();
    let mut crossed = HashSet::new();
    for (face, level) in Face::ALL
        .into_iter()
        .flat_map(|f| [0, 2, globe.leaf_level()].map(|l| (f, l)))
    {
        let last = (1u32 << level) - 1;
        for along in [0, last / 3, last] {
            let sides = [
                ((1, 0), last, along),
                ((-1, 0), 0, along),
                ((0, 1), along, last),
                ((0, -1), along, 0),
            ];
            for (step, x, y) in sides {
                let id = ChunkId { face, level, x, y };
                let other = id
                    .neighbor(step.0, step.1)
                    .expect("a side neighbour always exists");
                assert_ne!(other.face, face);
                crossed.insert((face, other.face));
                assert_edge_shared(&globe, id, step);
            }
        }
    }
    assert_eq!(crossed.len(), 24);
}
