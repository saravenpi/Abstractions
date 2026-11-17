//! The flight systems: look around the ship, then step it at a fixed rate.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::ShipEvent;
use super::body::Ship;
use super::flight::{Report, STEP, Scene, rotate, step};
use super::helm::Helm;
use super::pilot::Autopilot;
use super::sky::Skies;
use super::transit::{catch_up, change_galaxy, enter_system, leave_system, reframe};
use super::warp::Warp;
use crate::space::{Clock, Frame};
use crate::universe::Universe;
use crate::world::Planet;

/// Longest frame simulated, in seconds, to survive hitches.
const MAX_FRAME: f64 = 0.1;

/// Seconds of universe time that pass unseen before the ship is carried along with its planet.
const GAP: f64 = 0.5;

/// Touchdown speed in metres per second under which no event is sent.
const QUIET: f64 = 0.5;

/// The universe and its clocks.
#[derive(SystemParam)]
pub struct Cosmos<'w> {
    /// The universe the ship flies in.
    pub universe: Res<'w, Universe>,
    /// Universe time.
    pub clock: Res<'w, Clock>,
    /// The planet the world streams.
    pub planet: Option<Res<'w, Planet>>,
}

/// Bookkeeping of the fixed step.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Pace {
    /// Seconds of frame time not yet simulated, always less than one step.
    pub carry: f64,
    /// What the last step did.
    pub report: Report,
    /// Universe time when the ship was last looked at.
    seen: Option<f64>,
}

/// Moves a ship between the stars into the frame of a star whose reach it is in: one of the
/// stars nearby, or on a long jump only the star of the destination.
fn arrive(ship: &mut Ship, universe: &Universe, autopilot: &Autopilot, skies: &mut Skies) {
    let Frame::Galaxy(galaxy) = ship.frame else {
        return;
    };
    if !autopilot.jump() {
        let stars = skies.stars(universe, galaxy, ship.pos);
        return enter_system(ship, stars);
    }
    let goal = autopilot.star().filter(|id| id.galaxy == galaxy);
    enter_system(ship, goal.and_then(|id| universe.star(id)).as_ref());
}

/// Switches the frame of the ship where it has to and looks up what surrounds it.
pub fn survey(
    cosmos: Cosmos,
    autopilot: Res<Autopilot>,
    mut skies: ResMut<Skies>,
    mut pace: ResMut<Pace>,
    mut ship: Single<&mut Ship>,
) {
    let (universe, time) = (&*cosmos.universe, cosmos.clock.seconds);
    let gap = pace.seen.replace(time).map_or(0.0, |seen| time - seen);
    match ship.frame {
        Frame::Surface(body) => reframe(&mut ship, skies.system(universe, body.star), time),
        Frame::System(star) => {
            let system = skies.system(universe, star);
            if gap > GAP {
                catch_up(&mut ship, system, time, gap);
            }
            reframe(&mut ship, system, time);
            leave_system(&mut ship, &system.star);
        }
        Frame::Galaxy(galaxy) => {
            change_galaxy(&mut ship, universe, galaxy);
            arrive(&mut ship, universe, &autopilot, &mut skies);
        }
    }
    skies.refresh(universe, cosmos.planet.as_deref(), &ship, time);
}

/// The resources the fixed step reads and writes.
#[derive(SystemParam)]
pub struct Flight<'w> {
    /// Frame time.
    time: Res<'w, Time>,
    /// What the pilot asks for.
    helm: Res<'w, Helm>,
    /// What surrounds the ship.
    skies: Res<'w, Skies>,
    /// Bookkeeping of the fixed step.
    pace: ResMut<'w, Pace>,
}

/// Steps the ship at a fixed rate through the frame time. In warp only its attitude moves.
pub fn simulate(
    mut flight: Flight,
    mut ship: Single<&mut Ship>,
    mut events: MessageWriter<ShipEvent>,
) {
    let dt = flight.time.delta_secs_f64().min(MAX_FRAME);
    if matches!(ship.warp, Warp::Active { .. }) {
        rotate(&mut ship, flight.helm.controls.turn, DVec3::ZERO, dt);
        *flight.pace = Pace {
            seen: flight.pace.seen,
            ..Pace::default()
        };
        return;
    }
    let skies = &*flight.skies;
    let ground = |pos: DVec3| skies.height(pos);
    let (mut elapsed, mut impact) = (0.0, 0.0f64);
    flight.pace.carry += dt;
    while flight.pace.carry >= STEP {
        let scene = Scene {
            planet: skies.planet(),
            ground: &ground,
            wells: &skies.wells,
            elapsed,
        };
        flight.pace.report = step(&mut ship, &flight.helm.controls, &scene, STEP);
        impact = impact.max(flight.pace.report.impact);
        elapsed += STEP;
        flight.pace.carry -= STEP;
    }
    if impact > QUIET {
        events.write(ShipEvent::Touchdown {
            speed: impact,
            damage: ship.damage,
        });
    }
}
