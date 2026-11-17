//! Wiring to the rest of the game: the ship drives the engine, the rumble and the warp, and
//! what happens to the ship and what the companion says become cues.

use bevy::prelude::*;

use super::cue::AudioCue;
use super::scene::AudioScene;
use crate::companion::CompanionChirp;
use crate::screens::Screen;
use crate::ship::{ShipEvent, ShipTelemetry, WarpState};

/// Dynamic pressure in pascals that counts as the strongest buffeting: a ship under full boost
/// in thick air settles near it.
pub const FULL_PRESSURE: f64 = 7.5e4;

/// Touchdown speed in metres per second that counts as a hard landing.
pub const HARD_LANDING: f64 = 12.0;

/// Returns the engine throttle, the buffeting and whether the ship is in warp, as the scene
/// holds them, from the instruments of the ship.
pub fn ship_scene(telemetry: &ShipTelemetry) -> (f32, f32, bool) {
    let pressure = (telemetry.dynamic_pressure / FULL_PRESSURE).clamp(0.0, 1.0);
    (
        telemetry.throttle.abs().min(1.0) as f32,
        pressure as f32,
        matches!(telemetry.warp, WarpState::Active(_)),
    )
}

/// Returns the cue for something that happened to the ship, if it makes a sound.
pub fn ship_cue(event: &ShipEvent) -> Option<AudioCue> {
    match *event {
        ShipEvent::Touchdown { speed, .. } => {
            Some(AudioCue::Land((speed / HARD_LANDING).clamp(0.0, 1.0) as f32))
        }
        ShipEvent::WarpSpooling => Some(AudioCue::WarpSpool),
        ShipEvent::WarpEngaged => Some(AudioCue::WarpEngage),
        ShipEvent::WarpDropped => Some(AudioCue::WarpExit),
        ShipEvent::WarpRefused => Some(AudioCue::UiCancel),
        ShipEvent::Liftoff | ShipEvent::Boarded | ShipEvent::Left => None,
    }
}

/// Writes the throttle, the buffeting and the warp of the ship into the scene. The cabin is
/// already sensed from where the player is.
fn follow_ship(telemetry: Option<Res<ShipTelemetry>>, mut scene: ResMut<AudioScene>) {
    let Some(telemetry) = telemetry else {
        return;
    };
    let (throttle, pressure, warping) = ship_scene(&telemetry);
    let next = AudioScene {
        throttle,
        dynamic_pressure: pressure,
        warping,
        ..*scene
    };
    scene.set_if_neq(next);
}

/// Turns what happened to the ship and what the companion said into cues.
fn relay(
    mut ship: MessageReader<ShipEvent>,
    mut chirps: MessageReader<CompanionChirp>,
    mut cues: MessageWriter<AudioCue>,
) {
    cues.write_batch(ship.read().filter_map(ship_cue));
    cues.write_batch(chirps.read().map(|chirp| AudioCue::Chirp(chirp.0)));
}

/// Registers the wiring. The scene fields are written after the audio module has sensed the
/// world, and the messages are registered here too so that the audio plugin runs on its own.
pub(super) fn plugin(app: &mut App) {
    app.add_message::<ShipEvent>()
        .add_message::<CompanionChirp>()
        .add_systems(
            Update,
            (
                follow_ship
                    .after(super::Sensing)
                    .run_if(in_state(Screen::Gameplay)),
                relay,
            ),
        );
}
