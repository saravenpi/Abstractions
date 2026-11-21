//! The master bus: bounded whatever happens, smooth gains, and the scene helpers.

use super::rig::{parked, seated};
use super::{CLICK, listen, mixer, planet};
use crate::audio::cue::AudioCue;
use crate::audio::dsp::limit::CEILING;
use crate::audio::scene::AudioScene;
use crate::audio::ship::feed::{Ch, HULLS, HullSound};
use crate::settings::Audio as Volumes;

/// Returns a ship with everything it has at its loudest, beside the listener.
fn roaring() -> HullSound {
    parked()
        .with(Ch::Thrust, 2.0)
        .with(Ch::Lift, 2.0)
        .with(Ch::Warp, 1.0)
        .with(Ch::Rcs, 1.0)
        .with(Ch::Rough, 1.0)
        .with(Ch::Speed, 400.0)
        .with(Ch::Distance, 0.0)
}

/// Everything at once, as loud as the scene can ask: the pilot in a roaring ship, two more
/// roaring beside it.
fn storm() -> AudioScene {
    let mut hulls = [roaring(); HULLS];
    hulls[0] = seated(roaring());
    AudioScene {
        hulls,
        wind: 1.0,
        shore: 1.0,
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
    assert!(alone(|scene| scene.rush = 1.0) > 0.005);
    assert!(alone(|scene| scene.boost = 1.0) > 0.005);
    assert!(alone(|scene| scene.flutter = 1.0) > 0.005);
    let windy = AudioScene {
        wind: 1.0,
        ..AudioScene::default()
    };
    assert!(level(windy) > level(AudioScene::default()));
}

/// Space silences everything outside, a burning ship beside the listener too, and the ship is
/// only heard from its cabin.
#[test]
fn space_is_silent_outside_the_cabin() {
    let burning = roaring().with(Ch::Air, 0.0);
    let mut space = AudioScene {
        in_space: true,
        wind: 1.0,
        shore: 1.0,
        cloud_depth: 1.0,
        rush: 1.0,
        flutter: 1.0,
        ..AudioScene::default()
    };
    space.hulls[0] = burning;
    assert_eq!(level(space), 0.0);
    space.hulls[0] = seated(burning);
    space.cabin = 1.0;
    assert!(level(space) > 0.005);
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
            ..Volumes::default()
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
