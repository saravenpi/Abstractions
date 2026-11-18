//! The flight systems: look around the ship, then step it at a fixed rate.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::body::Ship;
use super::flight::{Controls, Report, STEP, Scene, rotate, step};
use super::gear::locked;
use super::helm::Helm;
use super::hull::meet;
use super::hull_cast::Solids;
use super::pilot::Autopilot;
use super::sky::Skies;
use super::transit::{catch_up, change_galaxy, enter_system, leave_system, reframe};
use super::warp::Warp;
use super::{Impact, ShipEvent};
use crate::home::Base;
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
    /// The home base, once it is found.
    pub base: Option<Res<'w, Base>>,
}

/// Bookkeeping of the fixed step.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Pace {
    /// Seconds of frame time not yet simulated, always less than one step.
    pub carry: f64,
    /// What the last step did.
    pub report: Report,
    /// Seconds of universe time the flight steps of this frame cover: the bodies around the
    /// ship were sampled that long before now.
    pub lag: f64,
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
///
/// Universe time is the time at the end of the frame, which everything is drawn at, and the
/// ship was last stepped up to the time of the frame before. So the frame changes and the
/// bodies are taken at that earlier moment, and the flight steps then carry the ship to now.
/// Taking them at the new time instead would move every planet a frame ahead of the ship, at
/// kilometres per second.
pub fn survey(
    cosmos: Cosmos,
    autopilot: Res<Autopilot>,
    mut skies: ResMut<Skies>,
    mut pace: ResMut<Pace>,
    mut ship: Single<&mut Ship>,
) {
    let (universe, time) = (&*cosmos.universe, cosmos.clock.seconds);
    let gap = pace.seen.replace(time).map_or(0.0, |seen| time - seen);
    pace.lag = gap.clamp(0.0, MAX_FRAME);
    let then = time - pace.lag;
    match ship.frame {
        Frame::Surface(body) => reframe(&mut ship, skies.system(universe, body.star), then),
        Frame::System(star) => {
            let system = skies.system(universe, star);
            if gap > GAP {
                catch_up(&mut ship, system, then, gap - pace.lag);
            }
            reframe(&mut ship, system, then);
            leave_system(&mut ship, &system.star);
        }
        Frame::Galaxy(galaxy) => {
            change_galaxy(&mut ship, universe, galaxy);
            arrive(&mut ship, universe, &autopilot, &mut skies);
        }
    }
    skies.base = cosmos.base.as_deref().copied();
    skies.refresh(universe, cosmos.planet.as_deref(), &ship, then);
}

/// The resources the fixed step reads and writes.
#[derive(SystemParam)]
pub struct Flight<'w, 's> {
    /// Frame time.
    time: Res<'w, Time>,
    /// What the pilot asks for.
    helm: Res<'w, Helm>,
    /// What surrounds the ship.
    skies: Res<'w, Skies>,
    /// Bookkeeping of the fixed step.
    pace: ResMut<'w, Pace>,
    /// The built solids near the player, in a game that runs the physics.
    solids: Option<Solids<'w, 's>>,
}

/// Advances `ship` by one fixed step in `scene`, then sweeps its hull along that step against
/// the built `solids` and books what it met on the report of the step.
pub fn stride(
    ship: &mut Ship,
    controls: &Controls,
    scene: &Scene,
    solids: Option<&Solids>,
) -> Report {
    let (frame, from, down) = (ship.frame, ship.pos, locked(ship));
    let cast =
        |at, rot, motion| solids.and_then(|solids| solids.cast(frame, (at, rot), motion, down));
    let mut report = step(ship, controls, scene, STEP);
    meet(ship, from, &cast, &mut report);
    report
}

impl Flight<'_, '_> {
    /// Steps `ship` at the fixed rate through `dt` seconds of frame time and returns the
    /// fastest touchdown of these steps in metres per second, zero without one, and the crash
    /// of these steps, if one was.
    fn advance(&mut self, ship: &mut Ship, dt: f64) -> (f64, Option<Impact>) {
        let (skies, solids) = (&*self.skies, self.solids.as_ref());
        let ground = |pos: DVec3| skies.height(pos);
        let (mut elapsed, mut impact, mut crash) = (0.0, 0.0f64, None);
        self.pace.carry += dt;
        while self.pace.carry >= STEP {
            let scene = Scene {
                planet: skies.planet(),
                ground: &ground,
                wells: &skies.wells,
                elapsed,
            };
            let report = stride(ship, &self.helm.controls, &scene, solids);
            impact = impact.max(report.impact);
            let speed = report.impact;
            crash = report.crash.map(|at| Impact { speed, at }).or(crash);
            (self.pace.report, elapsed) = (report, elapsed + STEP);
            self.pace.carry -= STEP;
        }
        (impact, crash)
    }
}

/// Steps the ship at a fixed rate through the frame time, and tells when it leaves the ground,
/// when it touches it and when it crashes. In warp only its attitude moves.
pub fn simulate(
    mut flight: Flight,
    mut ship: Single<&mut Ship>,
    mut events: MessageWriter<ShipEvent>,
    mut impacts: MessageWriter<Impact>,
) {
    let dt = flight.time.delta_secs_f64().min(MAX_FRAME);
    if matches!(ship.warp, Warp::Active { .. }) {
        rotate(&mut ship, flight.helm.controls.turn, DVec3::ZERO, dt);
        *flight.pace = Pace {
            seen: flight.pace.seen,
            lag: flight.pace.lag,
            ..Pace::default()
        };
        return;
    }
    let parked = ship.landed;
    let (impact, crash) = flight.advance(&mut ship, dt);
    if let Some(crash) = crash {
        impacts.write(crash);
    }
    if parked && !ship.landed {
        events.write(ShipEvent::Liftoff);
    }
    if impact > QUIET {
        events.write(ShipEvent::Touchdown {
            speed: impact,
            damage: ship.damage,
        });
    }
}
