//! Tests of the cabin: its first reflections come within 2 to 6 milliseconds, its tail dies within
//! its decay time, and the seat is quieter than the chase view by the stated amount.

use super::room::{Room, Seat};
use super::tune::{COMB, SEAT};
use crate::audio::dsp::rand::Noise;

/// Sample rate of the tests.
const RATE: f32 = 48_000.0;

/// The impulse response of the cabin: nothing before 2 ms, a first reflection by 6 ms, and 60 dB
/// under its peak by 0.2 seconds, the long end of a cabin's decay; it is tuned to 0.15.
#[test]
fn the_cabin_rings_briefly() {
    let mut room = Room::new(RATE);
    let heard: Vec<f32> = (0..(0.4 * RATE) as usize)
        .map(|i| {
            let [left, right] = room.frame(if i == 0 { 1.0 } else { 0.0 });
            left.abs().max(right.abs())
        })
        .collect();
    let first = heard.iter().position(|x| *x > 1.0e-4).expect("it reflects");
    let ms = first as f32 / RATE * 1000.0;
    assert!((2.0..=6.0).contains(&ms), "first reflection at {ms} ms");
    let peak = heard.iter().fold(0.0_f32, |p, x| p.max(*x));
    let after = &heard[(0.2 * RATE) as usize..];
    let rest = after.iter().fold(0.0_f32, |p, x| p.max(*x));
    assert!(
        COMB.1 < 0.2 && rest < 1.0e-3 * peak,
        "{rest} after 0.2 s against {peak}"
    );
}

/// For the same sound of the ships, ears in the cabin hear it 4 dB quieter than a chase view,
/// give or take what the reflections give back.
#[test]
fn the_seat_is_quieter_than_the_chase_view() {
    let level = |cabin: f32| {
        let mut seat = Seat::new(RATE);
        let mut noise = Noise::new(4);
        let mut sum = 0.0;
        for i in 0..(2.0 * RATE) as usize {
            let x = 0.1 * noise.tick();
            let [left, _] = seat.frame([x, x], 0.0, cabin);
            sum += if i as f32 > 0.5 * RATE {
                left * left
            } else {
                0.0
            };
        }
        sum.sqrt()
    };
    let drop = 20.0 * (level(1.0) / level(0.0)).log10();
    let stated = 20.0 * SEAT.log10();
    assert!((stated + 4.0).abs() < 0.1, "{stated}");
    assert!(drop < -2.0 && drop > -5.0, "{drop} dB");
}
