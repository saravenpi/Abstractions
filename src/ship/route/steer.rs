//! Steering the route a frame: telling the sky what it aims at, holding the course, and dropping a route that
//! goes nowhere.

use bevy::math::DVec3;

use super::ascent::{climbs, departure};
use super::{beacon, hold};
use crate::computer::Target;
use crate::ship::body::Ship;
use crate::ship::flight::Controls;
use crate::ship::helm::Helm;
use crate::ship::pilot::{Autopilot, Goal, Phase, Settle};
use crate::ship::sky::Skies;
use crate::ship::tune::ORBIT_TOLERANCE;
use crate::ship::warp::{Tier, Warp};
use crate::ship::wells::Well;
use crate::space::Frame;
use crate::universe::{BodyId, PlanetSpec};

/// Ends the route at its destination: the drive that runs is asked to stop.
pub(super) fn arrive(autopilot: &mut Autopilot, helm: &mut Helm, ship: &Ship) {
    helm.warp = ship.warp != Warp::Off;
    *autopilot = Autopilot::ARRIVED;
}

/// Tells the sky what the course aims at: the beacon of a jump that is blind to the stars on the way, and
/// the body whose lock shrinks to its arrival shell for the course that is flown to it.
pub(super) fn mark(skies: &mut Skies, goal: &Goal, (tier, pos, to): (Tier, DVec3, DVec3)) {
    skies.beacon = tier.blind().then(|| beacon(pos, to, goal.lock));
    if let Target::Body(body) = goal.target {
        skies.arrive = Some((body, goal.lock));
    }
}

/// Stops the drive that ran on the course before when `settle` asks for it, lets go of every
/// control so that the ship holds its heading, and returns what the autopilot is doing.
pub(super) fn drop_first(helm: &mut Helm, settle: Settle) -> Phase {
    helm.controls = Controls::default();
    helm.warp |= settle == Settle::Stop;
    Phase::Dropping
}

/// Takes the next step of the route: holds the course or drops the drive that ran on the one before,
/// and drops a route that spools without coming nearer, see [`lose`].
pub(super) fn steer(
    autopilot: &mut Autopilot,
    (ship, helm): (&mut Ship, &mut Helm),
    skies: &Skies,
    (time, to, lock): (f64, DVec3, f64),
) {
    let off = ship.warp == Warp::Off;
    let settle = autopilot.settle(!off, time);
    autopilot.phase = match settle {
        Settle::Clear => hold(ship, helm, &skies.wells, (to, lock)),
        _ => drop_first(helm, settle),
    };
    if off && autopilot.phase == Phase::Spooling && autopilot.stalled(to.length()) {
        lose(autopilot, helm);
    }
}

/// Drops a route that spooled the drive again and again without coming nearer, so that it does
/// not loop: the drive is not asked for, the pilot has the controls, and the route reads as lost.
fn lose(autopilot: &mut Autopilot, helm: &mut Helm) {
    autopilot.cancel();
    autopilot.lost = true;
    helm.warp = false;
    helm.controls = Controls::default();
}

/// Returns the planet whose departure orbit a ship in the frame of a star is under, or nothing: such a
/// ship is in the lock sphere of the planet, or between it and the drop-out margin, where the drive
/// refuses or drops itself, and a route to anywhere but that planet climbs to the departure orbit
/// first, as it does from the air. A ship in the orbit itself, within two orbit tolerances of it, is
/// not under it.
fn lingers(ship: &Ship, skies: &Skies, target: Target) -> Option<BodyId> {
    if !matches!(ship.frame, Frame::System(_)) {
        return None;
    }
    let planets = skies.spec()?.planets.iter().zip(skies.wells.get(1..)?);
    let under = |(planet, well): &(&PlanetSpec, &Well)| {
        let floor = planet.radius + departure(planet) - 2.0 * ORBIT_TOLERANCE;
        Target::Body(planet.id) != target && well.pos.distance(ship.pos) < floor
    };
    planets.filter(under).map(|(planet, _)| planet.id).next()
}

/// Returns whether the route climbs this frame: it is in the air of a planet, in a climb already, or
/// under the departure orbit of a planet it does not go to, in which case it sets the climb off.
pub(super) fn climbing(
    autopilot: &mut Autopilot,
    ship: &Ship,
    skies: &Skies,
    target: Target,
) -> bool {
    if !climbs(ship) {
        autopilot.leaving = lingers(ship, skies, target).or(autopilot.leaving);
    }
    autopilot.phase == Phase::Climbing || climbs(ship) || autopilot.leaving.is_some()
}
