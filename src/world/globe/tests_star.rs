//! Tests of the light the starts of a new game stand in: a temperate planet that turns is fair at every mild
//! latitude, so its starts are the ones it always had, and a planet locked to its star starts on its day side.

use bevy::math::DVec3;

use super::Globe;
use crate::universe::{LIT, PlanetSpec, Universe, sun};

/// Returns the home planet of the world of seed 7, which is locked to its star.
fn locked() -> PlanetSpec {
    let universe = Universe::new(7);
    universe.planet(universe.home())
}

/// On a globe built without a planet, which turns like the Earth under its temperate record, no place at the
/// latitudes of the starts lacks light or warmth: the fair rule adds nothing to any score there, and the
/// starts keep the order they had before it.
#[test]
fn a_temperate_turning_planet_is_fair_at_every_mild_latitude() {
    let fair = Globe::new(1, 6.371e6, 9.81).fair();
    for (degrees, k) in [20.0_f64, 27.0, 34.0, 41.0, 48.0].into_iter().zip(0..) {
        for step in 0..24 {
            let (lat, east) = (degrees.to_radians(), f64::from(step * 15 + k).to_radians());
            let north = DVec3::new(lat.cos() * east.cos(), lat.sin(), lat.cos() * east.sin());
            assert_eq!(fair.lack(north), 0.0, "{degrees} north");
            assert_eq!(
                fair.lack(north * DVec3::new(1.0, -1.0, 1.0)),
                0.0,
                "{degrees} south"
            );
        }
    }
}

/// On the locked home of seed 7 the night side lacks light, more the farther from the star, and the
/// point under the star lacks none. Every start of a new game there faces the star, [`LIT`] or higher.
#[test]
fn the_starts_of_a_locked_planet_face_its_star() {
    let planet = locked();
    assert!(planet.locked(), "the home of seed 7 no longer locks");
    let globe = Globe::of(&planet);
    let (fair, star) = (globe.fair(), sun(&planet, 0.0));
    let side = DVec3::Y.cross(star);
    assert!(fair.lack(side) > 0.0 && fair.lack(-star) > fair.lack(side));
    let starts = globe.landfalls();
    assert!(!starts.is_empty(), "no start on the home of seed 7");
    let lit = LIT.to_radians().sin() - 0.01;
    for dir in &starts {
        assert!(star.dot(*dir) >= lit, "a start at {}", star.dot(*dir));
    }
}
