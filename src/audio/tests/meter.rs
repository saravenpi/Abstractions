//! A loudness meter after ITU-R BS.1770 and EBU R 128, written out by hand: the K-weighting, the
//! gated integrated loudness, the momentary and the short-term loudness, the loudness range of
//! EBU Tech 3342 and a true peak from four times oversampling. It measures rendered buffers;
//! the game does not ship it.

use std::f64::consts::PI;

/// Frames in a tenth of a second at the synthesiser rate: the step of every sliding window.
const HOP: usize = 4800;

/// The two stages of the K-weighting at 48 kHz, as BS.1770 gives them: `[b0, b1, b2, a1, a2]`.
/// The first is the shelf of a head, the second the high-pass that models how lows are heard.
const K_WEIGHTING: [[f64; 5]; 2] = [
    [
        1.53512485958697,
        -2.69169618940638,
        1.19839281085285,
        -1.69065929318241,
        0.73248077421585,
    ],
    [1.0, -2.0, 1.0, -1.99004745483398, 0.99007225036621],
];

/// Loudness in LUFS under which a block is silence and counts for nothing.
const SILENCE: f64 = -70.0;

/// Samples either side of a point the true-peak interpolation reads.
const REACH: isize = 6;

/// What the meter reads of one buffer. Loudness is in LUFS, the range in LU, the peak in dBTP.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Loudness {
    /// Gated loudness of the whole buffer.
    pub integrated: f64,
    /// Loudest 3 seconds.
    pub short_term: f64,
    /// Loudest 0.4 seconds.
    pub momentary: f64,
    /// Highest level of the signal between and at its samples.
    pub true_peak: f64,
    /// Spread between the quiet and the loud 3 second stretches.
    pub range: f64,
}

/// Returns the loudness in LUFS of a K-weighted power summed over the channels.
fn lufs(power: f64) -> f64 {
    -0.691 + 10.0 * power.log10()
}

/// Returns the K-weighted power of each tenth of a second of `sound`, summed over both channels.
fn powers(sound: &[[f32; 2]]) -> Vec<f64> {
    let mut state = [[[0.0_f64; 2]; 2]; 2];
    let mut weigh = |side: usize, sample: f32| {
        K_WEIGHTING.iter().zip(&mut state[side]).fold(
            f64::from(sample),
            |x, ([b0, b1, b2, a1, a2], z)| {
                let y = b0 * x + z[0];
                *z = [b1 * x - a1 * y + z[1], b2 * x - a2 * y];
                y
            },
        )
    };
    let mut hop = |frames: &[[f32; 2]; HOP]| {
        let energy: f64 = frames
            .iter()
            .map(|frame| weigh(0, frame[0]).powi(2) + weigh(1, frame[1]).powi(2))
            .sum();
        energy / HOP as f64
    };
    sound.as_chunks::<HOP>().0.iter().map(&mut hop).collect()
}

/// Returns the mean power of every window of `tenths` tenths of a second, a tenth apart.
fn windows(powers: &[f64], tenths: usize) -> Vec<f64> {
    let mean = |window: &[f64]| window.iter().sum::<f64>() / tenths as f64;
    powers.windows(tenths).map(mean).collect()
}

/// Returns the blocks that pass the two gates: those over [`SILENCE`], then of those the ones
/// no more than `below` LU under their mean.
fn gated(blocks: &[f64], below: f64) -> Vec<f64> {
    let heard: Vec<f64> = blocks
        .iter()
        .copied()
        .filter(|power| lufs(*power) > SILENCE)
        .collect();
    let mean = heard.iter().sum::<f64>() / heard.len().max(1) as f64;
    let floor = lufs(mean) - below;
    heard
        .into_iter()
        .filter(|power| lufs(*power) > floor)
        .collect()
}

/// Returns the loudness range in LU: between the tenth and the ninety-fifth percentile of the
/// short-term loudness, gated 20 LU under its mean.
fn range(short: &[f64]) -> f64 {
    let mut kept: Vec<f64> = gated(short, 20.0).into_iter().map(lufs).collect();
    kept.sort_by(f64::total_cmp);
    let Some(last) = kept.len().checked_sub(1) else {
        return 0.0;
    };
    let at = |share: f64| kept[(share * last as f64).round() as usize];
    at(0.95) - at(0.10)
}

/// Returns the true peak of `sound` in dBTP: the highest level of the samples and of three
/// points between each pair of them, read with a windowed sinc.
fn true_peak(sound: &[[f32; 2]]) -> f64 {
    let tap = |t: f64| match t.abs() < 1.0e-9 {
        true => 1.0,
        false => (PI * t).sin() / (PI * t) * 0.5 * (1.0 + (PI * t / REACH as f64).cos()),
    };
    let taps: Vec<[f64; 3]> = (1 - REACH..=REACH)
        .map(|k| [0.25, 0.5, 0.75].map(|part| tap(k as f64 - part)))
        .collect();
    let between = |i: usize, side: usize| {
        let mut sums = [0.0_f64; 3];
        for (k, tap) in (1 - REACH..=REACH).zip(&taps) {
            let x = f64::from(sound[(i as isize + k) as usize][side]);
            sums = [0, 1, 2].map(|part| sums[part] + x * tap[part]);
        }
        sums.into_iter().fold(0.0, |peak, y| y.abs().max(peak))
    };
    let inner = REACH as usize..sound.len().saturating_sub(REACH as usize);
    let hidden = inner
        .flat_map(|i| [between(i, 0), between(i, 1)])
        .fold(0.0, f64::max);
    let plain = sound
        .iter()
        .flatten()
        .fold(0.0, |p, x| f64::from(x.abs()).max(p));
    20.0 * hidden.max(plain).log10()
}

/// Measures `sound`, a stereo buffer at the synthesiser rate.
pub(super) fn measure(sound: &[[f32; 2]]) -> Loudness {
    let powers = powers(sound);
    let (blocks, short) = (windows(&powers, 4), windows(&powers, 30));
    let kept = gated(&blocks, 10.0);
    let loudest = |powers: &[f64]| lufs(powers.iter().copied().fold(0.0, f64::max));
    Loudness {
        integrated: lufs(kept.iter().sum::<f64>() / kept.len().max(1) as f64),
        short_term: loudest(&short),
        momentary: loudest(&blocks),
        true_peak: true_peak(sound),
        range: range(&short),
    }
}
