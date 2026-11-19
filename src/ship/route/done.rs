//! Ending a course a moment after the ship arrives, so its line leaves the HUD and the map.

use bevy::prelude::*;

use crate::computer::{Destination, RouteStatus};

/// Seconds the arrival stays on the HUD before the course ends.
pub const SHOWN: f32 = 6.0;

/// Returns how many seconds the route has read as arrived after `delta` more seconds, and
/// whether the course ends now. A route that does not read as arrived starts again from zero.
pub fn tick(since: f32, delta: f32, arrived: bool) -> (f32, bool) {
    let since = [0.0, since + delta][usize::from(arrived)];
    (since, since >= SHOWN)
}

/// Clears the course once the route has read as arrived for [`SHOWN`] seconds.
pub fn finish(
    time: Res<Time>,
    status: Res<RouteStatus>,
    mut destination: ResMut<Destination>,
    mut since: Local<f32>,
) {
    let arrived = *status == RouteStatus::Arrived;
    let (held, done) = tick(*since, time.delta_secs(), arrived);
    *since = held;
    if done && destination.0.is_some() {
        destination.0 = None;
        *since = 0.0;
    }
}
