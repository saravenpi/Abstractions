//! Tests of the ears: a sound on the right is louder and earlier in the right ear, a flat sound is
//! the same in both, a sound behind loses some highs, and the body comes through on its own.

use bevy::prelude::*;

use super::ear::Ear;
use super::listen::Listener;
use super::spot::{Between, Origin, Spot, spot};
use crate::audio::dsp::rand::Noise;

/// Sample rate of the tests.
const RATE: f32 = 48_000.0;

/// Returns what the ears hear of one click through `spot`, sample by sample.
fn click(spot: &Spot) -> Vec<[f32; 2]> {
    let mut ear = Ear::new(1.0);
    ear.place(spot, RATE);
    (0..256)
        .map(|i| {
            let x = if i == 0 { 1.0 } else { 0.0 };
            ear.frame(x, x)
        })
        .collect()
}

/// Returns the energy of each ear and the sample each first hears anything, for `heard`.
fn read(heard: &[[f32; 2]]) -> ([f32; 2], [usize; 2]) {
    let energy = [0, 1].map(|i| heard.iter().map(|frame| frame[i] * frame[i]).sum::<f32>());
    let first = [0, 1].map(|i| {
        heard
            .iter()
            .position(|frame| frame[i].abs() > 1.0e-4)
            .unwrap_or(usize::MAX)
    });
    (energy, first)
}

/// Returns the spot of a source 10 metres away along `toward`, for a listener facing ahead.
fn toward(toward: Vec3) -> Spot {
    spot(
        &Listener::default(),
        Origin::At(toward * 10.0, 10.0),
        Between::default(),
    )
}

/// A source on the right is louder in the right ear and reaches it first, and one on the left
/// the other way round.
#[test]
fn a_source_on_the_right_is_louder_and_earlier_on_the_right() {
    let (energy, first) = read(&click(&toward(Vec3::X)));
    assert!(energy[1] > 3.0 * energy[0], "{energy:?}");
    assert!(first[1] + 20 < first[0], "{first:?}");
    let (energy, first) = read(&click(&toward(Vec3::NEG_X)));
    assert!(energy[0] > 3.0 * energy[1] && first[0] + 20 < first[1]);
}

/// A flat sound and a sound straight ahead come to both ears alike, at their level.
#[test]
fn flat_sounds_stay_centred() {
    for spot in [Spot::FLAT, toward(Vec3::NEG_Z)] {
        let heard = click(&spot);
        assert!(
            heard
                .iter()
                .all(|frame| (frame[0] - frame[1]).abs() < 1.0e-6)
        );
    }
    let [left, right] = click(&Spot::FLAT)[0];
    assert!((left - 1.0).abs() < 1.0e-5 && (right - 1.0).abs() < 1.0e-5);
}

/// A sound from behind loses some of its highs against the same sound ahead, and keeps its lows.
#[test]
fn a_sound_behind_is_duller() {
    let level = |spot: Spot, bright: bool| {
        let mut ear = Ear::new(1.0);
        ear.place(&spot, RATE);
        let mut noise = Noise::new(9);
        let (mut low, mut sum) = (0.0, 0.0);
        for _ in 0..48_000 {
            let x = noise.tick();
            low += 0.04 * (x - low);
            let fed = if bright { x - low } else { low };
            sum += ear.frame(fed, 0.0)[0].powi(2);
        }
        sum
    };
    let (front, back) = (toward(Vec3::NEG_Z), toward(Vec3::Z));
    assert!(level(back, true) < 0.85 * level(front, true));
    assert!(level(back, false) > 0.9 * level(front, false));
}

/// What the body carries is heard in both ears whatever the air.
#[test]
fn the_body_comes_through_alone() {
    let held = Spot {
        air: 0.0,
        body: 0.5,
        ..Spot::FLAT
    };
    let [left, right] = click(&held)[0];
    assert!((left - 0.5).abs() < 1.0e-6 && (right - 0.5).abs() < 1.0e-6);
}
