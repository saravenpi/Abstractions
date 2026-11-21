//! The lift nozzles over the sea: one continuous hiss of spray, with no crackle and no pulse, the
//! kind of sound a hovering jet makes on water rather than a rattle of clicks.

use std::f32::consts::TAU;

use super::rig::{alone, parked};
use super::{frames, mixer};
use crate::audio::ship::feed::Ch;

/// Returns the last five of six seconds of a ship hovering on its lift nozzles 5 metres over the
/// sea, heard from 20 metres, in stereo.
pub(super) fn hover() -> Vec<[f32; 2]> {
    let hull = parked()
        .with(Ch::Lift, 1.0)
        .with(Ch::Height, 5.0)
        .with(Ch::Wet, 1.0)
        .with(Ch::Distance, 20.0)
        .with(Ch::Aspect, 0.0);
    let (mut mixer, _shared) = mixer(&alone(hull));
    let sound: Vec<[f32; 2]> = (0..frames(6.0)).map(|_| mixer.frame()).collect();
    sound[frames(1.0)..].to_vec()
}

/// The spray is noise, not a rattle: its samples have the kurtosis of noise, about 3, where a
/// crackle of shocks would raise it far over that.
#[test]
fn the_spray_does_not_crackle() {
    let left: Vec<f64> = hover().iter().map(|frame| f64::from(frame[0])).collect();
    let mean = left.iter().sum::<f64>() / left.len() as f64;
    let moment =
        |power: i32| left.iter().map(|x| (x - mean).powi(power)).sum::<f64>() / left.len() as f64;
    let kurtosis = moment(4) / moment(2).powi(2);
    assert!(moment(2) > 1.0e-8 && kurtosis < 4.5, "kurtosis {kurtosis}");
}

/// The level of the spray holds no pulse between 5 and 40 Hz: averaged over ten half seconds, no
/// rate of its envelope stands more than three times over the middle of the others.
#[test]
fn the_spray_has_no_rate() {
    let window = frames(0.005);
    let envelope: Vec<f32> = hover()
        .chunks(window)
        .map(|part| (part.iter().map(|f| f[0] * f[0]).sum::<f32>() / part.len() as f32).sqrt())
        .collect();
    let mut power = [0.0_f32; 21];
    for segment in envelope.chunks_exact(100).take(10) {
        let mean = segment.iter().sum::<f32>() / 100.0;
        for (k, bin) in power.iter_mut().enumerate().skip(3) {
            let (re, im) = segment
                .iter()
                .enumerate()
                .fold((0.0, 0.0), |(re, im), (n, x)| {
                    let turn = TAU * (k * n) as f32 / 100.0;
                    (re + (x - mean) * turn.cos(), im + (x - mean) * turn.sin())
                });
            *bin += re * re + im * im;
        }
    }
    let mut rates = power[3..].to_vec();
    rates.sort_by(f32::total_cmp);
    let (middle, top) = (rates[rates.len() / 2], rates[rates.len() - 1]);
    assert!(top < 3.0 * middle, "a pulse in the spray: {power:?}");
}
