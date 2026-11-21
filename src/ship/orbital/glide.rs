//! The glide from the cruise into the air toward the target, as plain math: where it begins,
//! the path it flies and the speed it flies it at.
//!
//! The path aims at a point [`END`] short of the target and above the ground, at most
//! [`GLIDE_ANGLE`] under the horizon, and flattens over the last [`FLARE`] metres. The speed is
//! the bound of the assist in air, [`heat_cap`]: 1.4 times the speed at which the nose starts
//! to glow at the height, a mild orange glow, the same bound the climb to orbit flies at. The
//! autopilot holds the boost, so in the low air the engines hold that speed against the drag.

use bevy::math::DVec3;

use super::tune::{DONE, END, FLARE, GLIDE_ANGLE, GLIDE_MOST};
use crate::ship::assist::NEUTRAL;
use crate::ship::cruise::{BOOST_SPEED, top_speed_at};
use crate::ship::drag::{density, heat_cap, terminal};
use crate::ship::flight::BOOST;
use crate::universe::PlanetSpec;

/// Least lever the glide sets: over the neutral band of the lever, where the speed hold would
/// read idle and stop the ship. High over a heavy planet the full lever asks for so much that the
/// glide speed is a few hundredths of it.
const LIVE: f64 = NEUTRAL * 1.2;

/// Returns the metres along the sea level before the target at which a cruise `height` metres
/// up hands over to the glide: where a path at [`GLIDE_ANGLE`] from that height reaches the aim.
pub fn handover(height: f64) -> f64 {
    END.1 + (height - END.0).max(0.0) / GLIDE_ANGLE.tan()
}

/// Returns the metres of track left at `height` metres above sea level, from the unit direction
/// `up` to the point where the glide toward the unit direction `goal` begins, on a planet of
/// sea level `radius` metres. Nothing is left once the ship is past it.
pub fn remaining(up: DVec3, goal: DVec3, radius: f64, height: f64) -> f64 {
    let arc = up.angle_between(goal) * radius;
    ((arc - handover(height)) * (radius + height) / radius).max(0.0)
}

/// Returns the speed in metres per second the glide asks for `height` metres above sea level:
/// the [`heat_cap`] there, a mild glow, and above the air the one at its top, which the ship is
/// about to enter, at most [`GLIDE_MOST`] over a body with no air.
pub fn speed(planet: &PlanetSpec, height: f64) -> f64 {
    let sample = height.min(planet.atmosphere_height - 1.0).max(0.0);
    heat_cap(density(planet, sample)).min(GLIDE_MOST)
}

/// Returns a rough time in seconds of the glide from `height` metres above sea level over
/// `planet`: its path at [`GLIDE_ANGLE`] at the speed it asks for half way down.
pub fn time(planet: &PlanetSpec, height: f64) -> f64 {
    let path = (height - END.0).max(0.0) / GLIDE_ANGLE.sin();
    path / speed(planet, 0.5 * height).max(1.0)
}

/// Returns the angle in radians under the horizon of the path of a ship `above` metres over the
/// ground, `arc` metres along the ground from the target: toward the aim at most
/// [`GLIDE_ANGLE`], flattened over the last [`FLARE`] metres down to its least share.
pub fn angle(above: f64, arc: f64) -> f64 {
    let rise = (above - END.0).max(0.0);
    let line = rise.atan2(arc - END.1).min(GLIDE_ANGLE);
    line * (rise / FLARE.0).clamp(FLARE.1, 1.0)
}

/// Where the gliding ship is against the planet, in the axes of the frame of the ship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fix {
    /// Metres from the centre of the planet.
    pub pos: DVec3,
    /// Velocity of the ground under the ship in the terms the speed hold flies: nothing in the
    /// surface frame, the spin of the planet in the frame of its star.
    pub frame: DVec3,
    /// Unit direction of the target from the centre of the planet.
    pub goal: DVec3,
    /// Height in metres above sea level of the ground under the ship.
    pub ground: f64,
}

/// What the glide asks of the ship for one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Order {
    /// Unit direction the nose points along: the velocity the speed hold flies.
    pub aim: DVec3,
    /// Main engine lever from 0 to 1.
    pub throttle: f64,
    /// Whether the glide is over: the ship is near the aim height.
    pub done: bool,
}

/// Returns what the glide asks of a ship at `fix` over `planet`: the nose along the path toward
/// the target at the glide speed against the ground, and the lever for that speed, under the
/// fastest the boosted engine holds against the drag of the air there, and never idle. The
/// caller holds the boost.
pub fn order(fix: &Fix, planet: &PlanetSpec) -> Order {
    let radius = fix.pos.length();
    let up = fix.pos / radius;
    let height = radius - planet.radius;
    let above = height - fix.ground;
    let arc = up.angle_between(fix.goal) * planet.radius;
    let toward = (fix.goal - up * fix.goal.dot(up))
        .try_normalize()
        .unwrap_or_else(|| up.any_orthonormal_vector());
    let tilt = angle(above, arc);
    let wish = (toward * tilt.cos() - up * tilt.sin()) * speed(planet, height);
    let ask = wish + fix.frame;
    let circular = (planet.mu() / radius).sqrt();
    let air = density(planet, height);
    let most = (top_speed_at(above.max(0.0), circular) * BOOST_SPEED).min(terminal(air, BOOST));
    Order {
        aim: ask.try_normalize().unwrap_or(toward),
        throttle: (ask.length() / most.max(1.0)).clamp(LIVE, 1.0),
        done: above <= END.0 + DONE,
    }
}
