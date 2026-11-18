//! Tests of the lane: what a standing and a flying focus want built, and in which order.

use bevy::prelude::*;

use super::lane::Lane;
use crate::world::mesh::ring_distance;

/// A lane ten chunks long along +x with the Medium radius.
const FLIGHT: Lane = Lane {
    from: IVec2::ZERO,
    lead: Vec2::new(10.0, 0.0),
    radius: 4,
};

/// Returns where `cell` comes in the build order of `lane`.
fn place(lane: &Lane, cell: IVec2) -> Option<usize> {
    lane.cells().iter().position(|c| *c == cell)
}

/// A focus that stands still wants the plain disc around it, nearest first, with the rings and
/// the keep range the streamer always had.
#[test]
fn standing_lane_is_the_disc() {
    let center = IVec2::new(2, -3);
    for radius in [2, 4, 8] {
        let lane = Lane::new(center, Vec2::ZERO, radius);
        let cells = lane.cells();
        let far = |c: &IVec2| (*c - center).length_squared();
        let disc = (-radius..=radius)
            .flat_map(|x| (-radius..=radius).map(move |y| IVec2::new(x, y)))
            .filter(|c| c.length_squared() <= radius * radius);
        assert_eq!(cells.len(), disc.count());
        assert_eq!(cells[0], center);
        assert!(cells.iter().all(|c| far(c) <= radius * radius));
        assert!(cells.windows(2).all(|w| far(&w[0]) <= far(&w[1])));
        assert!(
            cells
                .iter()
                .all(|c| lane.rank(*c) == ring_distance(*c, center))
        );
        let edge = center + IVec2::new(radius + 1, 0);
        assert!(lane.holds(edge, 1) && !lane.holds(edge, 0));
        assert!(!lane.holds(edge + IVec2::Y, 1));
    }
}

/// A flying focus wants the ground along its path first: just ahead, then farther ahead, then
/// to the sides, and what lies behind last.
#[test]
fn lane_builds_what_is_ahead_first() {
    let cost = |x: i32, y: i32| FLIGHT.cost(IVec2::new(x, y));
    assert_eq!(FLIGHT.cells()[0], IVec2::ZERO);
    assert!(cost(1, 0) < cost(4, 0) && cost(4, 0) < cost(0, 3));
    assert!(cost(2, 1) < cost(-2, 0));
    assert_eq!(cost(0, 3), 3.0);
    assert_eq!(cost(-2, 0), 4.0);
    let order = |x: i32, y: i32| place(&FLIGHT, IVec2::new(x, y));
    assert!(order(5, 0) < order(0, 4) && order(0, 4).is_some());
    assert!(order(14, 0).is_some() && order(10, 4).is_some());
    assert!(order(15, 0).is_none() && order(5, 5).is_none());
    let still = Lane::new(IVec2::ZERO, Vec2::ZERO, 4);
    assert_eq!(still.cost(IVec2::new(-3, 0)), still.cost(IVec2::new(3, 0)));
}

/// The part of the disc a fast focus is about to leave is not started, the chunks that touch
/// the focus always are, and a slow focus still fills its whole disc.
#[test]
fn lane_leaves_its_wake_unbuilt() {
    let wants = |x: i32, y: i32| FLIGHT.wants(IVec2::new(x, y));
    assert!(!wants(-3, 0) && !wants(-4, 0) && !wants(-2, -2));
    assert!(wants(-1, 0) && wants(-1, -1) && wants(-2, 0));
    assert!(wants(0, -4) && wants(0, 4));
    assert!(FLIGHT.holds(IVec2::new(-4, 0), 0) && FLIGHT.holds(IVec2::new(-5, 0), 1));
    let slow = Lane {
        lead: Vec2::new(2.0, 0.0),
        ..FLIGHT
    };
    let disc = Lane::new(IVec2::ZERO, Vec2::ZERO, 4).cells();
    assert!(disc.iter().all(|cell| slow.wants(*cell)));
}

/// Rings are counted from the line of flight, also past the point ahead, so a chunk is built
/// at the level it will have when the focus flies by.
#[test]
fn lane_ranks_from_the_line_of_flight() {
    let rank = |x: i32, y: i32| FLIGHT.rank(IVec2::new(x, y));
    assert_eq!([rank(5, 0), rank(5, 2), rank(5, -3)], [0, 2, 3]);
    assert_eq!([rank(13, 0), rank(13, 2)], [0, 2]);
    assert_eq!([rank(-3, 0), rank(-2, 2)], [3, 2]);
    let skew = Lane {
        lead: Vec2::new(6.0, 6.0),
        ..FLIGHT
    };
    assert_eq!(skew.rank(IVec2::new(3, 3)), 0);
    assert_eq!(skew.rank(IVec2::new(4, 2)), 1);
}

/// A focus that barely moves ranks its chunks as one that stands still, within a ring: the
/// line of flight only runs on as far as the lead, so the levels do not flip as it starts off.
#[test]
fn slow_lane_ranks_like_the_disc() {
    let still = Lane::new(IVec2::ZERO, Vec2::ZERO, 8);
    for lead in [0.1, 0.3, 0.6] {
        let slow = Lane::new(IVec2::ZERO, Vec2::new(lead, lead * 0.5), 8);
        for cell in still.cells() {
            let change = (slow.rank(cell) - still.rank(cell)).abs();
            assert!(change <= 1, "{cell} at lead {lead}");
        }
    }
    assert_eq!(
        Lane::new(IVec2::ZERO, Vec2::new(0.1, 0.0), 8).lead,
        Vec2::ZERO
    );
    assert_eq!(Lane::new(IVec2::ZERO, Vec2::new(2.6, 0.0), 8).lead.x, 2.5);
}
