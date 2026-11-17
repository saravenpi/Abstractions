//! Flying with the player: whether the player flies, and where the companion flies beside it.

use bevy::prelude::*;

use super::spot::{Probe, Stance, lift};
use super::tune::{WING, WING_SPEED};
use crate::gear::FlightState;

/// Sink rate in metres per second above which a flight counts as a dive.
const DIVE_SINK: f32 = 25.0;

/// Share of the airspeed the sink rate must pass for a dive to count as steep: about 40 degrees.
const DIVE_SLOPE: f32 = 0.64;

/// Airspeed in metres per second above which a body in the air without its wings counts as flying.
const SWIFT: f32 = 18.0;

/// Returns whether the gear in `state` flies the body, its bank, and whether it dives steeply
/// and fast. A jump, or a hop on the boots, is not a flight: the wings are out or the fall is fast.
pub fn aloft(state: &FlightState) -> (bool, f32, bool) {
    let flying = state.mode.winged() || (state.mode.airborne() && state.airspeed > SWIFT);
    let steep = state.velocity.y < -DIVE_SINK && state.velocity.y < -DIVE_SLOPE * state.airspeed;
    (
        flying,
        state.bank * f32::from(u8::from(flying)),
        flying && steep,
    )
}

impl Stance {
    /// Returns the spot it flies at beside a player flying at `velocity`: behind and to the
    /// left, a little above, so it stays out of the line of a camera over the right shoulder.
    /// A slow or hovering player is flown beside along the view instead.
    pub fn formation(&self, velocity: Vec3) -> Vec3 {
        let course = velocity
            .try_normalize()
            .filter(|_| velocity.length() > WING_SPEED);
        let ahead = course.unwrap_or_else(|| self.level());
        let across = ahead
            .cross(self.up)
            .try_normalize()
            .unwrap_or_else(|| self.right());
        self.eye - ahead * WING.1 - across * WING.0 + self.up * WING.2
    }
}

impl Probe<'_> {
    /// Returns `pos` raised until it is `clearance` above the ground and the water both here and
    /// where `velocity` carries it in `ahead` seconds, so a fast body climbs before a slope.
    pub fn clear_ahead(&self, pos: Vec3, velocity: Vec3, (ahead, clearance): (f32, f32)) -> Vec3 {
        let gap = self.gap(pos).min(self.gap(pos + velocity * ahead));
        lift(pos, self.up(pos), gap, clearance)
    }
}
