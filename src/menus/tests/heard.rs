//! Keeping the interface sounds a test app asks for, since a message lives two frames only.

use bevy::prelude::*;

use crate::audio::AudioCue;

/// Every sound asked for since the last call to [`cues`].
#[derive(Resource, Default)]
pub(super) struct Heard(Vec<AudioCue>);

/// Keeps the sounds of the frame.
pub(super) fn hear(mut cues: MessageReader<AudioCue>, mut heard: ResMut<Heard>) {
    heard.0.extend(cues.read().copied());
}

/// Returns and forgets every sound asked for since the last call.
pub(super) fn cues(app: &mut App) -> Vec<AudioCue> {
    std::mem::take(&mut app.world_mut().resource_mut::<Heard>().0)
}
