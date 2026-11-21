//! Every cue renders a short, clean, audible sound, and the queue to the audio thread holds.

use super::{CLICK, RATE, frames, listen, mixer};
use crate::audio::cue::{AudioCue, ChirpKind, StepSurface};
use crate::audio::cues::Pool;
use crate::audio::scene::AudioScene;
use crate::audio::shared::Shared;
use crate::audio::steps::step_surface;
use crate::world::Surface;

/// A scene with no music and no ambience, so that only cues sound.
fn quiet() -> AudioScene {
    AudioScene {
        in_space: true,
        ..AudioScene::default()
    }
}

/// Each cue renders a finite, audible, bounded buffer that starts and ends on silence.
#[test]
fn every_cue_renders() {
    for cue in AudioCue::all() {
        let mut pool = Pool::new(RATE);
        pool.play(cue);
        let buffer: Vec<f32> = (0..frames(5.0)).map(|_| pool.frame()[0]).collect();
        let peak = buffer.iter().fold(0.0_f32, |peak, x| peak.max(x.abs()));
        let jump = buffer
            .windows(2)
            .fold(0.0_f32, |j, w| j.max((w[1] - w[0]).abs()));
        assert!(buffer.iter().all(|x| x.is_finite()), "{cue:?}");
        assert!(peak > 0.02 && peak < 0.9, "{cue:?} peak {peak}");
        assert!(jump < 0.25, "{cue:?} jump {jump}");
        assert!(buffer[0].abs() < 1e-3 && !pool.busy(), "{cue:?}");
    }
}

/// A cue pushed by the game comes out of the mixer, within range.
#[test]
fn cues_cross_to_the_mixer() {
    for cue in AudioCue::all() {
        let (mut mixer, shared) = mixer(&quiet());
        assert_eq!(listen(&mut mixer, 0.5).peak, 0.0);
        assert!(shared.push(cue));
        let stats = listen(&mut mixer, 4.0);
        assert!(
            !stats.broken && stats.peak > 0.01 && stats.peak <= 1.0,
            "{cue:?}"
        );
        assert!(stats.jump < 5.0 * CLICK, "{cue:?} jump {}", stats.jump);
    }
}

/// A flood of cues never pushes the output past full scale, and a full queue drops cues.
#[test]
fn cue_flood_is_bounded() {
    let (mut mixer, shared) = mixer(&quiet());
    let mut dropped = 0;
    for _ in 0..40 {
        dropped += AudioCue::all().filter(|cue| !shared.push(*cue)).count();
        let stats = listen(&mut mixer, 0.05);
        assert!(!stats.broken && stats.peak <= 1.0);
    }
    assert_eq!(dropped, 0);
    let pushed = (0..200).filter(|_| shared.push(AudioCue::UiMove)).count();
    assert_eq!(pushed, 64);
}

/// Cues survive the trip through one word.
#[test]
fn cues_round_trip() {
    let shared = Shared::default();
    let cues: Vec<AudioCue> = AudioCue::all().chain([AudioCue::Land(0.25)]).collect();
    for cue in &cues {
        assert!(shared.push(*cue));
    }
    for cue in &cues {
        assert_eq!(shared.pop(), Some(*cue));
    }
    assert_eq!(shared.pop(), None);
    assert_eq!(AudioCue::all().count(), 20);
    assert!(cues.contains(&AudioCue::Switch));
    assert!(cues.contains(&AudioCue::Chirp(ChirpKind::Sleepy)));
}

/// Ground covers map to footstep sounds, and sea floors to none.
#[test]
fn surfaces_map_to_steps() {
    assert_eq!(step_surface(Surface::Sand), Some(StepSurface::Sand));
    assert_eq!(step_surface(Surface::Grass), Some(StepSurface::Grass));
    assert_eq!(step_surface(Surface::Rock), Some(StepSurface::Rock));
    assert_eq!(step_surface(Surface::Snow), Some(StepSurface::Snow));
    assert_eq!(step_surface(Surface::Water), None);
}
