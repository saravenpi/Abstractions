//! The wiring: the instruments of the ship fill the scene, and what happens to the ship and what
//! the companion says come out as cues.

use bevy::ecs::message::Messages;
use bevy::prelude::*;

use super::app::{app, enter_world};
use crate::audio::wire::{HARD_LANDING, ship_cue, warping};
use crate::audio::{AudioCue, AudioScene, ChirpKind};
use crate::companion::CompanionChirp;
use crate::ship::{ShipEvent, ShipTelemetry, WarpState};

/// Every chirp the companion can send.
const CHIRPS: [ChirpKind; 8] = [
    ChirpKind::Happy,
    ChirpKind::Curious,
    ChirpKind::Surprised,
    ChirpKind::Sleepy,
    ChirpKind::Worried,
    ChirpKind::Delight,
    ChirpKind::Hum,
    ChirpKind::Wake,
];

/// The warp of the ship reaches the scene and silences the music; a drive that only spools
/// does not.
#[test]
fn the_warp_of_the_ship_fills_the_scene() {
    let mut flying = ShipTelemetry {
        warp: WarpState::Spooling(0.5),
        ..ShipTelemetry::default()
    };
    assert!(!warping(&flying) && !warping(&ShipTelemetry::default()));
    flying.warp = WarpState::Active(1.0e9);
    assert!(warping(&flying));
    let mut app = app("wire-scene");
    enter_world(&mut app);
    app.insert_resource(flying);
    app.update();
    let scene = *app.world().resource::<AudioScene>();
    assert!(scene.warping && scene.music_level() == 0.0);
}

/// A touchdown and the three moments of a warp map to the cue that sounds them, and what
/// makes no sound maps to none.
#[test]
fn ship_events_map_to_cues() {
    let soft = ShipEvent::Touchdown {
        speed: HARD_LANDING / 2.0,
        damage: 0.0,
    };
    assert_eq!(ship_cue(&soft), Some(AudioCue::Land(0.5)));
    assert_eq!(
        ship_cue(&ShipEvent::WarpSpooling),
        Some(AudioCue::WarpSpool)
    );
    assert_eq!(
        ship_cue(&ShipEvent::WarpEngaged),
        Some(AudioCue::WarpEngage)
    );
    assert_eq!(ship_cue(&ShipEvent::WarpDropped), Some(AudioCue::WarpExit));
    assert_eq!(
        ship_cue(&ShipEvent::Headlight(true)),
        Some(AudioCue::Switch)
    );
    for silent in [ShipEvent::Boarded, ShipEvent::Left, ShipEvent::Liftoff] {
        assert_eq!(ship_cue(&silent), None);
    }
}

/// Inside the app, what happens to the ship and every chirp of the companion come out as cues,
/// in the order they were sent, and every chirp has a sound.
#[test]
fn ship_events_and_chirps_become_cues() {
    let mut app = app("wire-cues");
    let world = app.world_mut();
    world.write_message(ShipEvent::WarpEngaged);
    world.write_message(ShipEvent::Boarded);
    world.write_message(ShipEvent::WarpDropped);
    for kind in CHIRPS {
        world.write_message(CompanionChirp(kind));
    }
    app.update();
    let mut cues = app.world_mut().resource_mut::<Messages<AudioCue>>();
    let heard: Vec<AudioCue> = cues.drain().collect();
    assert_eq!(heard[..2], [AudioCue::WarpEngage, AudioCue::WarpExit]);
    let said: Vec<AudioCue> = CHIRPS.into_iter().map(AudioCue::Chirp).collect();
    assert_eq!(heard[2..], said);
    assert!(
        said.iter()
            .all(|cue| AudioCue::all().any(|known| known == *cue))
    );
}
