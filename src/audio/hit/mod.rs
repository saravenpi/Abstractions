//! Blows heard: the game sends an [`Impact`] where a blow lands and a [`Whoosh`] where a tool is
//! swung, and this module turns each into a cue placed for the listener.
//!
//! The sound follows the contact: loud with the logarithm of the energy, ringing at a pitch that
//! follows the stiffness and the mass of what was struck and of the tool, a short wet noise only when
//! the blow cut, a high tick when it glanced off. A swing whooshes louder and lower the heavier the
//! tool is. At most three blips sound for a blow, and blows leave [`super::cues`] voices free for
//! the rest.

mod impact;
mod pack;
mod sound;
pub mod tune;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

pub use impact::{Body, Impact, Outer, Whoosh};
pub use sound::{HitSound, Kind, of_impact, of_whoosh};

use super::cue::AudioCue;
use crate::render::GameCamera;

/// Returns where `at` is for a listener whose camera is `ear`: left to right from -1 to 1, and how
/// near from 0 to 1, a half at [`tune::HEARD`] metres.
pub fn place(ear: &GlobalTransform, at: Vec3) -> (f32, f32) {
    let away = at - ear.translation();
    let side = away.normalize_or_zero().dot(ear.right().as_vec3());
    (side, tune::HEARD / (tune::HEARD + away.length()))
}

/// Plays every blow that landed.
fn strike(
    mut impacts: MessageReader<Impact>,
    camera: Query<&GlobalTransform, With<GameCamera>>,
    mut cues: MessageWriter<AudioCue>,
) {
    let Some(ear) = camera.iter().next() else {
        return;
    };
    for impact in impacts.read() {
        cues.write(AudioCue::Hit(of_impact(impact, place(ear, impact.at))));
    }
}

/// Plays every tool swung.
fn swing(
    mut swings: MessageReader<Whoosh>,
    camera: Query<&GlobalTransform, With<GameCamera>>,
    mut cues: MessageWriter<AudioCue>,
) {
    let Some(ear) = camera.iter().next() else {
        return;
    };
    for swing in swings.read() {
        cues.write(AudioCue::Hit(of_whoosh(swing, place(ear, swing.at))));
    }
}

/// Registers the messages of blows and the systems that play them.
pub(super) fn plugin(app: &mut App) {
    app.add_message::<Impact>()
        .add_message::<Whoosh>()
        .add_systems(
            Update,
            (
                strike.run_if(on_message::<Impact>),
                swing.run_if(on_message::<Whoosh>),
            ),
        );
}
