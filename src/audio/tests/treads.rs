//! The footsteps as the treads play them: every ground sounds the way it should against the
//! others, no two steps are the same sound, and a harder and heavier step is louder.

use super::rig::{band, rms};
use super::takes_steps::heard;
use super::{RATE, frames};
use crate::audio::cue::StepSurface;
use crate::audio::spatial::{Air, Spot};
use crate::audio::step::Step;
use crate::audio::tread::Treads;

/// Seconds between two steps the tests play.
const GAP: f32 = 0.4;

/// Plays `count` copies of the listener's own `step` one after the other, [`GAP`] seconds apart,
/// in the air of home, and returns the stereo sound.
pub(super) fn trod(step: Step, count: usize) -> Vec<[f32; 2]> {
    trod_at(step, heard(0.0, 0.0, Air::home()), count)
}

/// Plays `count` copies of `step` heard as `spot` says, [`GAP`] seconds apart, and returns the
/// stereo sound.
pub(super) fn trod_at(step: Step, spot: Spot, count: usize) -> Vec<[f32; 2]> {
    let mut pool = Treads::new(RATE);
    let mut out = Vec::with_capacity(count * frames(GAP));
    for _ in 0..count {
        pool.play(&step, &spot);
        out.extend((0..frames(GAP)).map(|_| pool.frame()));
    }
    out
}

/// Returns the left channel of `sound`.
pub(super) fn left(sound: &[[f32; 2]]) -> Vec<f32> {
    sound.iter().map(|frame| frame[0]).collect()
}

/// Returns how bright four walking steps on `surface` are: their level at 4 kHz over their
/// level at 250 Hz.
fn bright(surface: StepSurface) -> f64 {
    let sound = left(&trod(Step::on(surface), 4));
    band(&sound, 4000.0) / band(&sound, 250.0)
}

/// Every ground has its colour: mud is darker than soil and soil than grass, gravel crunches,
/// sand hisses and water splashes brighter than soft soil, and the metal deck rings at its tone
/// where rock and grass do not.
#[test]
fn the_spectrum_follows_the_ground() {
    let (mud, soil, grass) = (
        bright(StepSurface::Mud),
        bright(StepSurface::Soil),
        bright(StepSurface::Grass),
    );
    assert!(
        mud < 0.5 * soil && soil < 0.7 * grass,
        "mud {mud} soil {soil} grass {grass}"
    );
    for loose in [StepSurface::Gravel, StepSurface::Sand, StepSurface::Water] {
        assert!(
            bright(loose) > 2.0 * soil,
            "{loose:?} {} soil {soil}",
            bright(loose)
        );
    }
    let ring = |surface: StepSurface| {
        let sound = left(&trod(Step::on(surface), 4));
        band(&sound, 640.0) / rms(&sound)
    };
    let metal = ring(StepSurface::Metal);
    for dull in [StepSurface::Rock, StepSurface::Grass] {
        assert!(
            metal > 2.5 * ring(dull),
            "metal {metal} {dull:?} {}",
            ring(dull)
        );
    }
}

/// The same step twice is two different sounds of about the same level: the pitch, the level,
/// the roll and the grains are drawn anew each time.
#[test]
fn no_two_steps_are_alike() {
    for surface in StepSurface::ALL {
        let sound = left(&trod(Step::on(surface), 2));
        let (first, second) = sound.split_at(frames(GAP));
        let apart = first
            .iter()
            .zip(second)
            .fold(0.0_f32, |most, (a, b)| most.max((a - b).abs()));
        let (a, b) = (rms(first), rms(second));
        assert!(
            apart > 0.1 * first.iter().fold(0.0_f32, |p, x| p.max(x.abs())),
            "{surface:?}"
        );
        assert!(
            a > 0.5 * b && b > 0.5 * a && a > 1.0e-3,
            "{surface:?}: {a} {b}"
        );
    }
}

/// A step that lands harder is louder, and on gravel one that carries more weight breaks more
/// grains and is louder too.
#[test]
fn a_harder_step_is_louder() {
    for surface in StepSurface::ALL {
        let soft = rms(&left(&trod(Step::on(surface), 4)));
        let hard = rms(&left(&trod(
            Step {
                force: 0.4,
                ..Step::on(surface)
            },
            4,
        )));
        assert!(hard > 1.25 * soft, "{surface:?}: {hard} {soft}");
    }
    let gravel = Step::on(StepSurface::Gravel);
    let light = rms(&left(&trod(
        Step {
            load: 1.0,
            ..gravel
        },
        4,
    )));
    let heavy = rms(&left(&trod(
        Step {
            load: 8.0,
            ..gravel
        },
        4,
    )));
    assert!(heavy > 1.2 * light, "{heavy} {light}");
}
