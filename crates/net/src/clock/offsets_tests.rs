//! Tests of the offset estimate: accuracy under jitter, the ring of sixteen, bad samples.

use super::{Offsets, RTT_MAX, SAMPLES, Sample};

/// Returns a number from 0 to 1 that changes with `n`, the same on every run.
fn noise(n: u32) -> f64 {
    let mixed = n.wrapping_mul(0x9e37_79b9) ^ (n >> 3).wrapping_mul(0x85eb_ca6b);
    f64::from(mixed.wrapping_mul(0xc2b2_ae35) >> 8) / f64::from(1u32 << 24)
}

/// Returns the sample of a ping sent at `sent` that took `up` seconds out and `down` back, to a
/// server whose clock is `offset` ahead of the local one.
fn ping(sent: f64, up: f64, down: f64, offset: f64) -> Sample {
    Sample {
        sent,
        received: sent + up + down,
        server: sent + up + offset,
    }
}

/// On paths of 50 to 150 ms with up to 20 ms of queueing out and 10 ms back, the estimate is
/// within 5 ms of the true offset after 32 pings.
#[test]
fn the_estimate_is_within_five_milliseconds_after_32_pings() {
    for run in 0..40u32 {
        let path = 0.05 + 0.1 * noise(run * 977);
        let truth = 3.0e7 + f64::from(run) * 1234.5;
        let mut offsets = Offsets::new();
        for k in 0..32u32 {
            let n = run * 1000 + k * 7;
            let up = path / 2.0 + 0.02 * noise(n + 1);
            let down = path / 2.0 + 0.01 * noise(n + 2);
            offsets.push(ping(100.0 + f64::from(k), up, down, truth));
        }
        let error = (offsets.offset().unwrap() - truth).abs();
        assert!(error < 0.005, "run {run}: off by {error} s");
    }
}

/// The ring keeps the last sixteen samples: a fast sample is forgotten after sixteen slow ones.
#[test]
fn the_ring_keeps_sixteen_samples() {
    let mut offsets = Offsets::new();
    assert!(offsets.is_empty());
    offsets.push(ping(0.0, 0.005, 0.005, 50.0));
    for k in 1..SAMPLES {
        offsets.push(ping(k as f64, 0.05, 0.05, 50.0));
    }
    assert_eq!(offsets.len(), SAMPLES);
    assert!((offsets.rtt_min().unwrap() - 0.01).abs() < 1e-9);
    offsets.push(ping(99.0, 0.05, 0.05, 50.0));
    assert_eq!(offsets.len(), SAMPLES);
    assert!((offsets.rtt_min().unwrap() - 0.1).abs() < 1e-9);
}

/// Negative, absurd and broken round trips are thrown away.
#[test]
fn bad_samples_are_thrown_away() {
    let mut offsets = Offsets::new();
    assert!(!offsets.push(ping(10.0, -0.2, 0.1, 5.0)));
    assert!(!offsets.push(ping(10.0, RTT_MAX, 0.1, 5.0)));
    assert!(!offsets.push(ping(f64::NAN, 0.05, 0.05, 5.0)));
    assert!(!offsets.push(ping(10.0, 0.05, 0.05, f64::INFINITY)));
    assert!(offsets.is_empty());
    assert_eq!(offsets.offset(), None);
    assert!(offsets.push(ping(10.0, 0.05, 0.05, 5.0)));
    assert!((offsets.offset().unwrap() - 5.0).abs() < 1e-9);
}

/// The smoothed round trip starts at the first one and moves an eighth of the way each time.
#[test]
fn the_smoothed_round_trip_follows_slowly() {
    let mut offsets = Offsets::new();
    assert_eq!(offsets.rtt_smoothed(), None);
    offsets.push(ping(0.0, 0.04, 0.04, 1.0));
    assert!((offsets.rtt_smoothed().unwrap() - 0.08).abs() < 1e-12);
    offsets.push(ping(1.0, 0.08, 0.08, 1.0));
    assert!((offsets.rtt_smoothed().unwrap() - 0.09).abs() < 1e-12);
    assert!((offsets.rtt_min().unwrap() - 0.08).abs() < 1e-12);
}
