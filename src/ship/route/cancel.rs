//! Stopping a route: the stop key of the computer drops the route and, on the next frame, the
//! drive that runs.

use bevy::prelude::*;

use crate::computer::Cancel;
use crate::ship::body::Ship;
use crate::ship::flight::Controls;
use crate::ship::helm::Helm;
use crate::ship::pilot::Autopilot;
use crate::ship::warp::Warp;

impl Autopilot {
    /// Drops the route and drops out of the warp it was flying, as the stop key does.
    pub fn stop(&mut self) {
        let flying = self.target.is_some();
        self.cancel();
        self.halt = flying;
    }
}

/// Stops the route when the board computer asks for it: the drive drops out and the ship is
/// handed back to the pilot.
pub fn on_cancel(_: On<Cancel>, mut autopilot: ResMut<Autopilot>) {
    autopilot.stop();
}

/// Run condition: the pilot stopped a route and the drive is yet to drop out.
pub fn halting(autopilot: Res<Autopilot>) -> bool {
    autopilot.halt
}

/// Ends a stop the pilot asked for: the drive that runs drops out, the pilot gets the controls
/// back and the engines idle.
pub fn halt(mut autopilot: ResMut<Autopilot>, mut helm: ResMut<Helm>, mut ship: Single<&mut Ship>) {
    autopilot.halt = false;
    helm.warp |= ship.warp != Warp::Off;
    (helm.controls, ship.throttle) = (Controls::default(), 0.0);
}
