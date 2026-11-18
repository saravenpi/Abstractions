//! Sound: procedural music, ambience and effects, all synthesised in code.
//!
//! Other modules talk to this one through two things only: they write [`AudioCue`] messages for
//! one-shot sounds and they set fields of the [`AudioScene`] resource for continuous ones. The
//! ship and the companion need neither: `wire` reads their instruments and messages.

mod ambience;
mod cue;
mod cues;
mod dsp;
mod feed;
mod mix;
mod music;
mod planet;
mod scene;
mod sense;
mod shared;
mod ship;
mod shore;
mod sky;
mod source;
mod steps;
mod suit;
mod wire;

#[cfg(test)]
mod tests;

pub use cue::{AudioCue, ChirpKind, StepSurface};
pub(crate) use dsp::rand::Rand;
pub use scene::{AudioScene, altitude_fade};

use bevy::audio::AddAudioSource;
use bevy::prelude::*;

/// The systems that fill [`AudioScene`] from the world, in `Update`.
///
/// A system that overrides a field the audio module fills itself must run after this set.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sensing;

/// Registers the cue message, the scene and the audio systems.
pub(super) fn plugin(app: &mut App) {
    app.add_message::<AudioCue>()
        .init_resource::<AudioScene>()
        .add_audio_source::<source::Synth>()
        .add_plugins((
            feed::plugin,
            sense::plugin,
            sky::plugin,
            steps::plugin,
            wire::plugin,
        ));
}
