//! A new destination while the drive runs: the drive drops out first, the ship holds its heading
//! while the drop-out plays, and only then turns onto the new course, spools and warps again.

use std::time::Duration;

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use crate::computer::{Target, ship_position, target_position};
use crate::ship::body::Ship;
use crate::ship::helm::Helm;
use crate::ship::pilot::{Autopilot, Phase, SETTLE};
use crate::ship::sim::Pace;
use crate::ship::sky::Skies;
use crate::ship::warp::Warp;
use crate::ship::wells::Well;
use crate::ship::{Impact, ShipEvent, drive, route, sim};
use crate::space::{Clock, Frame};
use crate::universe::Universe;

/// Seconds of one frame of the test flight.
pub(super) const DT: f64 = 1.0 / 30.0;

/// Largest angle in radians the nose may drift while the ship holds its heading: what the
/// attitude thrusters need to stop the last trim of the cruise.
pub(super) const STILL: f64 = 2.0e-3;

/// One frame of the flight as the test saw it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Seen {
    /// What the autopilot was doing.
    pub(super) phase: Phase,
    /// State of the drive.
    pub(super) warp: Warp,
    /// Unit vector of the nose in the frame of the star.
    pub(super) nose: DVec3,
}

/// Everything the ship announced, in order.
#[derive(Resource, Debug, Default)]
struct Told(Vec<ShipEvent>);

/// Moves universe time on by the frame time and keeps what the ship announced.
fn tick(
    time: Res<Time>,
    mut clock: ResMut<Clock>,
    mut told: ResMut<Told>,
    mut events: MessageReader<ShipEvent>,
) {
    clock.seconds += time.delta_secs_f64();
    told.0.extend(events.read().copied());
}

/// Returns the route and flight systems of the game around a ship at rest beside the home
/// planet of universe 7, 400 radii out, far outside every lock sphere, and the other planets
/// of the system as destinations.
pub(super) fn flight() -> (App, Vec<Target>) {
    let universe = Universe::new(7);
    let home = universe.home();
    let (system, planet) = (universe.system(home.star), universe.planet(home));
    let well = Well::planet(&planet, 0.0);
    let pos = well.pos + well.pos.normalize() * planet.radius * 400.0;
    let mut ship = Ship::at_rest(Frame::System(home.star), pos, DQuat::IDENTITY);
    ship.vel = well.vel;
    let mut app = App::new();
    let step = Duration::from_secs_f64(DT);
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(step))
        .insert_resource(universe)
        .init_resource::<Clock>()
        .init_resource::<Autopilot>()
        .init_resource::<Skies>()
        .init_resource::<Helm>()
        .init_resource::<Pace>()
        .init_resource::<Told>()
        .add_message::<ShipEvent>()
        .add_message::<Impact>()
        .add_systems(First, tick)
        .add_systems(
            Update,
            (route::fly, sim::survey, drive::drive, sim::simulate).chain(),
        );
    app.world_mut().spawn(ship);
    let others = system.planets.iter().filter(|other| other.id != home);
    (app, others.map(|other| Target::Body(other.id)).collect())
}

/// Returns the unit vector from the ship to `target` now.
fn course(app: &mut App, target: Target) -> DVec3 {
    let ship = see_ship(app);
    let (universe, time) = (
        *app.world().resource::<Universe>(),
        app.world().resource::<Clock>().seconds,
    );
    let here = ship_position(&universe, ship.frame, ship.pos, time).expect("a ship somewhere");
    let there = target_position(&universe, target, time).expect("a planet somewhere");
    (there - here).normalize()
}

/// Returns a copy of the ship.
fn see_ship(app: &mut App) -> Ship {
    let world = app.world_mut();
    let mut ships = world.query::<&Ship>();
    ships.single(world).expect("one ship").clone()
}

