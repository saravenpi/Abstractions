//! What the route does once the climb reaches its orbit: a route to the planet ends there, a
//! route to anything else goes on with the warp drive.

use crate::ship::pilot::{Autopilot, Phase};

/// Ends the climb of `autopilot` in its orbit. A route to the planet that was climbed from, `stay`,
/// has arrived. A route to anything else waits for the drive, which the next frame of the route
/// turns and spools, and forgets the planet it left.
pub(in crate::ship) fn reached(autopilot: &mut Autopilot, stay: bool) {
    match stay {
        true => *autopilot = Autopilot::ARRIVED,
        false => (autopilot.leaving, autopilot.phase) = (None, Phase::Waiting),
    }
}
