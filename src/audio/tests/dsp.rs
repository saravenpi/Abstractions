//! The signal building blocks: limiter, offset removal, oscillators, filters and reverb.

use super::RATE;
use crate::audio::dsp::filter::{Lag, Svf, glide};
use crate::audio::dsp::limit::{CEILING, DcBlock, KNEE, limit};
use crate::audio::dsp::osc::{note_hz, sine, triangle};
use crate::audio::dsp::rand::{Noise, Rand};
use crate::audio::dsp::reverb::Reverb;

/// The limiter is the identity below its knee, monotonic, and bounded for any input.
#[test]
fn limiter_is_bounded() {
    assert_eq!(limit(0.3), 0.3);
    assert_eq!(limit(-KNEE), -KNEE);
    assert_eq!(limit(f32::NAN), 0.0);
    assert_eq!(limit(f32::INFINITY), 0.0);
    let curve: Vec<f32> = (0..4000).map(|i| limit(i as f32 * 0.01)).collect();
    assert!(curve.windows(2).all(|pair| pair[1] >= pair[0]));
    assert!(curve.iter().all(|&x| x <= CEILING));
    assert!((limit(1e9) - CEILING).abs() < 1e-6 && (limit(-1e9) + CEILING).abs() < 1e-6);
}

/// The cheap sine stays within a thousandth of the real one, and the triangle is in step with it.
#[test]
fn oscillators_are_accurate() {
    for i in 0..1000 {
        let turn = i as f32 / 1000.0;
        assert!((sine(turn) - (turn * std::f32::consts::TAU).sin()).abs() < 1.2e-3);
        assert!(triangle(turn).abs() <= 1.0);
        assert!(triangle(turn) * sine(turn) >= -1e-6);
    }
}

/// A constant input leaves the offset filter as silence, exactly.
#[test]
fn offset_is_removed() {
    let mut block = DcBlock::default();
    let out: Vec<f32> = (0..96_000).map(|_| block.run(0.5, 0.998)).collect();
    assert_eq!(out[0], 0.5);
    assert_eq!(out[95_999], 0.0);
}

/// A settling value lands exactly on its target, and a plain lag approaches it.
#[test]
fn smoothing_reaches_its_target() {
    let (mut settled, mut plain) = (Lag(0.0), Lag(0.0));
    let k = glide(0.04, RATE);
    for _ in 0..48_000 {
        settled.settle(1.0, k);
        plain.to(1.0, k);
    }
    assert_eq!(settled.0, 1.0);
    assert!((plain.0 - 1.0).abs() < 1e-3);
    assert!((note_hz(69.0) - 440.0).abs() < 1e-3 && (note_hz(57.0) - 220.0).abs() < 1e-3);
}

/// The filter stays stable and bounded on full-scale noise at any cutoff.
#[test]
fn filter_is_stable() {
    for hz in [20.0, 200.0, 2000.0, 12_000.0, 40_000.0] {
        let (mut filter, mut noise) = (Svf::default(), Noise::new(7));
        filter.tune(hz, 1.4, RATE);
        let peak = (0..48_000).fold(0.0_f32, |peak, _| {
            let (low, band) = filter.run(noise.tick());
            peak.max(low.abs()).max(band.abs())
        });
        assert!(peak.is_finite() && peak < 4.0, "{hz} Hz peak {peak}");
    }
}

/// The reverb rings after a click, dies away, and never blows up at its longest tail.
#[test]
fn reverb_decays() {
    let mut reverb = Reverb::new(RATE);
    reverb.shape(1.8, 9.0, 0.3);
    let mut energy = |frames: usize, click: f32| {
        let run = |i: usize| reverb.run(if i == 0 { click } else { 0.0 }, 0.0);
        (0..frames)
            .map(run)
            .fold(0.0, |sum, out| sum + out[0] * out[0] + out[1] * out[1])
    };
    let (early, late) = (energy(96_000, 1.0), energy(96_000, 0.0));
    assert!(early > 0.0 && early.is_finite());
    assert!(late < early && late > 0.0);
    for _ in 0..12 {
        energy(96_000, 0.0);
    }
    assert!(energy(48_000, 0.0) < 1e-4 * early);
}

/// The seeded generator repeats itself and stays inside its ranges.
#[test]
fn random_numbers_are_deterministic() {
    let (mut a, mut b) = (Rand::new(9, 1), Rand::new(9, 1));
    for _ in 0..1000 {
        let (x, y) = (a.range(2.0, 3.0), b.range(2.0, 3.0));
        assert!(x == y && (2.0..3.0).contains(&x));
        assert!(a.below(7) < 7 && b.below(7) < 7);
    }
    assert!(Rand::new(9, 1).bits() != Rand::new(9, 2).bits());
}
