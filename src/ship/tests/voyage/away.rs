//! The far half of the voyage: the warp to another planet, the landing and the first steps.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::{here, key};
use crate::audio::{AudioCue, ChirpKind};
use crate::companion::CompanionChirp;
use crate::computer::{Atlas, Engage, RouteStatus, Target, ship_position, target_position};
use crate::render::FlightFx;
use crate::ship::berth::landing_spot;
use crate::ship::landing::GEAR_DROP;
use crate::ship::tests::rig::{FRAME, Game, Log};
use crate::ship::warp::Warp;
use crate::ship::wells::PLANET_LOCK;
use crate::ship::{Autopilot, Phase, ShipEvent, ShipTelemetry};
use crate::space::{Clock, Frame, Location, Travel};
use crate::universe::{BodyId, Universe};
use crate::world::globe::Face;
use crate::world::{Planet, Terrain};

/// Returns the planet of the system of `home` that is nearest to it now, itself excluded.
fn neighbour(game: &Game, home: BodyId) -> BodyId {
    let (universe, time) = (game.get::<Universe>(), game.get::<Clock>().seconds);
    let system = universe.system(home.star);
    let from = universe.planet_position(home, time);
    let gap = |id: BodyId| universe.planet_position(id, time).distance(from);
    let others = system.planets.iter().map(|planet| planet.id);
    others
        .filter(|id| *id != home)
        .min_by(|a, b| gap(*a).total_cmp(&gap(*b)))
        .expect("the home system has another planet")
}

/// Returns the metres from the ship to the centre of planet `body`.
fn distance_to(game: &mut Game, body: BodyId) -> f64 {
    let (universe, time) = (game.get::<Universe>(), game.get::<Clock>().seconds);
    let ship = game.ship();
    let here = ship_position(&universe, ship.frame, ship.pos, time).expect("the ship is somewhere");
    let there = target_position(&universe, Target::Body(body), time).expect("the planet exists");
    here.distance(there)
}

/// Returns where each of `wanted` first shows in `seen`, failing when one is missing.
fn order<T: PartialEq + std::fmt::Debug>(seen: &[T], wanted: &[T]) -> Vec<usize> {
    let find = |item: &T| seen.iter().position(|other| other == item);
    let found: Option<Vec<usize>> = wanted.iter().map(find).collect();
    found.unwrap_or_else(|| panic!("{wanted:?} not all in {seen:?}"))
}

/// Engaging the autopilot to another planet of the system spools the drive, warps and drops
/// the ship out at the lock distance of that planet, which becomes the planet the world streams.
pub fn warps_to_another_planet(game: &mut Game, home: BodyId) -> BodyId {
    let far = neighbour(game, home);
    game.app.world_mut().trigger(Engage(Target::Body(far)));
    game.until("the drive to spool", 3000, |game| {
        game.get::<Autopilot>().phase() == Phase::Spooling
    });
    assert_eq!(game.get::<RouteStatus>(), RouteStatus::Spooling);
    for key in [
        key(game, |keys| keys.ship.rise),
        key(game, |keys| keys.ship.throttle_up),
    ] {
        game.release(key);
    }
    game.until("the ship to go to warp", 400, |game| {
        game.get::<Autopilot>().phase() == Phase::Cruising
    });
    game.step(40);
    assert!(game.get::<FlightFx>().warp > 0.9, "no warp effect");
    assert!(game.get::<crate::audio::AudioScene>().warping);
    game.until("the ship to arrive", 9000, |game| {
        game.get::<Autopilot>().phase() == Phase::Arrived
    });
    game.step(3);
    arrived(game, far);
    far
}

