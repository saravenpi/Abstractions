//! The climb to orbit: a route flown from the air of a planet takes the ship up to a circular
//! orbit above the air, with the speed hold of the assist, and lets go of the assist there. A
//! route to that planet ends in the orbit. A route to anything else climbs past the lock sphere
//! of the planet and goes on with the warp drive.

use bevy::math::DVec3;

use super::handoff::reached;
use super::tune::LOCK_MARGIN;
use crate::computer::Target;
use crate::ship::body::Ship;
use crate::ship::cruise::top_speed_at;
use crate::ship::flight::Controls;
use crate::ship::frames::spin_rate;
use crate::ship::helm::Helm;
use crate::ship::orbit::apsides;
use crate::ship::pilot::{Autopilot, Phase};
use crate::ship::sky::Skies;
use crate::ship::steer::aim;
use crate::ship::tune::{CLIMB_TIME, ORBIT_ALTITUDE, ORBIT_RAMP, ORBIT_TOLERANCE};
use crate::ship::wells::{PLANET_LOCK, Well};
use crate::space::Frame;
use crate::universe::{BodyId, PlanetSpec};

/// Where the ship is against the planet it climbs from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::ship) struct Fix {
    /// Metres from the centre of the planet, in the axes of the frame of the ship.
    pub pos: DVec3,
    /// Velocity in metres per second against the centre of the planet, in axes that do not turn.
    pub vel: DVec3,
    /// Velocity in metres per second that the frame of the ship itself has at `pos`: the ground
    /// speed of the turning planet in its surface frame, nothing in the frame of the star.
    pub frame: DVec3,
}

/// What the climb asks of the ship: the velocity the speed hold flies, as a unit direction for
/// the nose and a share of the top speed for the lever, and whether the orbit is reached.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::ship) struct Order {
    /// Unit direction the nose points to.
    pub aim: DVec3,
    /// Main engine lever from 0 to 1.
    pub throttle: f64,
    /// Whether the ship is in the orbit.
    pub done: bool,
}

/// Returns the height in metres above sea level of the orbit around `planet`: the height of
/// [`ORBIT_ALTITUDE`], or a quarter higher than the air when that is thicker.
pub(in crate::ship) fn altitude(planet: &PlanetSpec) -> f64 {
    ORBIT_ALTITUDE.max(planet.atmosphere_height * 1.25)
}

/// Returns the height in metres above sea level of the orbit a route to another place leaves
/// `planet` from: the orbit of [`altitude`], or higher when the lock sphere of the planet is
/// wider, by [`LOCK_MARGIN`].
pub(in crate::ship) fn departure(planet: &PlanetSpec) -> f64 {
    altitude(planet).max(planet.radius * (PLANET_LOCK - 1.0) * LOCK_MARGIN)
}

/// Returns true when a route is a climb: the ship is in the air of a planet and off the ground.
pub(in crate::ship) fn climbs(ship: &Ship) -> bool {
    matches!(ship.frame, Frame::Surface(_)) && !ship.landed
}

/// Returns what the climb to the height `goal` in metres asks of a ship at `fix` above `planet`.
///
/// The speed hold flies the velocity the climb wants: up at a speed that closes the gap to the
/// orbit in [`CLIMB_TIME`] seconds, and along the equator at the speed of a circular orbit,
/// which it reaches at the share [`ORBIT_RAMP`] of the height. The sideways speed grows with the
/// square of the height, so the ship climbs through the air and turns over above it.
pub(in crate::ship) fn order(fix: &Fix, planet: &PlanetSpec, goal: f64) -> Order {
    let radius = fix.pos.length();
    let up = fix.pos / radius;
    let height = radius - planet.radius;
    let circular = (planet.mu() / radius).sqrt();
    let east = DVec3::Y
        .cross(up)
        .try_normalize()
        .unwrap_or_else(|| up.any_orthonormal_vector());
    let share = (height / (goal * ORBIT_RAMP)).clamp(0.0, 1.0);
    let wish = up * ((goal - height) / CLIMB_TIME) + east * (circular * share * share);
    let ask = wish - fix.frame;
    let top = top_speed_at(height, circular);
    let target = planet.radius + goal;
    let done = apsides(fix.pos, fix.vel, planet.mu()).is_some_and(|(far, near)| {
        (far - target).abs().max((near - target).abs()) < ORBIT_TOLERANCE
    });
    Order {
        aim: ask.try_normalize().unwrap_or(up),
        throttle: (ask.length() / top).min(1.0),
        done,
    }
}

/// Returns where the ship is against `planet`, or nothing when it is not in a frame that has
/// it. In the frame of the star the planet is the well of the system.
fn fix(ship: &Ship, skies: &Skies, planet: &PlanetSpec) -> Option<Fix> {
    match ship.frame {
        Frame::Surface(_) => {
            let frame = DVec3::Y.cross(ship.pos) * spin_rate(planet);
            Some(Fix {
                pos: ship.pos,
                vel: ship.vel + frame,
                frame,
            })
        }
        Frame::System(_) => {
            let well = well_of(skies, planet.id)?;
            Some(Fix {
                pos: ship.pos - well.pos,
                vel: ship.vel - well.vel,
                frame: DVec3::ZERO,
            })
        }
        Frame::Galaxy(_) => None,
    }
}

/// Returns the well of the planet `body` in the system the ship was last in.
fn well_of(skies: &Skies, body: BodyId) -> Option<&Well> {
    let index = skies
        .spec()?
        .planets
        .iter()
        .position(|planet| planet.id == body)?;
    skies.wells.get(index + 1)
}

/// Flies the climb to the orbit around the planet `body` for one frame: turns the nose onto the
/// course, sets the lever and keeps the assist on. The orbit is the one of [`altitude`] when
/// `stay` is true, and the one of [`departure`] otherwise. Returns whether the orbit is reached,
/// in which case the assist is let go and the ship is left to orbit, or nothing when the climb
/// cannot go on.
fn steer_up(
    ship: &mut Ship,
    helm: &mut Helm,
    skies: &Skies,
    (body, stay): (BodyId, bool),
) -> Option<bool> {
    let system = skies.spec()?;
    let planet = system.planets.iter().find(|planet| planet.id == body)?;
    if ship.landed {
        return None;
    }
    let goal = [departure(planet), altitude(planet)][usize::from(stay)];
    let order = order(&fix(ship, skies, planet)?, planet, goal);
    if order.done {
        helm.controls = Controls::default();
        (ship.throttle, ship.assist) = (0.0, false);
        return Some(true);
    }
    helm.controls = Controls {
        turn: aim(ship.rot, order.aim),
        ..Controls::default()
    };
    (ship.throttle, ship.assist) = (order.throttle, true);
    Some(false)
}

/// Flies the climb out of the air of the planet the ship is over for one frame. It ends the
/// route when the orbit is reached around the planet `target`, hands over to the warp drive when
/// the route goes elsewhere, and ends the route when the climb cannot go on.
pub(in crate::ship) fn climb(
    autopilot: &mut Autopilot,
    ship: &mut Ship,
    helm: &mut Helm,
    skies: &Skies,
    target: Target,
) {
    if let Frame::Surface(body) = ship.frame {
        autopilot.leaving = Some(body);
    }
    let Some(body) = autopilot.leaving else {
        return autopilot.cancel();
    };
    let stay = target == Target::Body(body);
    match steer_up(ship, helm, skies, (body, stay)) {
        None => autopilot.cancel(),
        Some(false) => autopilot.phase = Phase::Climbing,
        Some(true) => reached(autopilot, stay),
    }
}
