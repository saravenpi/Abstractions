//! The weather in the scene: rain and dust as continuous sounds, far thunder as sounds from a
//! bearing of its own a kilometre off, late by the time sound takes to cross it.

use bevy::prelude::*;

use super::cue::AudioCue;
use super::scene::AudioScene;
use super::spatial::tune::{THUNDER, THUNDER_RISE};
use super::spatial::{Listener, Origin, Sound};
use crate::screens::Screen;
use crate::space::Clock;
use crate::universe::hash::child;
use crate::weather::{Forecast, THUNDER_SECONDS, Weather, thunder};

/// Writes the rain and the dust of the weather at the camera into the scene.
fn sense_weather(weather: Option<Res<Weather>>, mut scene: ResMut<AudioScene>) {
    let sky = weather.as_deref().copied().unwrap_or_default();
    if (scene.rain, scene.dust) != (sky.rain, sky.dust) {
        (scene.rain, scene.dust) = (sky.rain, sky.dust);
    }
}

/// Returns the thunder cue of the slot that `seconds` of universe time falls in, when `last`
/// was another slot: none for the first slot heard, none twice in one slot. How the air carries
/// the clap is the work of the spatial path.
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
        .then_some(AudioCue::Thunder(level))
}

/// The thunder slot last listened to, nothing until one was heard in the world being played.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LastSlot(pub Option<i64>);

/// Returns where the clap of `slot` on the planet of `seed` rolls from for `listener`: a bearing
/// of its own, [`THUNDER`] metres off, up in the clouds.
pub fn bolt(listener: &Listener, seed: u64, slot: i64) -> Vec3 {
    let turn =
        (child(seed, slot as u64) >> 40) as f32 / (1_u64 << 24) as f32 * std::f32::consts::TAU;
    let flat = listener.up.any_orthonormal_vector();
    let away = Quat::from_axis_angle(listener.up, turn) * flat;
    listener.at + (away + listener.up * THUNDER_RISE) * THUNDER
}

/// Rolls far thunder in a storm, at most once per slot of [`THUNDER_SECONDS`].
fn rumble(
    (clock, weather): (Option<Res<Clock>>, Option<Res<Weather>>),
    (scene, listener): (Res<AudioScene>, Res<Listener>),
    mut last: ResMut<LastSlot>,
    mut sounds: MessageWriter<Sound>,
) {
    let (Some(clock), Some(weather)) = (clock, weather) else {
        return;
    };
    let mut slot = last.0;
    let cue = clap(&mut slot, clock.seconds, &weather, &scene);
    last.set_if_neq(LastSlot(slot));
    if let (Some(cue), Some(slot)) = (cue, slot) {
        let at = bolt(&listener, scene.planet_seed, slot);
        sounds.write(Sound {
            cue,
            from: Origin::At(at, THUNDER),
        });
    }
}

/// Forgets the slot last listened to when the world is left, so that the first slot of the
/// next world is its first one heard and never claps against the time of another world.
fn forget(mut last: ResMut<LastSlot>) {
    last.set_if_neq(LastSlot::default());
}

/// Registers the weather sounds, which follow the weather once it is read.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<LastSlot>()
        .add_systems(OnExit(Screen::Gameplay), forget)
        .add_systems(
            Update,
            (sense_weather, rumble)
                .chain()
                .in_set(super::Sensing)
                .after(Forecast)
                .run_if(in_state(Screen::Gameplay)),
        );
}
