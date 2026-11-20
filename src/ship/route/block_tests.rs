//! Tests of what keeps the drive from a course: the first failing check, in order, and the
//! state the route shows for each phase of the autopilot.

use bevy::math::{DQuat, DVec3};

use super::block::{Reading, aligned, blocker, status};
use crate::computer::{Block, RouteStatus};
use crate::ship::Ship;
use crate::ship::pilot::Phase;
use crate::ship::wells::Well;
use crate::space::Frame;
use crate::universe::{BodyId, StarId};

/// Returns a body of `radius` metres at the origin with its lock `radii` radii out.
fn well(radius: f64, radii: f64) -> Well {
    Well {
        pos: DVec3::ZERO,
        vel: DVec3::ZERO,
        mu: 1.0,
        radius,
        lock: radius * radii,
        hill: f64::INFINITY,
    }
}

#[test]
fn the_first_failing_check_is_the_reason() {
    let star = StarId::default();
    let mut ship = Ship::at_rest(Frame::System(star), DVec3::X * 1400.0, DQuat::IDENTITY);
    let wells = [well(1000.0, 3.0)];
    let inside = Block::Inside {
        star: false,
        lock: 30,
        tenths: 14,
    };
    assert_eq!(blocker(&ship, &wells), Some(inside));
    ship.pos = DVec3::X * 4000.0;
    assert_eq!(blocker(&ship, &wells), None);
    let near_star = blocker(&ship, &[well(1000.0, 5.0)]);
    assert!(matches!(
        near_star,
        Some(Block::Inside {
            star: true,
            lock: 50,
            tenths: 40
        })
    ));
    ship.frame = Frame::Surface(BodyId::default());
    assert_eq!(blocker(&ship, &wells), Some(Block::Surface));
    ship.landed = true;
    assert_eq!(blocker(&ship, &wells), Some(Block::Landed));
}

#[test]
fn the_route_says_why_and_how_far() {
    let read = |phase: Phase, block: Option<Block>, course: bool| {
        status(Reading {
            phase,
            block,
            aligned: 40,
            course,
        })
    };
    let landed = Some(Block::Landed);
    assert_eq!(
        read(Phase::Idle, landed, true),
        RouteStatus::Blocked(Block::Landed)
    );
    assert_eq!(read(Phase::Idle, landed, false), RouteStatus::Idle);
    assert_eq!(
        read(Phase::Waiting, landed, true),
        RouteStatus::Blocked(Block::Landed)
    );
    assert_eq!(read(Phase::Aligning, None, true), RouteStatus::Aligning(40));
    assert_eq!(read(Phase::Cruising, None, true), RouteStatus::InWarp);
    assert_eq!(aligned(DVec3::X, DVec3::X * 5.0), 100);
    assert_eq!(aligned(DVec3::X, DVec3::Y), 50);
    assert_eq!(aligned(DVec3::X, DVec3::NEG_X), 0);
}
