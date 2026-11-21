//! Wiring to the rest of the game: the warp of the ship silences the music, what happens to the
//! ship becomes cues of the cabin, and what the companion says comes from where it floats. The
//! voice of the ship itself follows what the ship module writes into the scene.

use avian3d::prelude::{Physics, PhysicsTime};
use bevy::prelude::*;

use super::cue::AudioCue;
use super::scene::AudioScene;
use super::spatial::tune::{BUDDY, CABIN_REACH, PANEL};
use super::spatial::{Listener, Origin, Sound};
use crate::companion::{Companion, CompanionChirp};
use crate::net::Remote;
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

/// Returns where a sound of the cabin of the ship starts for `listener`: from the panel ahead of
/// the seat while the ears are in the cabin, which rings with it, from the cockpit `at` through
/// the hull while they are outside, and flat when no cockpit is known.
pub fn cabin(listener: &Listener, at: Option<Vec3>) -> Origin {
    let (panel, reach) = PANEL;
    match (listener.cabin, at) {
        (true, _) => Origin::At(listener.at + listener.turn * Vec3::from(panel), reach),
        (false, Some(at)) => Origin::At(at, CABIN_REACH),
        (false, None) => Origin::Flat,
    }
}

/// Writes into the scene whether the world stands still: the pause menu freezes physics out of a
/// session. In a session the world goes on behind the menu, and so do its sounds.
fn follow_pause(time: Option<Res<Time<Physics>>>, mut scene: ResMut<AudioScene>) {
    let paused = time.is_some_and(|time| time.is_paused());
    if scene.paused != paused {
        scene.paused = paused;
    }
}

/// Turns what happened to the ship into sounds of its cabin, and what the companion said into
/// sounds from where it floats: flat when it floats nowhere.
fn relay(
    (mut ship, mut chirps): (MessageReader<ShipEvent>, MessageReader<CompanionChirp>),
    companion: Query<&GlobalTransform, (With<Companion>, Without<Remote>)>,
    (listener, scene): (Res<Listener>, Res<AudioScene>),
    mut sounds: MessageWriter<Sound>,
) {
    let inside = cabin(&listener, scene.cockpit);
    sounds.write_batch(
        ship.read()
            .filter_map(ship_cue)
            .map(|cue| Sound { cue, from: inside }),
    );
    let from = companion
        .single()
        .map_or(Origin::Flat, |at| Origin::At(at.translation(), BUDDY));
    sounds.write_batch(chirps.read().map(|chirp| Sound {
        cue: AudioCue::Chirp(chirp.0),
        from,
    }));
}

/// Registers the wiring. The scene fields are written after the audio module has sensed the
/// world, and the messages are registered here too so that the audio plugin runs on its own.
pub(super) fn plugin(app: &mut App) {
    app.add_message::<ShipEvent>()
        .add_message::<CompanionChirp>()
        .add_systems(
            Update,
            (
                (follow_ship, follow_pause)
                    .after(super::Sensing)
                    .run_if(in_state(Screen::Gameplay)),
                relay,
            ),
        );
}
