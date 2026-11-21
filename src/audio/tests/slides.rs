//! The slide of a body on the ground and the reworked grounds: a slide rustles about as loud as a
//! walk, steps on sand and in water hiss and splash with no tone held in them, and snow crunches
//! softly, squeaking briefly only on a cold world.

use super::RATE;
use super::rig::{band, rms};
use super::takes_steps::heard;
use super::treads::{left, trod};
use crate::audio::cue::StepSurface;
use crate::audio::spatial::Air;
use crate::audio::step::{Contact, Step};
use crate::audio::tread::Treads;

/// Returns the left channel of two seconds of a body sliding on `surface` at 14 metres per second,
/// as the feet send it: one scrape every tenth of a second.
fn slid(surface: StepSurface) -> Vec<f32> {
    let scrape = Step {
        contact: Contact::Skid,
        force: 0.39,
        load: 1.0,
        mass: 80.0,
        ..Step::on(surface)
    };
    let own = heard(0.0, 0.0, Air::home());
    let mut pool = Treads::new(RATE);
    let gap = (0.1 * RATE) as usize;
    (0..20 * gap)
        .map(|i| {
            if i % gap == 0 {
                pool.play(&scrape, &own);
            }
            pool.frame()[0]
        })
        .collect()
}

/// A slide on grass or on sand is no more than 4 dB over a walk on the same ground, and still heard.
#[test]
fn a_slide_rustles_about_as_loud_as_a_walk() {
    for surface in [StepSurface::Grass, StepSurface::Sand] {
        let slide = rms(&slid(surface)[(0.3 * RATE) as usize..]);
        let walk = rms(&left(&trod(Step::on(surface), 5)));
        let over = 20.0 * (slide / walk).log10();
        assert!(over < 4.0 && over > -15.0, "{surface:?}: {over:+.1} dB");
    }
}

/// A step on sand, in shallow water or wading to the knee holds no tone: no third of an octave from
/// 200 Hz to 8 kHz stands far over the two beside it. A step on sand or in shallow water has died
/// away a second and a half on; wading to the knee is one long swish on purpose.
#[test]
fn sand_and_water_hold_no_tone() {
    let water = |depth: f32| Step {
        depth,
        ..Step::on(StepSurface::Water)
    };
    for (step, ends) in [
        (Step::on(StepSurface::Sand), true),
        (water(0.1), true),
        (water(0.5), false),
    ] {
        let sound = left(&trod(step, 3));
        let grid: Vec<f64> = (0..16)
            .map(|k| band(&sound, 200.0 * 2.0_f32.powf(k as f32 / 3.0)))
            .collect();
        for k in 1..15 {
            let beside = 0.5 * (grid[k - 1] + grid[k + 1]);
            assert!(grid[k] < 2.5 * beside, "{step:?}: band {k} {grid:?}");
        }
        let mut pool = Treads::new(RATE);
        pool.play(&step, &heard(0.0, 0.0, Air::home()));
        let alone: Vec<f32> = (0..(1.5 * RATE) as usize)
            .map(|_| pool.frame()[0])
            .collect();
        let tail = rms(&alone[(1.45 * RATE) as usize..]);
        assert!(!ends || tail < 0.01 * rms(&alone), "{step:?} still sounds");
    }
}

/// Returns how far each third of an octave from 200 Hz to 8 kHz of `sound` stands over the two
/// beside it, at its worst.
fn tone(sound: &[f32]) -> f64 {
    let grid: Vec<f64> = (0..16)
        .map(|k| band(sound, 200.0 * 2.0_f32.powf(k as f32 / 3.0)))
        .collect();
    (1..15)
        .map(|k| grid[k] / (0.5 * (grid[k - 1] + grid[k + 1])).max(1.0e-12))
        .fold(0.0, f64::max)
}

/// Snow crunches softly: within 5 dB of a step on soil, no tone held in a warm one, and on a cold
/// world a squeak that stays short and never rings on.
#[test]
fn snow_crunches_softly() {
    let warm = Step::on(StepSurface::Snow);
    let cold = Step { cold: true, ..warm };
    let soil = rms(&left(&trod(Step::on(StepSurface::Soil), 4)));
    for step in [warm, cold] {
        let sound = left(&trod(step, 4));
        let off = 20.0 * (rms(&sound) / soil).log10();
        assert!(off.abs() < 5.0, "{step:?}: {off:+.1} dB against soil");
        let most = if step.cold { 4.0 } else { 2.5 };
        assert!(tone(&sound) < most, "{step:?}: a tone of {}", tone(&sound));
        let mut pool = Treads::new(RATE);
        pool.play(&step, &heard(0.0, 0.0, Air::home()));
        let alone: Vec<f32> = (0..(1.0 * RATE) as usize)
            .map(|_| pool.frame()[0])
            .collect();
        assert!(rms(&alone[(0.9 * RATE) as usize..]) < 0.01 * rms(&alone));
    }
}
