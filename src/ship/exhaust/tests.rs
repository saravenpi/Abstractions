//! Exhaust tests: the shape of a plume follows the thrust and the air, the engines spool over
//! their time, and the state of a ship comes from its real thrust.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::shape::{Shape, ease, shape, spool, warp};
use super::state::{demand, lift};
use super::tune::{HOVER_PRESSURE, SCROLL, SPOOL_DOWN, SPOOL_UP};
use crate::ship::body::Ship;
use crate::ship::flight::Report;
use crate::ship::tune::LIFT_ACCEL;
use crate::space::Frame;
use crate::universe::BodyId;

/// Thick air holds the gas in a narrow, bright jet with shock diamonds; vacuum lets it spread into
/// a wider, long, faint plume with none, whose envelope fades as it widens. Since 0.11.0 the
/// vacuum plume is 1.8 times as wide as in thick air, down from 2.8: the owner found it far too
/// wide.
#[test]
fn thin_air_widens_and_dims_the_plume() {
    let (thick, vacuum) = (shape(0.3, 1.0, 1.0), shape(0.3, 1.0, 0.0));
    assert!(vacuum.width > 1.5 * thick.width, "{vacuum:?} {thick:?}");
    assert!(vacuum.width < 2.0 * thick.width, "{vacuum:?} {thick:?}");
    assert!((vacuum.spread * thick.width - vacuum.width).abs() < 1.0e-6);
    assert_eq!(thick.spread, 1.0);
    assert!(vacuum.length > 2.0 * thick.length);
    assert!(vacuum.bright < 0.5 * thick.bright);
    assert!(thick.shock > 0.99 && vacuum.shock == 0.0);
}

/// More thrust gives a longer, wider, brighter plume, and none gives nothing.
#[test]
fn the_plume_follows_the_thrust() {
    let (half, full) = (shape(0.3, 0.5, 1.0), shape(0.3, 1.0, 1.0));
    assert!(full.length > half.length && full.width > half.width && full.bright > half.bright);
    assert_eq!(shape(0.3, 0.0, 1.0), Shape::default());
    assert_eq!(warp(0.3, 0.0), Shape::default());
}

/// The noise scrolls at the speed of the exhaust, capped where the eye still sees it flow.
#[test]
fn the_noise_scroll_is_capped() {
    for air in [0.0, 0.5, 1.0] {
        assert!(shape(0.3, 1.0, air).scroll <= SCROLL);
    }
    assert!(shape(0.3, 1.0, 1.0).scroll > 0.0);
}

/// The warp plume is longer and narrower than the longest thrust plume, whatever the air.
#[test]
fn the_warp_plume_is_a_long_narrow_streak() {
    let (streak, thrust) = (warp(0.3, 1.0), shape(0.3, 1.0, 0.0));
    assert!(streak.length > thrust.length && streak.width < thrust.width);
    assert!(warp(0.3, 0.5).length < streak.length);
}

/// An engine spools up to 63 percent of what is asked in its spool-up time and shuts down to 37
/// percent in its shut-down time, and cools more slowly than it spools.
#[test]
fn the_engines_spool_over_their_time() {
    let up = spool(0.0, 1.0, SPOOL_UP);
    let down = spool(1.0, 0.0, SPOOL_DOWN);
    assert!((up - 0.632).abs() < 0.01, "{up}");
    assert!((down - 0.368).abs() < 0.01, "{down}");
    let (mut warm, mut cool) = (0.0, 1.0);
    for _ in 0..60 {
        warm = ease(warm, 1.0, (0.8, 5.0), 1.0 / 60.0);
        cool = ease(cool, 0.0, (0.8, 5.0), 1.0 / 60.0);
    }
    assert!(warm > 0.7 && cool > 0.8, "{warm} {cool}");
}

/// Returns a ship at rest over the home planet.
fn flying() -> Ship {
    let frame = Frame::Surface(BodyId::default());
    Ship::at_rest(frame, DVec3::Y * 6.0e6, DQuat::IDENTITY)
}

/// The lift nozzles carry the push of the vertical thrusters, as a share of their full push, the
/// lever on the gear, and are shut at rest and in fast air.
#[test]
fn the_lift_nozzles_follow_the_vertical_push() {
    let ship = flying();
    let report = Report {
        proper: DVec3::Y * LIFT_ACCEL * 0.4,
        ..Report::default()
    };
    assert!((lift(&ship, &report, 0.0) - 0.4).abs() < 1.0e-5);
    let landed = Ship {
        landed: true,
        ..ship.clone()
    };
    assert_eq!(lift(&landed, &report, 1.0), 0.0);
    let fast = Report {
        pressure: HOVER_PRESSURE * 2.0,
        ..report
    };
    assert_eq!(lift(&ship, &fast, 1.0), 0.0);
    let turned = Ship {
        rot: DQuat::from_rotation_x(std::f64::consts::PI),
        ..ship
    };
    assert_eq!(lift(&turned, &report, 0.0), 0.0);
    let gear = Ship {
        touching: true,
        ..flying()
    };
    assert_eq!(lift(&gear, &report, 0.0), 0.0);
    assert!((lift(&gear, &report, 0.6) - 0.6).abs() < 1.0e-6);
}

/// The attitude jets follow the stick without the assist, and with it the gap between the turn
/// the stick asks for and the turn the ship has.
#[test]
fn the_attitude_jets_follow_the_stick_and_the_gap() {
    let stick = Vec3::new(1.0, 0.0, 0.0);
    assert_eq!(demand(stick, None), stick);
    let mut ship = flying();
    assert_eq!(demand(Vec3::ZERO, Some(&ship)), Vec3::ZERO);
    ship.spin = DVec3::new(0.0, 0.0, 0.6);
    assert_eq!(demand(Vec3::ZERO, Some(&ship)), Vec3::new(0.0, 0.0, -1.0));
    ship.spin = DVec3::new(0.7700000000000001, 0.0, 0.0);
    let held = demand(Vec3::X * 0.7, Some(&ship));
    assert!(held.x.abs() < 0.3, "{held}");
}
