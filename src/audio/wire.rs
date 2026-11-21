//! Wiring to the rest of the game: the warp of the ship silences the music, and what happens
//! to the ship and what the companion says become cues. The voice of the ship itself follows
//! what the ship module writes into the scene.

use bevy::prelude::*;

use super::cue::AudioCue;
use super::scene::AudioScene;
use crate::companion::CompanionChirp;
use crate::screens::Screen;
use crate::ship::{ShipEvent, ShipTelemetry, WarpState};

/// Touchdown speed in metres per second that counts as a hard landing.
pub const HARD_LANDING: f64 = 12.0;

/// Returns whether the ship is in warp, from its instruments.
pub fn warping(telemetry: &ShipTelemetry) -> bool {
    matches!(telemetry.warp, WarpState::Active(_))
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
        ShipEvent::Headlight(_) => Some(AudioCue::Switch),
        ShipEvent::Liftoff | ShipEvent::Boarded | ShipEvent::Left => None,
    }
}

/// Writes the warp of the ship into the scene. The cabin is already sensed from where the
/// player is.
fn follow_ship(telemetry: Option<Res<ShipTelemetry>>, mut scene: ResMut<AudioScene>) {
    let Some(telemetry) = telemetry else {
        return;
    };
    let warping = warping(&telemetry);
    if scene.warping != warping {
        scene.warping = warping;
    }
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
