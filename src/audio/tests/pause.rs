//! The pause: the sounds of the world fade out and hold silent, the music and the interface play
//! on, and a resume brings the world back with no burst of what was asked while it stood still.

use std::sync::Arc;

use super::rig::{parked, rms};
use super::{RATE, frames, mixer};
use crate::audio::cue::{AudioCue, StepSurface};
use crate::audio::mix::Mixer;
use crate::audio::scene::AudioScene;
use crate::audio::shared::Shared;
use crate::audio::ship::feed::Ch;
use crate::audio::spatial::Spot;
use crate::audio::step::Step;
use crate::settings::Audio as Volumes;

/// Returns the volumes of the tests of the pause: the music off, so that only the world sounds.
fn quiet() -> Volumes {
    Volumes {
        music: 0.0,
        ..Volumes::default()
    }
}

/// Returns a stormy day with a ship burning 60 metres off, `paused` or not.
fn storm(paused: bool) -> AudioScene {
    let mut scene = AudioScene {
        wind: 1.0,
        rain: 1.0,
        paused,
        ..AudioScene::default()
    };
    scene.hulls[0] = parked().with(Ch::Thrust, 1.0);
    scene
}

/// Returns the loudest root mean square of a 50 millisecond window of `sound`.
fn loudest(sound: &[f32]) -> f64 {
    sound.chunks(frames(0.05)).map(rms).fold(0.0, f64::max)
}

/// Returns `seconds` of the left channel of `mixer`.
fn take(mixer: &mut Mixer, seconds: f32) -> Vec<f32> {
    (0..frames(seconds)).map(|_| mixer.frame()[0]).collect()
}

/// Returns a mixer that has played the storm for four seconds with the music off and has just been
/// paused, its shared values, and the last two seconds of the steady storm.
fn paused() -> (Mixer, Arc<Shared>, Vec<f32>) {
    let (mut mixer, shared) = mixer(&storm(false));
    shared.publish(&storm(false), &quiet());
    let steady = take(&mut mixer, 4.0)[frames(2.0)..].to_vec();
    shared.publish(&storm(true), &quiet());
    (mixer, shared, steady)
}

/// Paused, the world is silent within 0.25 seconds and holds silent, and the interface still
/// sounds.
#[test]
fn a_pause_silences_the_world() {
    let (mut mixer, shared, steady) = paused();
    let fading = take(&mut mixer, 0.25);
    assert!(rms(&fading[fading.len() - frames(0.02)..]) < 0.01 * rms(&steady));
    let still = take(&mut mixer, 1.0);
    assert!(
        still.iter().all(|x| x.abs() < 1.0e-6),
        "the world still sounds"
    );
    shared.push(AudioCue::UiConfirm, &Spot::FLAT);
    assert!(
        rms(&take(&mut mixer, 0.5)) > 1.0e-4,
        "the interface fell silent"
    );
}

/// Resumed, the world is back at its steady level within half a second, and what was asked during
/// the pause never plays: no window after the resume is louder than the steady storm.
#[test]
fn a_resume_brings_the_world_back_without_a_burst() {
    let (mut mixer, shared, steady) = paused();
    take(&mut mixer, 0.5);
    for _ in 0..20 {
        shared.push(AudioCue::Footstep(Step::on(StepSurface::Rock)), &Spot::FLAT);
    }
    shared.push(AudioCue::Thunder(1.0), &Spot::FLAT);
    take(&mut mixer, 0.5);
    shared.publish(&storm(false), &quiet());
    let back = take(&mut mixer, 1.0);
    assert!(
        loudest(&back) <= 1.1 * loudest(&steady),
        "{} {}",
        loudest(&back),
        loudest(&steady)
    );
    assert!(rms(&back[frames(0.5)..]) > 0.8 * rms(&steady));
}

/// A pause leaves the music as it was: the same piece, sample for sample.
#[test]
fn a_pause_leaves_the_music_playing() {
    let played = |paused: bool| {
        let scene = AudioScene {
            paused,
            ..AudioScene::default()
        };
        let (mut mixer, _shared) = mixer(&scene);
        (0..frames(3.0))
            .map(|_| mixer.frame()[0])
            .collect::<Vec<f32>>()
    };
    let (on, still) = (played(false), played(true));
    assert!(rms(&on) > 0.005);
    assert_eq!(on, still);
}
