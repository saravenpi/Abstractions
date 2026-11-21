//! The offline renderer of the footsteps: it plays the takes of `takes_steps` over a calm day
//! through the real mixer and writes each one as a WAV file beside the takes of the ship.
//!
//! It runs only when `SHIP_AUDIO_DIR` names the folder to write to:
//! `SHIP_AUDIO_DIR=/tmp/ship-audio cargo test --features dev --lib -- audio::tests::render --nocapture`.

use std::path::Path;

use super::render::write;
use super::takes_steps::{Take, steps, takes};
use super::{RATE, frames, planet};
use crate::audio::cue::AudioCue;
use crate::audio::dsp::limit::CEILING;
use crate::audio::scene::AudioScene;
use crate::audio::spatial::Air;

/// Returns the scene a take plays over: a fair day with a light wind, or a world with no air.
fn day(vacuum: bool) -> AudioScene {
    match vacuum {
        true => AudioScene {
            altitude_fade: 0.0,
            ..planet(18)
        },
        false => AudioScene {
            wind: 0.15,
            shore: 0.25,
            ..planet(5)
        },
    }
}

/// Renders `take` through the mixer, each step pushed at the sample it is due.
fn record(take: &Take) -> Vec<[f32; 2]> {
    let vacuum = take.0.contains("vacuum");
    let (mut mixer, shared) = super::mixer(&day(vacuum));
    let mut due = steps(take, if vacuum { Air::VACUUM } else { Air::home() })
        .into_iter()
        .peekable();
    let mut out = Vec::with_capacity(frames(take.1));
    for i in 0..frames(take.1) {
        let t = i as f32 / RATE;
        while let Some((_, step, spot)) = due.next_if(|(at, _, _)| *at <= t) {
            shared.push(AudioCue::Footstep(step), &spot);
        }
        out.push(mixer.frame());
    }
    out
}

/// Renders every take of the footsteps into the folder `SHIP_AUDIO_DIR` names, and does nothing
/// without it. Every take stays within full scale, and every one is heard.
#[test]
fn step_takes_render() {
    let Some(dir) = std::env::var_os("SHIP_AUDIO_DIR") else {
        return;
    };
    std::fs::create_dir_all(&dir).expect("the folder can be made");
    for take in &takes() {
        let sound = record(take);
        let peak = sound.iter().flatten().fold(0.0_f32, |p, x| p.max(x.abs()));
        assert!(peak <= CEILING && peak > 0.005, "{}: {peak}", take.0);
        let path = Path::new(&dir).join(format!("{}.wav", take.0));
        write(&path, &sound).expect("the file can be written");
        println!("{}: peak {peak:.3}", path.display());
    }
}
