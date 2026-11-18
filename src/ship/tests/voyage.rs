//! The whole game end to end, without a window: a new world, a flight to another planet, a
//! landing, a walk and a reload. Every stage asserts what the stage before left behind.

mod away;
mod back;
mod ground;
mod stranded;

use bevy::prelude::*;

use super::rig::{FLIGHT, Game};
use crate::audio::{AudioScene, ChirpKind};
use crate::companion::CompanionChirp;
use crate::render::CloudCrossing;
use crate::settings::{Keymap, Persistent, Settings};
use crate::ship::shake::Buffet;
use crate::ship::{ShipEvent, ShipTelemetry};
use crate::space::{Anchor, Frame, Location};
use crate::universe::{BodyId, PlanetSpec, Universe};

/// Returns the key the settings bind to what `pick` names.
fn key(game: &Game, pick: impl Fn(&Keymap) -> KeyCode) -> KeyCode {
    let settings = game.app.world().resource::<Persistent<Settings>>();
    pick(&settings.controls.keys)
}

/// Returns the planet the player is at and what the universe says about it.
fn here(game: &Game) -> (BodyId, PlanetSpec) {
    let body = game.get::<Location>().body;
    (body, game.get::<Universe>().planet(body))
}

/// The vertical thrusters lift the landed ship off the ground, and the companion notices.
fn lifts_off(game: &mut Game) {
    assert!(game.ship().landed && game.get::<ShipTelemetry>().landed);
    game.press(key(game, |keys| keys.ship.rise));
    game.until("the ship to leave the ground", 120, |game| {
        !game.ship().landed
    });
    game.until("the ship to climb", 300, |game| {
        game.get::<ShipTelemetry>().altitude_ground > 15.0
    });
    let log = game.get::<super::rig::Log>();
    assert!(log.ship.contains(&ShipEvent::Liftoff));
    assert!(log.chirps.contains(&CompanionChirp(ChirpKind::Surprised)));
    let ship = game.ship();
    assert!(ship.vel.dot(ship.pos.normalize()) > 1.0, "not climbing");
    game.press(key(game, |keys| keys.ship.throttle_up));
    game.pace(FLIGHT);
}

/// The climbing ship crosses the cloud band: the band is heard at its peak in the middle, the
/// crossing depth rises only where the cloud field holds a cloud on the way, never past full,
/// and is zero again above the band, and the view never shakes more than fully.
fn crosses_the_clouds(game: &mut Game) {
    let (_, planet) = here(game);
    assert_eq!(game.get::<CloudCrossing>().depth, 0.0);
    let (mut peak, mut shake, mut heard) = (0.0f32, 0.0f32, 0.0f32);
    for _ in 0..3000 {
        if game.get::<ShipTelemetry>().altitude_sea > planet.cloud_top + 60.0 {
            break;
        }
        game.step(1);
        assert!(game.hull_in_view(), "the climbing ship left the frame");
        peak = peak.max(game.get::<CloudCrossing>().depth);
        shake = shake.max(game.get::<Buffet>().level);
        heard = heard.max(game.get::<AudioScene>().cloud_depth);
    }
    assert!(game.get::<ShipTelemetry>().altitude_sea > planet.cloud_top);
    assert!((0.0..=1.0).contains(&peak), "the crossing peaked at {peak}");
    assert!(heard > 0.9 && shake <= 1.0, "{heard} {shake}");
    game.step(2);
    assert_eq!(game.get::<CloudCrossing>().depth, 0.0);
}

/// Above the top of the atmosphere the ship and the render anchor are in the frame of the star,
/// and the change of frame moves the ship no farther than it flies in a frame.
fn reaches_space(game: &mut Game) -> BodyId {
    let (home, planet) = here(game);
    let system = Frame::System(home.star);
    let mut last = game.get::<ShipTelemetry>();
    for _ in 0..3000 {
        game.step(1);
        let framed = game.hull_in_view();
        assert!(framed, "the ship left the frame on its way out");
        let now = game.get::<ShipTelemetry>();
        if now.frame == system {
            break;
        }
        last = now;
    }
    let now = game.get::<ShipTelemetry>();
    assert_eq!(now.frame, system, "the ship never left the air");
    assert!(now.altitude_sea > planet.atmosphere_height);
    let jump = (now.altitude_sea - last.altitude_sea).abs();
    let flown = 2.0 * last.surface_speed * FLIGHT + 5.0;
    assert!(
        jump < flown,
        "the height jumped by {jump} m at the top of the air"
    );
    let (ship, anchor) = (game.ship(), game.get::<Anchor>());
    assert_eq!((ship.frame, anchor.frame), (system, system));
    assert!(
        anchor.origin.distance(ship.pos) < 1e3,
        "the anchor left the ship"
    );
    assert!(game.get::<AudioScene>().in_space);
    home
}

/// In space the chase view still shows the whole ship.
fn frames_the_ship(game: &mut Game) {
    game.step(10);
    assert!(game.hull_in_view(), "the ship is not all in view in space");
}

/// Runs one stage of the voyage and prints how long it took.
fn stage<T>(name: &str, game: &mut Game, run: impl FnOnce(&mut Game) -> T) -> T {
    let start = std::time::Instant::now();
    let out = run(game);
    println!("voyage: {name} took {:.1?}", start.elapsed());
    out
}

/// A new game: on foot on Kodya, into the ship and its chase view, up through the clouds into
/// space with the whole hull in the frame, by warp to
/// another planet of the system, down onto it, out on foot, and back to the same place from the
/// save. Then into the air again, and back in the seat from a save taken in flight.
#[test]
fn a_new_game_flies_to_another_planet_and_resumes() {
    let mut game = Game::full("voyage");
    let game = &mut game;
    stage("load", game, |game| game.start("Voyage", None));
    stage("stand", game, ground::stands_on_kodya);
    stage("base", game, ground::starts_at_the_base);
    stage("board", game, ground::boards);
    stage("chase view", game, ground::looks_from_behind);
    stage("lift off", game, lifts_off);
    stage("clouds", game, crosses_the_clouds);
    let home = stage("space", game, reaches_space);
    stage("space view", game, frames_the_ship);
    let far = stage("warp", game, |game| {
        away::warps_to_another_planet(game, home)
    });
    stage("land", game, |game| away::lands(game, far));
    stage("step out", game, |game| away::steps_out(game, far));
    stage("resume on foot", game, |game| {
        back::resumes_on_foot(game, far)
    });
    stage("resume in flight", game, |game| {
        back::resumes_in_flight(game, far)
    });
}
