//! A course across a galaxy keeps its heading and never loops: the drive that flies a heading exactly on
//! the course is not dropped for missing it, and a route that spools again and again without coming
//! nearer is dropped.

use bevy::math::{DQuat, DVec3, IVec3};

use super::align::missed;
use crate::computer::Target;
use crate::ship::body::Ship;
use crate::ship::pilot::{Autopilot, SPOOLS};
use crate::ship::warp::Warp;
use crate::space::Frame;
use crate::universe::{GalaxyId, StarId};

/// Metres between two galaxies, a few cells wide.
const ACROSS: f64 = 3.0e22;

/// Lock sphere of a star of the Sun's size, in metres: five solar radii.
const LOCK: f64 = 3.5e9;

/// A ship in warp whose nose points along `dir`.
fn flying(dir: DVec3) -> Ship {
    let rot = DQuat::from_rotation_arc(DVec3::NEG_Z, dir);
    let mut ship = Ship::at_rest(Frame::Galaxy(GalaxyId(IVec3::ZERO)), DVec3::ZERO, rot);
    ship.warp = Warp::Active { speed: 1.0e19 };
    ship
}

/// A ship whose heading is the course to the far galaxy to the last bit of a 64-bit direction is not
/// dropped, from any side, where an angle taken from a cosine reads 1.5e-8 radians for a hair of
/// error, which is 4.5e14 m across 3e22 m against a lock of 3.5e9 m.
#[test]
fn a_heading_on_the_course_is_not_missed_across_a_galaxy() {
    for dir in [
        DVec3::new(1.0, 0.2, -0.3),
        DVec3::new(-0.7, 0.1, 0.9),
        DVec3::new(0.01, 1.0, 0.02),
        DVec3::new(0.3, -0.5, -1.0),
    ] {
        let dir = dir.normalize();
        let ship = flying(dir);
        assert!(!missed(&ship, None, (dir * ACROSS, LOCK)), "{dir}");
    }
}

/// A heading off the course by a billionth of a radian is missed: across 3e22 m it is 3e13 m aside.
#[test]
fn a_heading_that_would_miss_the_star_is_missed() {
    let dir = DVec3::new(1.0, 0.2, -0.3).normalize();
    let ship = flying(DQuat::from_rotation_y(1.0e-9) * dir);
    assert!(missed(&ship, None, (dir * ACROSS, LOCK)));
}

/// Spools that come no nearer are counted, and the route is dropped after [`SPOOLS`] of them; a spool that
/// starts nearer begins the count again.
#[test]
fn a_route_that_spools_without_coming_nearer_is_dropped() {
    let mut pilot = Autopilot::default();
    pilot.engage(Target::Star(StarId::default()));
    assert!(!pilot.stalled(1.0e21));
    for _ in 0..SPOOLS {
        assert!(!pilot.stalled(1.0e21));
    }
    assert!(pilot.stalled(1.0e21));
    assert!(!pilot.stalled(0.5e21));
    for _ in 0..SPOOLS {
        assert!(!pilot.stalled(0.5e21));
    }
}
