//! Holding the pilot in the seat until the ground beside the hatch is solid, and stopping the motors when the
//! pilot leaves.

use super::body::Ship;
use super::warp::Warp;

/// Seconds the pilot is held at the most: past it the pilot steps out on the height of the ground, which
/// the body is put back on when it sinks.
pub const HOLD: f32 = 10.0;

/// Returns the seconds the pilot has been held after a frame of `dt` seconds that began after `held`, or
/// nothing once the pilot may step out: the ground is `solid`, or [`HOLD`] seconds have passed.
pub fn hold(held: f32, solid: bool, dt: f32) -> Option<f32> {
    (!solid && held < HOLD).then_some(held + dt)
}

/// Steps the hold of a pilot who `pressed` the board key this frame, over ground that is `solid` or not,
/// after a frame of `dt` seconds. Returns true on the frame the pilot may step out.
pub fn release(held: &mut Option<f32>, pressed: bool, solid: bool, dt: f32) -> bool {
    let waited = if pressed { Some(0.0) } else { *held };
    *held = waited.and_then(|seconds| hold(seconds, solid, dt));
    waited.is_some() && held.is_none()
}

/// Stops the motors of a ship its pilot leaves: the throttle to idle and a spooling warp drive
/// off. The boost and the vertical thrusters are held keys the helm lets go of on foot.
pub fn idle(ship: &mut Ship) {
    ship.throttle = 0.0;
    if matches!(ship.warp, Warp::Spooling { .. }) {
        ship.warp = Warp::Off;
    }
}
