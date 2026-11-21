//! Stopping the motors when the pilot leaves.

use super::body::Ship;
use super::warp::Warp;

/// Stops the motors of a ship its pilot leaves: the throttle to idle and a spooling warp drive
/// off. The boost and the vertical thrusters are held keys the helm lets go of on foot.
pub fn idle(ship: &mut Ship) {
    ship.throttle = 0.0;
    if matches!(ship.warp, Warp::Spooling { .. }) {
        ship.warp = Warp::Off;
    }
}
