//! Carries the scene, the volumes and the cues to the audio thread, and starts the stream.

use std::sync::Arc;

use bevy::audio::Volume;
use bevy::prelude::*;

use super::cue::AudioCue;
use super::scene::AudioScene;
use super::shared::Shared;
use super::source::Synth;
use crate::settings::{Persistent, Settings};

/// The game side of the bridge to the audio thread.
#[derive(Resource, Debug, Default)]
pub(super) struct Bridge(pub(super) Arc<Shared>);

/// Marks the entity that plays the synthesiser.
#[derive(Component, Debug)]
struct Speaker;

/// Spawns the one entity that plays the synthesiser for the whole run.
///
/// Without an audio device Bevy never starts the stream: the entity waits and nothing else happens.
fn start(mut commands: Commands, bridge: Res<Bridge>, mut synths: ResMut<Assets<Synth>>) {
    let synth = synths.add(Synth::new(bridge.0.clone()));
    commands.spawn((Speaker, AudioPlayer(synth), PlaybackSettings::ONCE));
}

/// Sets the stream to unity gain once it plays.
///
/// Bevy starts a stream at the global volume, which is the master setting. The mixer applies the
/// master volume itself, smoothly, so it must not be applied twice.
fn unity(mut sinks: Query<&mut AudioSink, (With<Speaker>, Added<AudioSink>)>) {
    for mut sink in &mut sinks {
        sink.set_volume(Volume::Linear(1.0));
    }
}

/// Publishes the scene and the volume settings.
fn publish(scene: Res<AudioScene>, settings: Res<Persistent<Settings>>, bridge: Res<Bridge>) {
    bridge.0.publish(&scene, &settings.audio);
}

/// Queues the cues written this frame.
fn forward(mut cues: MessageReader<AudioCue>, bridge: Res<Bridge>) {
    for cue in cues.read() {
        bridge.0.push(*cue);
    }
}

/// Registers the feeding systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Bridge>()
        .add_systems(Startup, start)
        .add_systems(Update, unity)
        .add_systems(PostUpdate, (publish, forward));
}