/// Runs frames until the autopilot is in `stop`, or `limit` frames without one, and returns
/// what every frame before that showed.
pub(super) fn frames(app: &mut App, limit: usize, stop: Option<Phase>) -> Vec<Seen> {
    let mut seen = Vec::new();
    for _ in 0..limit {
        app.update();
        let (ship, phase) = (see_ship(app), app.world().resource::<Autopilot>().phase());
        if stop == Some(phase) {
            return seen;
        }
        seen.push(Seen {
            phase,
            warp: ship.warp,
            nose: ship.forward(),
        });
    }
    assert!(stop.is_none(), "{stop:?} not within {limit} frames");
    seen
}

/// A warp to one planet with another one picked a second into the cruise, as the tests see it.
struct Rerouted {
    /// The game, a second into the second warp.
    app: App,
    /// The last frame before the new destination was picked.
    before: Seen,
    /// Every frame from the pick until the autopilot starts to turn.
    held: Vec<Seen>,
    /// Every frame from the turn until a second into the second warp.
    after: Vec<Seen>,
    /// The new destination.
    to: Target,
}

/// Warps toward a planet, picks the planet farthest off that course a second into the cruise
/// and flies on until a second into the warp toward it.
fn rerouted() -> Rerouted {
    let (mut app, targets) = flight();
    let engage = |app: &mut App, to: Target| app.world_mut().resource_mut::<Autopilot>().engage(to);
    engage(&mut app, targets[0]);
    frames(&mut app, 3000, Some(Phase::Cruising));
    let before = *frames(&mut app, 30, None).last().unwrap();
    let old = course(&mut app, targets[0]);
    let far = |app: &mut App, to: &Target| course(app, *to).angle_between(old);
    let mut rest = targets[1..].to_vec();
    rest.sort_by(|a, b| far(&mut app, b).total_cmp(&far(&mut app, a)));
    let turn = far(&mut app, &rest[0]);
    assert!(
        turn > 0.1,
        "the new course is only {turn} rad off the old one"
    );
    engage(&mut app, rest[0]);
    let held = frames(&mut app, 600, Some(Phase::Aligning));
    let mut after = frames(&mut app, 3000, Some(Phase::Cruising));
    after.extend(frames(&mut app, 30, None));
    Rerouted {
        app,
        before,
        held,
        after,
        to: rest[0],
    }
}

/// A destination picked during a warp stops the drive at once, and the nose stays where it was
/// for the whole drop-out: the ship never turns toward the new course while the drive is engaged.
#[test]
fn a_new_destination_in_warp_drops_out_before_it_turns() {
    let Rerouted {
        before,
        held,
        after,
        ..
    } = rerouted();
    assert!(matches!(before.warp, Warp::Active { .. }));
    let still = |seen: &Seen| {
        (seen.phase, seen.warp) == (Phase::Dropping, Warp::Off)
            && seen.nose.angle_between(before.nose) < STILL
    };
    assert!(held.iter().all(still), "{held:?}");
    assert!(
        held.len() as f64 * DT >= SETTLE - DT,
        "{} frames",
        held.len()
    );
    let engaged = |seen: &&Seen| matches!(seen.warp, Warp::Active { .. });
    let turning = |seen: &Seen| matches!(seen.phase, Phase::Dropping | Phase::Aligning);
    assert!(!after.iter().filter(engaged).any(turning), "a turn in warp");
}

/// After the drop-out the ship turns, then the drive spools and engages again on the new
/// course: drop-out, align, spool, warp, in that order, each announced once.
#[test]
fn a_new_destination_in_warp_turns_then_spools_then_warps() {
    let Rerouted {
        mut app, after, to, ..
    } = rerouted();
    let mut order: Vec<Phase> = after.iter().map(|seen| seen.phase).collect();
    order.dedup();
    assert_eq!(order, [Phase::Aligning, Phase::Spooling, Phase::Cruising]);
    let last = after.last().unwrap();
    let off = last.nose.angle_between(course(&mut app, to));
    assert!(
        matches!(last.warp, Warp::Active { .. }) && off < 0.04,
        "{off}"
    );
    let wanted = [
        ShipEvent::WarpSpooling,
        ShipEvent::WarpEngaged,
        ShipEvent::WarpDropped,
        ShipEvent::WarpSpooling,
        ShipEvent::WarpEngaged,
    ];
    assert_eq!(app.world().resource::<Told>().0[..], wanted);
}
