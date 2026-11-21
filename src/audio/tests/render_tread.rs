//! Takes of the reworked grounds for the owner: a walk on sand, a walk in shallow water, a wade to
//! the knee, and a glide that lands and slides on grass and on sand. Every step comes from the real
//! gait of the robot and is heard as the listener's own, through the real mixer.
//!
//! It runs only when `SHIP_AUDIO_DIR` names the folder to write to:
//! `SHIP_AUDIO_DIR=/tmp/ship-audio cargo test --features dev --lib -- audio::tests::render --nocapture`.

use std::path::Path;

use super::render::write;
use super::rig::alone;
use super::takes_steps::{heard, slide, walk};
use super::{RATE, frames, mixer};
use crate::audio::cue::{AudioCue, StepSurface};
use crate::audio::dsp::limit::CEILING;
use crate::audio::ship::feed::HullSound;
use crate::audio::spatial::Air;
use crate::audio::step::Step;
use crate::avatar::AvatarPose;
use crate::avatar::tread::stage::live;
use crate::universe::STANDARD_GRAVITY;

/// One take: its file name, its length in seconds, its ground, the depth of water on it in metres
/// and whether the world is cold enough for snow to squeak, and what the body does.
type Take = (
    &'static str,
    f32,
    StepSurface,
    (f32, bool),
    fn(f32) -> AvatarPose,
);

/// Every take.
const TAKES: [Take; 7] = [
    ("40_walk_sand", 6.0, StepSurface::Sand, (0.0, false), walk),
    (
        "41_walk_shallow_water",
        6.0,
        StepSurface::Water,
        (0.1, false),
        walk,
    ),
    (
        "42_wade_knee_deep",
        6.0,
        StepSurface::Water,
        (0.5, false),
        walk,
    ),
    (
        "43_glide_landing_slide_grass",
        4.0,
        StepSurface::Grass,
        (0.0, false),
        slide,
    ),
    (
        "44_glide_landing_slide_sand",
        4.0,
        StepSurface::Sand,
        (0.0, false),
        slide,
    ),
    (
        "46_walk_snow_warm",
        6.0,
        StepSurface::Snow,
        (0.0, false),
        walk,
    ),
    (
        "46_walk_snow_cold",
        6.0,
        StepSurface::Snow,
        (0.0, true),
        walk,
    ),
];

/// Renders `take` over a still, silent day, each step pushed at the sample it is due.
fn record(&(_, seconds, surface, (depth, cold), script): &Take) -> Vec<[f32; 2]> {
    let (mut mixer, shared) = mixer(&alone(HullSound::default()));
    let own = heard(0.0, 0.0, Air::home());
    let placed = |(t, step): (f32, Step)| {
        let step = Step {
            surface,
            depth,
            cold,
            ..step
        };
        (t + 0.5, step)
    };
    let steps = live(script, STANDARD_GRAVITY as f32, seconds - 0.5);
    let mut due = steps.into_iter().map(placed).peekable();
    (0..frames(seconds))
        .map(|i| {
            let t = i as f32 / RATE;
            while let Some((_, step)) = due.next_if(|(at, _)| *at <= t) {
                shared.push(AudioCue::Footstep(step), &own);
            }
            mixer.frame()
        })
        .collect()
}

/// Renders every take of the grounds into the folder `SHIP_AUDIO_DIR` names, and does nothing
/// without it. Every take stays within full scale and is heard.
#[test]
fn tread_takes_render() {
    let Some(dir) = std::env::var_os("SHIP_AUDIO_DIR") else {
        return;
    };
    std::fs::create_dir_all(&dir).expect("the folder can be made");
    for take in &TAKES {
        let sound = record(take);
        let peak = sound.iter().flatten().fold(0.0_f32, |p, x| p.max(x.abs()));
        assert!(peak <= CEILING && peak > 1.0e-3, "{}: {peak}", take.0);
        let path = Path::new(&dir).join(format!("{}.wav", take.0));
        write(&path, &sound).expect("the file can be written");
        println!("{}: peak {peak:.4}", path.display());
    }
}
