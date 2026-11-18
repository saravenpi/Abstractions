//! Flying with the player: whether the player flies, and where the companion flies beside it.

use bevy::prelude::*;

use super::brain::Brain;
use super::spot::{Probe, Stance, lift};
use super::spring::ramp;
use super::tune::{ENTRY, WING, WING_SPEED};
use crate::gear::FlightState;

/// Sink rate in metres per second above which a flight counts as a dive.
const DIVE_SINK: f32 = 25.0;

/// Share of the airspeed the sink rate must pass for a dive to count as steep: about 40 degrees.
const DIVE_SLOPE: f32 = 0.64;

/// How fast the side of the course takes over from the side of the body as the two line up:
/// the course alone decides once its right is a quarter of the way to the right of the body.
/// It is a ramp and not a sign, so a course that swings across the body, as when the boots
/// slide sideways, moves the spot round and never flips it to the other wing.
const SIDE_GAIN: f32 = 4.0;

/// Airspeed in metres per second above which a body in the air without its wings counts as flying.
const SWIFT: f32 = 18.0;

/// Returns whether the gear in `state` flies the body, and whether it dives steeply and fast.
/// A jump, or a hop on the boots, is not a flight: the wings are out or the fall is fast.
pub fn aloft(state: &FlightState) -> (bool, bool) {
    let flying = state.mode.winged() || (state.mode.airborne() && state.airspeed > SWIFT);
    let steep = state.velocity.y < -DIVE_SINK && state.velocity.y < -DIVE_SLOPE * state.airspeed;
    (flying, flying && steep)
}

impl Stance {
    /// Returns the spot it flies at beside a player flying at `velocity`: behind and to the
    /// left, a little above, so it stays out of the line of a camera over the right shoulder.
    /// A slow or hovering player is flown beside along the view instead, and the spot turns
    /// from the view to the course as the speed crosses [`WING_SPEED`], never in one step: the
    /// two spots are blended, not the two directions, which may be opposite.
    ///
    /// `right` is the right of the player's body, or zero when it is not known. In a loop the
    /// course passes the vertical and comes back the other way while the body keeps its
    /// heading. So the side of the course is taken on the side of the body, and the steeper
    /// the course the more the body alone tells it: the companion stays on the same wing all
    /// the way round instead of crossing over at the top.
    pub fn formation(&self, velocity: Vec3, right: Vec3) -> Vec3 {
        let level = self.level();
        let share = ramp(WING_SPEED.0, WING_SPEED.1, velocity.length());
        let course = velocity.normalize_or(level);
        self.wing(level, right)
            .lerp(self.wing(course, right), share)
    }

    /// Returns the spot behind, to the left of and above a player that goes `ahead`, whose
    /// body has its right at `right`, or zero when that is not known.
    fn wing(&self, ahead: Vec3, right: Vec3) -> Vec3 {
        let flat = ahead.cross(self.up);
        let across = match right.try_normalize() {
            Some(side) => {
                let seen = flat * (flat.dot(side) * SIDE_GAIN).clamp(-1.0, 1.0);
                let own = (1.0 - seen.length()).max(0.0);
                (side * own + seen).normalize_or(side)
            }
            None => flat.try_normalize().unwrap_or_else(|| self.right()),
        };
        self.eye - ahead * WING.1 - across * WING.0 + self.up * WING.2
    }
}

impl Brain {
    /// Returns where a companion truly left behind starts again: beside the camera and a
    /// little behind it, on the side of its place, so it is out of the picture and flies into
    /// it. Before the camera is known that is just behind the player.
    pub fn entry(&self) -> Vec3 {
        let Some(view) = self.scene.view.try_normalize() else {
            return self.stance.behind();
        };
        let right = view.cross(self.stance.up).normalize_or(self.stance.right());
        let side = (self.goal().0 - self.scene.camera).dot(right).signum();
        self.scene.camera + right * (side * ENTRY.0) - view * ENTRY.1
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
