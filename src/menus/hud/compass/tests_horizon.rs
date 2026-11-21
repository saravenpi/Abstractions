//! Tests of the line of sight to a place on the planet below: the planet hides the far side
//! from orbit, and hides nothing near the eye.

use bevy::math::DVec3;

use super::horizon::behind;

/// Sea level radius of the planet of the tests, the Earth's, in metres.
const RADIUS: f64 = 6.371e6;

/// Returns the point `height` metres over the unit direction `dir` of the planet.
fn over(dir: DVec3, height: f64) -> DVec3 {
    dir.normalize() * (RADIUS + height)
}

/// Returns the sphere the planet hides things with for an eye at `eye` and a place at `place`:
/// sea level, or 100 m under the lower of the two.
fn sphere(eye: DVec3, place: DVec3) -> (DVec3, f64) {
    (
        DVec3::ZERO,
        RADIUS.min(eye.length() - 100.0).min(place.length() - 100.0),
    )
}

/// From 800 km up the base on the far side of the planet is behind it, the base under the ship
/// and one 1,000 km away are not.
#[test]
fn from_orbit_the_far_side_is_behind_the_planet() {
    let eye = over(DVec3::X, 800_000.0);
    let far = over(DVec3::NEG_X, 50.0);
    assert!(behind(eye, far, sphere(eye, far)));
    for dir in [DVec3::X, DVec3::new(1.0, 0.157, 0.0)] {
        let base = over(dir, 50.0);
        assert!(!behind(eye, base, sphere(eye, base)), "{dir}");
    }
}

/// On the ground nothing within 30 km is hidden by the curve of the planet, even with the base
/// on a sea-level plain: the ground it stands on does not count.
#[test]
fn on_the_ground_near_places_are_not_behind_the_planet() {
    let eye = over(DVec3::X, 2.0);
    let angle = 30_000.0 / RADIUS;
    let base = over(DVec3::new(angle.cos(), angle.sin(), 0.0), 3.0);
    assert!(!behind(eye, base, sphere(eye, base)));
}