/// Checks the arrival: out of warp within the lock distance the autopilot aims for, the events
/// and the sounds in order, and the new planet streamed.
fn arrived(game: &mut Game, far: BodyId) {
    let lock = game.get::<Universe>().planet(far).radius * PLANET_LOCK;
    let gap = distance_to(game, far) / lock;
    assert!(gap > 1.0 && gap < 1.35, "dropped out at {gap} lock radii");
    assert_eq!(game.ship().warp, Warp::Off);
    assert_eq!(game.get::<RouteStatus>(), RouteStatus::Arrived);
    let log = game.get::<Log>();
    let events = [
        ShipEvent::WarpSpooling,
        ShipEvent::WarpEngaged,
        ShipEvent::WarpDropped,
    ];
    let cues = [
        AudioCue::WarpSpool,
        AudioCue::WarpEngage,
        AudioCue::WarpExit,
    ];
    assert!(order(&log.ship, &events).is_sorted());
    assert!(order(&log.cues, &cues).is_sorted());
    assert_eq!(game.get::<Location>().body, far);
    game.until("the new planet to stream", 10, |game| {
        game.get::<Planet>().body == far
    });
    game.step(40);
    assert_eq!(game.get::<FlightFx>().warp, 0.0);
}

/// Puts the ship 12 m over dry, level ground of the streamed planet `far`, at rest. The descent
/// from the lock distance is flown by hand in the game and is not part of this test.
fn hover_over_land(game: &mut Game, far: BodyId) {
    let globe = game.get::<Planet>().globe;
    let north = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    let spawn = north.find_spawn();
    let height = |pos: DVec3| f64::from(globe.height(pos.normalize()));
    let near = north.embed(spawn.x, 0.0, spawn.z);
    let up = landing_spot(&height, near).normalize();
    let ground = globe.radius() + height(up).max(0.0);
    game.with_ship(|ship| {
        ship.frame = Frame::Surface(far);
        ship.pos = up * (ground + GEAR_DROP + 12.0);
        ship.rot = DQuat::from_rotation_arc(DVec3::Y, up);
        (ship.vel, ship.spin, ship.throttle) = (DVec3::ZERO, DVec3::ZERO, 0.0);
    });
    game.step(2);
}

/// Holding the thrusters down from a hover sets the ship on the ground, where it comes to rest.
pub fn lands(game: &mut Game, far: BodyId) {
    hover_over_land(game, far);
    assert_eq!(
        game.get::<crate::space::Anchor>().frame,
        Frame::Surface(far)
    );
    let before = game.get::<Log>().ship.len();
    game.press(key(game, |keys| keys.ship.sink));
    game.until("the ship to land", 900, |game| game.ship().landed);
    game.release(key(game, |keys| keys.ship.sink));
    game.step(2);
    let (log, telemetry) = (game.get::<Log>(), game.get::<ShipTelemetry>());
    let touched = log.ship[before..].iter().find_map(|event| match event {
        ShipEvent::Touchdown { speed, damage } => Some((*speed, *damage)),
        _ => None,
    });
    let (speed, damage) = touched.expect("a touchdown");
    assert!(speed < 6.0 && damage == 0.0, "touched down at {speed}");
    assert!(telemetry.landed && telemetry.altitude_ground < 1.0);
    assert!(log.cues.iter().any(|cue| matches!(cue, AudioCue::Land(_))));
    assert!(game.get::<Atlas>().visited(Target::Body(far)));
}

/// The board key puts the player on the ground of the new planet beside the ship, and the
/// companion is delighted by the first steps there.
pub fn steps_out(game: &mut Game, far: BodyId) {
    game.pace(FRAME);
    let seat = game.walker().feet;
    game.wait("the ground under the ship", |game| game.has_ground(seat));
    game.press(key(game, |keys| keys.ship.board));
    game.step(1);
    game.release(key(game, |keys| keys.ship.board));
    game.step(2);
    assert_eq!(game.get::<Location>().travel, Travel::OnFoot);
    assert!(game.get::<Log>().ship.contains(&ShipEvent::Left));
    game.until("the player to stand", 300, |game| game.walker().grounded);
    game.step(30);
    let (walker, (body, planet)) = (game.walker(), here(game));
    assert_eq!((walker.body, body), (Some(far), far));
    assert!(walker.grounded && !walker.seated);
    assert!(
        walker.gap.abs() < 0.6,
        "feet {} m off the ground",
        walker.gap
    );
    assert!(walker.at.length() > planet.radius, "under the sea");
    assert!(game.get::<Atlas>().walked.contains(&far));
    let chirps = game.get::<Log>().chirps;
    assert!(chirps.contains(&CompanionChirp(ChirpKind::Delight)));
}
