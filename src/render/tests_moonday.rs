//! Day tests on a moon: the star goes round a moon's sky once per solar day of its planet's year
//! and its own turn, and asking for a time of day moves universe time forward to reach it.

use bevy::math::{DMat3, DQuat, DVec3};
use bevy::prelude::*;

use super::clock::{catch_up, solar_day};
use super::observer::Observer;
use crate::space::{Anchor, Frame};
use crate::universe::{PlanetClass, PlanetSpec, StarSpec, Universe};

/// Returns a moon of the gas giant of the home system of seed 3, and the star.
fn moon() -> (PlanetSpec, StarSpec) {
    let universe = Universe::new(3);
    let system = universe.system(universe.home().star);
    let giant = system
        .planets
        .iter()
        .find(|planet| planet.record.class == PlanetClass::GasGiant && planet.moons > 0);
    let giant = giant.expect("the home system has a giant with moons");
    let moon = universe.moons(giant.id).remove(0);
    (moon, system.star)
}

/// Returns a surface anchor standing at sea level where the moon-fixed direction `up` leaves it.
fn standing(moon: &PlanetSpec, up: DVec3) -> Anchor {
    let east = DVec3::Y.cross(up).normalize();
    let to_frame = DQuat::from_mat3(&DMat3::from_cols(east, up, east.cross(up)));
    Anchor {
        frame: Frame::Surface(moon.id),
        origin: up * moon.radius,
        rotation: to_frame.inverse(),
        flat: false,
    }
}

/// The solar day of a moon is finite, the phase goes round once in it, and a wanted phase is
/// reached by moving universe time forward, whichever way the star crosses the moon's sky.
#[test]
fn a_moon_has_days_and_reaches_a_wanted_time() {
    let (moon, star) = moon();
    let anchor = standing(&moon, DVec3::new(0.3, 0.5, -0.8).normalize());
    let phase = |time: f64| Observer::new(&anchor, Vec3::ZERO, &moon, &star, time).phase();
    let day = solar_day(&moon);
    assert!(day.is_finite() && day.abs() > 1.0, "{day}");
    let start = 5_000.0;
    let turned = (phase(start + day.abs()) - phase(start) + 0.5).rem_euclid(1.0) - 0.5;
    assert!(turned.abs() < 2e-3, "{turned}");
    for wanted in [0.0, 0.08, 0.25, 0.6, 0.99] {
        let later = start + catch_up(phase(start), wanted, &moon);
        let off = (phase(later) - wanted + 0.5).rem_euclid(1.0) - 0.5;
        assert!(
            later >= start && off.abs() < 1e-3,
            "{wanted}: {}",
            phase(later)
        );
    }
}
