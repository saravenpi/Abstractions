//! Tests that nothing hangs in the air above a room: every piece up there is held by the walls.

use bevy::prelude::*;

use super::probe::{Bx, Shot};
use super::tags::Tag;
use super::tests_doors::every_house;
use super::village::House;

/// How close two pieces must be, in meters, to hold each other.
const TOUCH: f32 = 0.025;

/// Returns the part of `b` within `TOUCH` of `a`, if the two pieces touch.
fn contact(a: &Bx, b: &Bx) -> Option<Bx> {
    let lo = (a.0 - Vec3::splat(TOUCH)).max(b.0);
    let hi = (a.1 + Vec3::splat(TOUCH)).min(b.1);
    lo.cmple(hi).all().then_some((lo, hi))
}

/// Returns the boxes and round pieces of `shot`, and for each how many pieces lie between it and a wall:
/// 0 for a wall, `usize::MAX` for a piece that no chain of touching pieces links to a wall.
fn held(shot: &Shot) -> (Vec<Bx>, Vec<usize>) {
    let closed = shot.closed().into_iter();
    let round = shot
        .ranges(Tag::Round)
        .into_iter()
        .map(|(s, e)| (s, shot.bounds(s, e)));
    let nodes: Vec<(usize, Bx)> = closed.chain(round).collect();
    let mut depth: Vec<usize> = nodes
        .iter()
        .map(|(at, _)| {
            if shot.is(Tag::Wall, *at) {
                0
            } else {
                usize::MAX
            }
        })
        .collect();
    for level in 0..nodes.len() {
        let reached = |i: usize| depth[i] == level;
        let front: Vec<usize> = (0..nodes.len()).filter(|i| reached(*i)).collect();
        let free = (0..nodes.len()).filter(|i| depth[*i] == usize::MAX);
        let next: Vec<usize> = free
            .filter(|i| {
                front
                    .iter()
                    .any(|f| contact(&nodes[*f].1, &nodes[*i].1).is_some())
            })
            .collect();
        next.iter().for_each(|i| depth[*i] = level + 1);
    }
    (nodes.into_iter().map(|n| n.1).collect(), depth)
}

/// Returns true when the piece `i` rests on pieces no farther from the walls than itself whose contacts
/// surround its middle, so that it does not stick out unsupported.
fn carried(nodes: &[Bx], depth: &[usize], i: usize) -> bool {
    let others = (0..nodes.len()).filter(|j| *j != i && depth[*j] <= depth[i]);
    let spots: Vec<Bx> = others
        .filter_map(|j| contact(&nodes[j], &nodes[i]))
        .collect();
    let lo = spots.iter().fold(Vec3::MAX, |m, s| m.min(s.0)) - Vec3::splat(TOUCH);
    let hi = spots.iter().fold(Vec3::MIN, |m, s| m.max(s.1)) + Vec3::splat(TOUCH);
    let mid = (nodes[i].0 + nodes[i].1) * 0.5;
    mid.xz().cmpge(lo.xz()).all() && mid.xz().cmple(hi.xz()).all()
}

/// Asserts that every piece above head height over the room of `house` is held by the walls, and that the
/// large ones rest on contacts that surround their middle.
fn check_room(shot: &Shot, house: House, what: &str) {
    let inner = house.shell().inner();
    let (nodes, depth) = held(shot);
    for (i, (lo, hi)) in nodes.iter().enumerate() {
        let mid = (*lo + *hi) * 0.5;
        if lo.y < 1.7 || mid.x.abs() >= inner.x || mid.z.abs() >= inner.y {
            continue;
        }
        assert!(
            depth[i] != usize::MAX,
            "{what}: the piece {lo} to {hi} hangs in the air"
        );
        let large = (hi.x - lo.x) * (hi.z - lo.z) >= 1.0;
        assert!(
            !large || carried(&nodes, &depth, i),
            "{what}: the piece {lo} to {hi} sticks out unsupported"
        );
    }
}

/// In every house kind, style and lot variation, every piece above head height over the room, such as a
/// hanging lantern, a ceiling or a turret, is linked to the walls by pieces that touch, and none sticks out.
#[test]
fn nothing_hangs_over_a_room() {
    every_house(12, check_room);
}
