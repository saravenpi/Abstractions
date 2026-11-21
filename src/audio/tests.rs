//! Offline tests: every buffer is rendered in memory, with no audio device.

mod app;
mod basin;
mod calls;
mod cues;
mod dsp;
mod giant;
mod harmony;
mod loudness;
mod meter;
mod meter_checks;
mod mix;
mod music;
mod planet;
mod render;
mod render_space;
mod render_steps;
mod rig;
mod scene;
mod scenes;
mod sensing;
mod ship;
mod ship_law;
mod ship_mix;
mod takes;
mod takes_ground;
mod takes_steps;
mod treads;
mod treads_heard;
mod views;
mod weather;
mod wire;

use std::sync::Arc;

use super::mix::Mixer;
use super::scene::AudioScene;
use super::shared::Shared;
use super::source::SYNTH_RATE;
use crate::settings::Audio as Volumes;

/// Sample rate the tests render at.
const RATE: f32 = SYNTH_RATE as f32;

/// Largest step between two samples that a clean signal of this synthesiser makes.
const CLICK: f32 = 0.05;

/// What a rendered stereo buffer looked like.
#[derive(Debug, Clone, Copy, Default)]
struct Stats {
    /// Frames seen.
    frames: usize,
    /// Largest absolute sample.
    peak: f32,
    /// Largest step between two samples of one channel.
    jump: f32,
    /// Sum of each channel.
    sum: [f64; 2],
    /// Sum of squares of both channels.
    energy: f64,
    /// Whether every sample was finite.
    broken: bool,
    /// Previous frame, if any.
    last: Option<[f32; 2]>,
}

impl Stats {
    /// Adds one frame.
    fn add(mut self, frame: [f32; 2]) -> Self {
        let last = self.last.unwrap_or(frame);
        for side in 0..2 {
            self.broken |= !frame[side].is_finite();
            self.peak = self.peak.max(frame[side].abs());
            self.jump = self.jump.max((frame[side] - last[side]).abs());
            self.sum[side] += f64::from(frame[side]);
            self.energy += f64::from(frame[side] * frame[side]);
        }
        self.last = Some(frame);
        self.frames += 1;
        self
    }

    /// Returns the largest constant offset of a channel.
    fn offset(&self) -> f64 {
        self.sum[0].abs().max(self.sum[1].abs()) / self.frames as f64
    }

    /// Returns the root mean square level.
    fn rms(&self) -> f64 {
        (self.energy / (2 * self.frames) as f64).sqrt()
    }
}

/// Returns the number of frames in `seconds`.
fn frames(seconds: f32) -> usize {
    (seconds * RATE) as usize
}

/// Builds a mixer and its shared values for a scene at the default volumes.
fn mixer(scene: &AudioScene) -> (Mixer, Arc<Shared>) {
    let shared = Arc::new(Shared::default());
    shared.publish(scene, &Volumes::default());
    (Mixer::new(RATE, shared.clone()), shared)
}

/// Returns a ground-level daytime scene on planet `seed`.
fn planet(seed: u64) -> AudioScene {
    AudioScene {
        planet_seed: seed,
        home: false,
        ..AudioScene::default()
    }
}

/// Renders `seconds` of a mixer and returns what came out.
fn listen(mixer: &mut Mixer, seconds: f32) -> Stats {
    (0..frames(seconds)).fold(Stats::default(), |stats, _| stats.add(mixer.frame()))
}
