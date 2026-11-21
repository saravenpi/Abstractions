//! The rig of the ship tests: a ship to start from, a scene where it sounds alone, and what
//! is measured of a rendered buffer.

use super::{frames, mixer};
use crate::audio::dsp::filter::Svf;
use crate::audio::scene::AudioScene;
use crate::audio::ship::feed::{Ch, HullSound};

/// Seconds rendered and thrown away before a measure: the pumps come up, the sound arrives.
const WARM: f32 = 5.0;

/// Returns a ship on its gear in the air of the home planet, engines running, heard from 60
/// metres at the angle its jets are loudest at.
pub(super) fn parked() -> HullSound {
    HullSound::default()
        .with(Ch::Present, 1.0)
        .with(Ch::Power, 1.0)
        .with(Ch::Air, 1.0)
        .with(Ch::Height, 0.0)
        .with(Ch::Distance, 60.0)
        .with(Ch::Aspect, 0.64)
}

/// Returns `hull` as its pilot hears it from the seat.
pub(super) fn seated(hull: HullSound) -> HullSound {
    hull.with(Ch::Inside, 1.0)
        .with(Ch::Distance, 0.0)
        .with(Ch::Aspect, -1.0)
}

/// Returns a scene with no music and no weather, where only `hull` sounds.
pub(super) fn alone(hull: HullSound) -> AudioScene {
    let mut scene = AudioScene {
        altitude_fade: 0.0,
        ..AudioScene::default()
    };
    scene.hulls[0] = hull;
    scene
}

/// Renders `seconds` of `hull` alone, once it has settled, and returns the left channel.
pub(super) fn heard(hull: HullSound, seconds: f32) -> Vec<f32> {
    let (mut mixer, _shared) = mixer(&alone(hull));
    for _ in 0..frames(WARM) {
        mixer.frame();
    }
    (0..frames(seconds)).map(|_| mixer.frame()[0]).collect()
}

/// Returns the root mean square level of `samples`.
pub(super) fn rms(samples: &[f32]) -> f64 {
    let energy: f64 = samples.iter().map(|x| f64::from(x * x)).sum();
    (energy / samples.len().max(1) as f64).sqrt()
}

/// Returns the level of `samples` in a narrow band around `hz`.
pub(super) fn band(samples: &[f32], hz: f32) -> f64 {
    let mut filter = Svf::default();
    filter.tune(hz, 3.0, super::RATE);
    let passed: Vec<f32> = samples.iter().map(|x| filter.band(*x)).collect();
    rms(&passed)
}
