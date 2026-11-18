//! Tests of the jitter buffer over simulated networks: accuracy on a circle with loss and
//! jitter, a body at rest, and how the delay adapts.

use glam::{DVec3, Quat, Vec3};

use super::{Buffer, DELAY_MAX, DELAY_MIN, DRIFT, Snap, View};

/// Seconds between two snapshots of a moving body.
const INTERVAL: f64 = 1.0 / 30.0;

/// Seconds between two snapshots of a body at rest.
const REST_INTERVAL: f64 = 0.5;

/// Shortest one-way trip of a datagram in seconds.
const LATENCY: f64 = 0.05;

/// Frames drawn per second.
const FPS: f64 = 60.0;

/// Returns a number from 0 to 1 that changes with `n`, the same on every run.
fn noise(n: u32) -> f64 {
    let mixed = n.wrapping_mul(0x9e37_79b9) ^ (n >> 3).wrapping_mul(0x85eb_ca6b);
    f64::from(mixed.wrapping_mul(0xc2b2_ae35) >> 8) / f64::from(1u32 << 24)
}

/// Returns the true state at `time` of a body going round a 20 m circle at 10 m/s.
fn circle(time: f64) -> Snap<()> {
    let angle = time * 10.0 / 20.0;
    let (sin, cos) = angle.sin_cos();
    Snap {
        time,
        position: DVec3::new(20.0 * cos, 0.0, 20.0 * sin),
        velocity: Some(Vec3::new((-10.0 * sin) as f32, 0.0, (10.0 * cos) as f32)),
        rotation: Quat::from_rotation_y(-angle as f32),
        extra: (),
    }
}

/// Returns the snapshots of `body` sent every `interval` for `seconds`, with the share `loss`
/// lost and up to `jitter` seconds of extra latency, paired with their arrival times in order.
fn arrivals(
    body: fn(f64) -> Snap<()>,
    interval: f64,
    jitter: f64,
    loss: f64,
    seconds: f64,
) -> Vec<(f64, Snap<()>)> {
    let count = (seconds / interval) as u32;
    let mut list: Vec<(f64, Snap<()>)> = (0..count)
        .filter(|&k| noise(k * 3) >= loss)
        .map(|k| (body(f64::from(k) * interval), noise(k * 3 + 1)))
        .map(|(snap, draw)| (snap.time + LATENCY + jitter * draw, snap))
        .collect();
    list.sort_by(|a, b| a.0.total_cmp(&b.0));
    list
}

/// Feeds `buffer` the `arrivals` and samples it every frame for `seconds`, calling `check` with
/// each state drawn, the frame time and the delay in use.
fn play(
    buffer: &mut Buffer<()>,
    arrivals: &[(f64, Snap<()>)],
    seconds: f64,
    mut check: impl FnMut(&View<()>, f64, f64),
) {
    let mut next = 0;
    for frame in 0..(seconds * FPS) as u32 {
        let now = f64::from(frame) / FPS;
        while let Some(&(arrival, snap)) = arrivals.get(next).filter(|(arrival, _)| *arrival <= now)
        {
            buffer.push(snap, arrival);
            next += 1;
        }
        if let Some(view) = buffer.sample(now) {
            check(&view, now, buffer.delay());
        }
    }
}

/// On a circle at 30 Hz with 5 percent loss and up to 20 ms of jitter, every state drawn is
/// within 2 cm of the true path. Rotations are not extrapolated, so the heading may lag by the
/// turn rate of 0.5 rad/s over at most 100 ms.
#[test]
fn a_circle_is_drawn_within_two_centimetres() {
    let mut buffer = Buffer::new(INTERVAL);
    let list = arrivals(circle, INTERVAL, 0.02, 0.05, 30.0);
    let mut worst: f64 = 0.0;
    play(&mut buffer, &list, 30.0, |view, now, _| {
        if now > 1.0 {
            worst = worst.max(view.position.distance(circle(view.time).position));
            let turn = view.rotation.angle_between(circle(view.time).rotation);
            assert!(turn < 0.06, "turn {turn} rad at {now}");
        }
    });
    assert!(worst < 0.02, "worst error {worst} m");
}

/// Returns a body at rest, sampled at `time`.
fn still(time: f64) -> Snap<()> {
    Snap {
        time,
        position: DVec3::new(3.0, 4.0, 5.0),
        velocity: Some(Vec3::ZERO),
        rotation: Quat::from_rotation_x(0.3),
        extra: (),
    }
}

/// A body at rest sending twice a second holds perfectly still and does not raise the delay.
#[test]
fn a_body_at_rest_holds_still() {
    let mut buffer = Buffer::new(INTERVAL);
    let start = buffer.delay();
    let list = arrivals(still, REST_INTERVAL, 0.02, 0.0, 20.0);
    play(&mut buffer, &list, 20.0, |view, _, delay| {
        assert_eq!(view.position, still(0.0).position);
        assert_eq!(view.rotation, still(0.0).rotation);
        assert!(delay <= start + 0.02, "delay {delay}");
    });
}

/// Under 40 ms of jitter the delay stays within its clamp, changes at most [`DRIFT`] per
/// second, and grows above two send intervals.
#[test]
fn the_delay_adapts_upward_under_jitter() {
    let mut buffer = Buffer::new(INTERVAL);
    let start = buffer.delay();
    assert!((start - 2.0 * INTERVAL).abs() < 1e-12);
    let list = arrivals(circle, INTERVAL, 0.04, 0.05, 30.0);
    let mut last = start;
    play(&mut buffer, &list, 30.0, |_, _, delay| {
        assert!((DELAY_MIN..=DELAY_MAX).contains(&delay));
        assert!((delay - last).abs() <= DRIFT / FPS + 1e-12);
        last = delay;
    });
    assert!(last > start + 0.02, "delay {last} from {start}");
}
