//! A spool that started from the departure orbit of a planet completes and the drive stays engaged: the
//! orbit lies outside the drop-out margin of the lock sphere, so a course with a little pull toward the
//! planet is not cut at the first step.

use bevy::math::{DQuat, DVec3};

use super::ascent::departure;
use crate::ship::body::Ship;
use crate::ship::warp::{CRUISE_CAP, Warp, advance};
use crate::ship::wells::Well;
use crate::space::Frame;
use crate::universe::Universe;

/// Seconds of one frame of the test.
const DT: f64 = 0.1;

/// Frames the drive is watched for after it engages: a second.
const WATCHED: usize = 10;

/// A ship that spools at the departure orbit of a planet, its nose a tenth of a radian below the
/// horizon, engages once and is still in warp a second later.
#[test]
fn a_spool_from_the_departure_orbit_completes_once() {
    let universe = Universe::new(7);
    let planet = universe.planet(universe.home());
    let well = Well::planet(&planet, 0.0);
    let up = DVec3::X;
    let pos = well.pos + up * (planet.radius + departure(&planet));
    let heading = (DVec3::Y - up * 0.1).normalize();
    let rot = DQuat::from_rotation_arc(DVec3::NEG_Z, heading);
    let mut ship = Ship::at_rest(Frame::System(universe.home().star), pos, rot);
    ship.warp = Warp::Spooling { left: DT / 2.0 };
    let wells = [well];
    advance(&mut ship, &wells, CRUISE_CAP, DT);
    assert!(matches!(ship.warp, Warp::Active { .. }), "{:?}", ship.warp);
    for frame in 0..WATCHED {
        advance(&mut ship, &wells, CRUISE_CAP, DT);
        assert!(
            matches!(ship.warp, Warp::Active { .. }),
            "dropped out at frame {frame}: {:?}",
            ship.warp
        );
    }
}
