//! Steering the route a frame: telling the sky what it aims at, holding the course, and dropping a route that
//! goes nowhere.

use bevy::math::DVec3;

use super::{beacon, hold};
use crate::computer::Target;
use crate::ship::body::Ship;
use crate::ship::flight::Controls;
use crate::ship::helm::Helm;
use crate::ship::pilot::{Autopilot, Goal, Phase, Settle};
use crate::ship::sky::Skies;
use crate::ship::warp::{Tier, Warp};

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
