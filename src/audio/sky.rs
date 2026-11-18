//! The weather in the scene: rain and dust as continuous sounds, far thunder as cues.

use bevy::prelude::*;

use super::cue::AudioCue;
use super::scene::AudioScene;
use crate::screens::Screen;
use crate::space::Clock;
use crate::weather::{Forecast, THUNDER_SECONDS, Weather, thunder};

/// Writes the rain and the dust of the weather at the camera into the scene.
fn sense_weather(weather: Option<Res<Weather>>, mut scene: ResMut<AudioScene>) {
    let sky = weather.as_deref().copied().unwrap_or_default();
    if (scene.rain, scene.dust) != (sky.rain, sky.dust) {
        (scene.rain, scene.dust) = (sky.rain, sky.dust);
    }
}

/// Returns the thunder cue of the slot that `seconds` of universe time falls in, when `last`
/// was another slot: none for the first slot heard, none twice in one slot. The clap thins
/// with the air left around the listener.
pub fn clap(
    last: &mut Option<i64>,
    seconds: f64,
    weather: &Weather,
    scene: &AudioScene,
) -> Option<AudioCue> {
    let slot = (seconds / THUNDER_SECONDS).floor() as i64;
    let before = last.replace(slot);
    let level = thunder(scene.planet_seed, slot, weather)?;
    (before.is_some_and(|before| before != slot) && !scene.in_space)
        .then_some(AudioCue::Thunder(level * scene.altitude_fade))
}

/// Rolls far thunder in a storm, at most once per slot of [`THUNDER_SECONDS`].
fn rumble(
    clock: Option<Res<Clock>>,
    weather: Option<Res<Weather>>,
    scene: Res<AudioScene>,
    mut last: Local<Option<i64>>,
    mut cues: MessageWriter<AudioCue>,
) {
    let (Some(clock), Some(weather)) = (clock, weather) else {
        return;
    };
    if let Some(cue) = clap(&mut last, clock.seconds, &weather, &scene) {
        cues.write(cue);
    }
}

/// Registers the weather sounds, which follow the weather once it is read.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (sense_weather, rumble)
            .chain()
            .in_set(super::Sensing)
            .after(Forecast)
            .run_if(in_state(Screen::Gameplay)),
    );
}
