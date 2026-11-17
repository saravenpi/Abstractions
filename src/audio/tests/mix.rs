//! The master bus: bounded whatever happens, smooth gains, and the scene helpers.

use super::{CLICK, listen, mixer, planet};
use crate::audio::cue::AudioCue;
use crate::audio::dsp::limit::CEILING;
use crate::audio::scene::AudioScene;
use crate::settings::Audio as Volumes;

/// Everything at once, as loud as the scene can ask.
fn storm() -> AudioScene {
    AudioScene {
        wind: 1.0,
        shore: 1.0,
        throttle: 1.0,
        dynamic_pressure: 1.0,
        cloud_depth: 1.0,
        cabin: 1.0,
        rush: 1.0,
        boost: 1.0,
        flutter: 1.0,
        ..planet(5)
    }
}

/// With every voice at full level and cues on top, the output stays within full scale.
#[test]
fn output_never_exceeds_full_scale() {
    for scene in [
        storm(),
        AudioScene {
            submerged: 1.0,
            ..storm()
        },
        AudioScene {
            in_space: true,
            ..storm()
        },
    ] {
        let (mut mixer, shared) = mixer(&scene);
        for _ in 0..20 {
            AudioCue::all().for_each(|cue| {
                shared.push(cue);
            });
            let stats = listen(&mut mixer, 0.5);
            assert!(
                !stats.broken && stats.peak <= CEILING,
                "peak {}",
                stats.peak
            );
            assert!(stats.rms() > 0.01);
        }
    }
}

/// Renders twelve seconds of a scene and returns its level.
fn level(scene: AudioScene) -> f64 {
    let (mut mixer, _shared) = mixer(&scene);
    let stats = listen(&mut mixer, 12.0);
    assert!(!stats.broken && stats.peak <= 1.0);
    stats.rms()
}

/// Returns the level of the title scene with no music and one change applied.
fn alone(change: impl Fn(&mut AudioScene)) -> f64 {
    let mut scene = AudioScene {
        altitude_fade: 0.0,
        ..AudioScene::default()
    };
    change(&mut scene);
    level(scene)
}

/// Each ambience and flight gear voice is heard on its own, without music, and wind adds to
/// the music.
#[test]
fn each_air_voice_sounds_alone() {
    assert_eq!(alone(|_| ()), 0.0);
    assert!(alone(|scene| scene.shore = 1.0) > 0.005);
    assert!(alone(|scene| scene.submerged = 1.0) > 0.005);
    assert!(alone(|scene| scene.cloud_depth = 1.0) > 0.005);
    assert!(alone(|scene| scene.dynamic_pressure = 1.0) > 0.005);
    assert!(alone(|scene| scene.rush = 1.0) > 0.005);
    assert!(alone(|scene| scene.boost = 1.0) > 0.005);
    assert!(alone(|scene| scene.flutter = 1.0) > 0.005);
    let windy = AudioScene {
        wind: 1.0,
        ..AudioScene::default()
    };
    assert!(level(windy) > level(AudioScene::default()));
}

/// Space silences everything outside, and the engine is only heard from the cabin.
#[test]
fn space_is_silent_outside_the_cabin() {
    let space = AudioScene {
        in_space: true,
        wind: 1.0,
        shore: 1.0,
        cloud_depth: 1.0,
        dynamic_pressure: 1.0,
        throttle: 0.6,
        rush: 1.0,
        flutter: 1.0,
        ..AudioScene::default()
    };
    assert_eq!(level(space), 0.0);
    assert!(
        level(AudioScene {
            cabin: 1.0,
            ..space
        }) > 0.005
    );
}

/// Volume settings act live: zero is silence, and a sudden change makes no step in the signal.
#[test]
fn volumes_are_live_and_smooth() {
    let scene = planet(8);
    let (mut mixer, shared) = mixer(&scene);
    let loud = listen(&mut mixer, 8.0);
    assert!(loud.rms() > 0.01 && loud.jump < CLICK);
    for (master, music, effects) in [
        (0.2, 1.0, 1.0),
        (1.0, 0.0, 1.0),
        (1.0, 1.0, 0.0),
        (1.0, 1.0, 1.0),
    ] {
        let volumes = Volumes {
            master,
            music,
            effects,
        };
        shared.publish(&scene, &volumes);
        let stats = listen(&mut mixer, 1.0);
        assert!(stats.jump < CLICK, "jump {}", stats.jump);
    }
    let mute = Volumes {
        master: 0.0,
        ..Volumes::default()
    };
    shared.publish(&scene, &mute);
    listen(&mut mixer, 2.0);
    assert!(listen(&mut mixer, 1.0).peak < 1e-6);
}
