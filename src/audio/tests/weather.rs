//! The weather sounds: rain in the open and on the hull, the dust hiss, and far thunder.

use std::sync::Arc;

use super::{CLICK, RATE, frames, listen};
use crate::audio::cue::AudioCue;
use crate::audio::cues::Pool;
use crate::audio::mix::Mixer;
use crate::audio::scene::AudioScene;
use crate::audio::shared::Shared;
use crate::audio::sky::{LastSlot, clap};
use crate::settings::Audio as Volumes;
use crate::weather::Weather;

/// Returns the level of six seconds of `scene` with the music muted.
fn heard(scene: AudioScene) -> f64 {
    let shared = Arc::new(Shared::default());
    let volumes = Volumes {
        music: 0.0,
        ..Volumes::default()
    };
    shared.publish(&scene, &volumes);
    let mut mixer = Mixer::new(RATE, shared);
    let stats = listen(&mut mixer, 6.0);
    assert!(!stats.broken && stats.peak <= 1.0);
    stats.rms()
}

/// Rain is heard in the open and drums on the hull from the cabin, dust hisses, all within
/// full scale, and none of it is heard in space or high above the air.
#[test]
fn rain_and_dust_are_heard_outside() {
    let calm = AudioScene::default();
    let quiet = heard(calm);
    for scene in [
        AudioScene { rain: 1.0, ..calm },
        AudioScene {
            rain: 1.0,
            cabin: 1.0,
            ..calm
        },
        AudioScene { dust: 1.0, ..calm },
    ] {
        assert!(heard(scene) > quiet + 0.005, "{scene:?}");
    }
    let space = AudioScene {
        rain: 1.0,
        dust: 1.0,
        in_space: true,
        ..calm
    };
    let high = AudioScene {
        rain: 1.0,
        dust: 1.0,
        altitude_fade: 0.0,
        ..calm
    };
    assert_eq!(heard(space), 0.0);
    assert_eq!(heard(high), quiet);
}

/// Thunder rolls at most once per slot, never in the first slot heard, never in space and never
/// without a storm.
#[test]
fn thunder_claps_once_per_slot_in_a_storm() {
    let storm = Weather {
        storm: 1.0,
        cloud: 1.0,
        ..Weather::default()
    };
    let scene = AudioScene::default();
    let mut last = None;
    let claps: Vec<AudioCue> = (0..1800)
        .filter_map(|second| clap(&mut last, f64::from(second), &storm, &scene))
        .collect();
    assert!((40..=160).contains(&claps.len()), "{}", claps.len());
    assert!(
        claps
            .iter()
            .all(|cue| matches!(cue, AudioCue::Thunder(level) if *level > 0.3))
    );
    let mut first = None;
    assert_eq!(clap(&mut first, 3.0, &storm, &scene), None);
    let space = AudioScene {
        in_space: true,
        ..scene
    };
    let mut last = None;
    assert!((0..900).all(|second| clap(&mut last, f64::from(second), &storm, &space).is_none()));
    let mut last = None;
    let calm = Weather::default();
    assert!((0..900).all(|second| clap(&mut last, f64::from(second), &calm, &scene).is_none()));
}

/// Far thunder renders a finite, bounded roll that starts on silence, fades out within five
/// seconds and makes no click, and it crosses to the mixer in one word.
#[test]
fn thunder_rolls_cleanly() {
    let mut pool = Pool::new(RATE);
    pool.play(AudioCue::Thunder(1.0));
    let buffer: Vec<f32> = (0..frames(5.0)).map(|_| pool.frame()[0]).collect();
    let peak = buffer.iter().fold(0.0_f32, |peak, x| peak.max(x.abs()));
    let jump = buffer
        .windows(2)
        .fold(0.0_f32, |j, w| j.max((w[1] - w[0]).abs()));
    assert!(buffer.iter().all(|x| x.is_finite()) && peak > 0.02 && peak < 0.9);
    assert!(
        jump < CLICK && buffer[0].abs() < 1e-3 && !pool.busy(),
        "{jump}"
    );
    let shared = Shared::default();
    assert!(shared.push(AudioCue::Thunder(0.5)));
    assert_eq!(shared.pop(), Some(AudioCue::Thunder(0.5)));
}

/// The thunder slot last listened to is forgotten when the world is left: the next world
/// starts as one that has heard nothing, whatever its universe time.
#[test]
fn leaving_the_world_forgets_the_thunder_slot() {
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    use crate::screens::Screen;
    let mut app = App::new();
    app.add_plugins((StatesPlugin, crate::audio::sky::plugin))
        .init_state::<Screen>()
        .init_resource::<AudioScene>()
        .init_resource::<Weather>()
        .add_message::<AudioCue>()
        .insert_resource(crate::space::Clock { seconds: 950.0 });
    let go = |app: &mut App, screen: Screen| {
        app.world_mut()
            .resource_mut::<NextState<Screen>>()
            .set(screen);
        app.update();
    };
    go(&mut app, Screen::Gameplay);
    assert_eq!(*app.world().resource::<LastSlot>(), LastSlot(Some(105)));
    go(&mut app, Screen::Title);
    assert_eq!(*app.world().resource::<LastSlot>(), LastSlot(None));
}
