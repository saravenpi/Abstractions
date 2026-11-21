//! The loudness meter against signals whose reading is known.

use std::f32::consts::TAU;

use super::meter::measure;
use super::{RATE, frames};

/// Returns `seconds` of a sine of `hz` on both channels, `db` decibels under full scale, from a
/// starting `phase` in turns.
fn tone(hz: f32, db: f32, seconds: f32, phase: f32) -> Vec<[f32; 2]> {
    let level = 10.0_f32.powf(db / 20.0);
    (0..frames(seconds))
        .map(|i| [level * (TAU * (hz * i as f32 / RATE + phase)).sin(); 2])
        .collect()
}

/// The reference of EBU R 128: a 1 kHz sine at -23 dBFS on both channels reads -23 LUFS, at
/// every time scale, with no range, and its true peak is its level.
#[test]
fn a_reference_tone_reads_its_level() {
    let read = measure(&tone(1000.0, -23.0, 20.0, 0.0));
    for loudness in [read.integrated, read.short_term, read.momentary] {
        assert!((loudness + 23.0).abs() < 0.1, "{loudness}");
    }
    assert!(read.range < 0.1 && (read.true_peak + 23.0).abs() < 0.1);
    let louder = measure(&tone(1000.0, -13.0, 20.0, 0.0));
    assert!((louder.integrated + 13.0).abs() < 0.1);
}

/// The K-weighting hears a low tone as quieter than a mid tone of the same level, and a high
/// tone as louder.
#[test]
fn the_weighting_follows_the_ear() {
    let read = |hz: f32| measure(&tone(hz, -23.0, 10.0, 0.0)).integrated;
    assert!(read(40.0) < read(1000.0) - 2.0);
    assert!(read(6000.0) > read(1000.0) + 2.0);
}

/// Silence gates out: a tone followed by silence, or by a tone far under it, reads as the tone
/// alone. Silence alone reads no loudness at all.
#[test]
fn silence_gates_out() {
    let mut sound = tone(1000.0, -23.0, 10.0, 0.0);
    sound.extend(vec![[0.0; 2]; frames(10.0)]);
    assert!((measure(&sound).integrated + 23.0).abs() < 0.2);
    let mut steps = tone(1000.0, -23.0, 10.0, 0.0);
    steps.extend(tone(1000.0, -45.0, 10.0, 0.0));
    assert!((measure(&steps).integrated + 23.0).abs() < 0.2);
    let nothing = vec![[0.0; 2]; frames(5.0)];
    assert_eq!(measure(&nothing).integrated, f64::NEG_INFINITY);
}

/// The loudness range of a programme that spends half its time 10 LU lower is 10 LU.
#[test]
fn the_range_is_the_spread() {
    let mut sound = tone(1000.0, -20.0, 30.0, 0.0);
    sound.extend(tone(1000.0, -30.0, 30.0, 0.0));
    let read = measure(&sound);
    assert!((read.range - 10.0).abs() < 0.5, "{}", read.range);
    assert!((read.short_term + 20.0).abs() < 0.1);
}

/// A tone at a quarter of the sample rate, sampled between its crests, peaks 3 dB over its
/// samples: the true peak finds the crest.
#[test]
fn the_true_peak_is_between_the_samples() {
    let sound = tone(RATE / 4.0, 0.0, 1.0, 0.125);
    let sampled = sound
        .iter()
        .fold(0.0_f32, |peak, frame| peak.max(frame[0].abs()));
    assert!((sampled - 0.707).abs() < 0.01);
    let read = measure(&sound);
    assert!(read.true_peak.abs() < 0.3, "{}", read.true_peak);
}
