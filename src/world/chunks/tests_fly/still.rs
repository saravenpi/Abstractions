//! Standing still and walking: the settled world is the one the streamer gave before it looked
//! ahead, node for node and chunk for chunk.

use bevy::prelude::*;

use super::super::Chunks;
use super::{Route, frame, rest, stage};
use crate::world::globe::{ChunkId, Face};

/// The nodes and the chunks of the home-sized planet settled at its spawn at Medium, as the
/// streamer gave them before it looked ahead: how many, and their digest.
const STOOD: [(usize, u64); 2] = [
    (418, 11_615_703_762_813_295_876),
    (49, 3_824_252_845_275_085_246),
];

/// The same after a walk of a hundred and twenty metres.
const WALKED: [(usize, u64); 2] = [
    (458, 5_613_989_752_754_919_172),
    (62, 7_719_096_710_386_607_901),
];

/// Walking speed of the robot in metres per second.
const WALK: f64 = 5.0;

/// Returns how many `ids` there are and a digest of them that does not depend on their order.
fn digest(ids: impl Iterator<Item = ChunkId>) -> (usize, u64) {
    let face = |id: &ChunkId| Face::ALL.iter().position(|f| *f == id.face).unwrap_or(0) as u64;
    let mut keys: Vec<[u64; 4]> = ids
        .map(|id| {
            [
                face(&id),
                u64::from(id.level),
                u64::from(id.x),
                u64::from(id.y),
            ]
        })
        .collect();
    keys.sort_unstable();
    let mix = |hash: u64, part: &u64| (hash ^ part).wrapping_mul(0x0100_0000_01b3);
    let hash = keys.iter().flatten().fold(0xcbf2_9ce4_8422_2325, mix);
    (keys.len(), hash)
}

/// Returns the digests of the coarse nodes and of the detail chunks of `app`.
fn state(app: &App) -> [(usize, u64); 2] {
    let chunks = app.world().resource::<Chunks>();
    assert!(chunks.loaded.values().all(|l| l.shown));
    [
        digest(chunks.nodes.live.keys().copied()),
        digest(chunks.loaded.keys().copied()),
    ]
}

/// Walks `route` ten metres at a time, with a rest after each, for a hundred and twenty metres:
/// far enough to change chunk and to split and merge nodes.
fn walk(app: &mut App, route: &mut Route) {
    route.speed = WALK;
    for _ in 0..12 {
        (0..120).for_each(|_| frame(app, route));
        rest(app, route);
    }
}

/// A focus that stands still, and one that walked, settle on the very nodes and chunks the
/// streamer built before it looked ahead.
#[test]
fn standing_and_walking_settle_as_before() {
    let (mut app, mut route) = stage(6000.0, 4);
    let stood = state(&app);
    walk(&mut app, &mut route);
    assert_eq!([stood, state(&app)], [STOOD, WALKED]);
}
