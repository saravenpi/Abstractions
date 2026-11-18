//! Tests of the snapshot ring through the buffer: order, capacity, holding, extrapolation,
//! blending without velocity, rotations and reset.

use glam::{DVec3, Quat, Vec3};

use super::{Buffer, CAPACITY, EXTRAPOLATE_MAX, Snap};

/// Returns a snapshot at `time` at `x` metres along the x axis, moving at `speed` along it.
fn at(time: f64, x: f64, speed: Option<f32>) -> Snap<u32> {
    Snap {
        time,
        position: DVec3::new(x, 0.0, 0.0),
        velocity: speed.map(|speed| Vec3::new(speed, 0.0, 0.0)),
        rotation: Quat::IDENTITY,
        extra: time as u32,
    }
}

/// Returns a buffer at its starting delay holding `snaps`.
fn filled(snaps: &[Snap<u32>]) -> Buffer<u32> {
    let mut buffer = Buffer::new(1.0 / 30.0);
    for snap in snaps {
        buffer.push(*snap, snap.time);
    }
    buffer
}

/// Late, repeated and broken snapshots are dropped, and the ring keeps the last [`CAPACITY`].
#[test]
fn late_and_repeated_snapshots_are_dropped() {
    let mut buffer = filled(&[at(1.0, 0.0, None), at(2.0, 1.0, None)]);
    assert!(!buffer.push(at(1.5, 9.0, None), 2.1));
    assert!(!buffer.push(at(2.0, 9.0, None), 2.1));
    assert!(!buffer.push(at(f64::NAN, 9.0, None), 2.1));
    assert_eq!(buffer.len(), 2);
    for k in 3..100 {
        assert!(buffer.push(at(f64::from(k), 0.0, None), f64::from(k)));
    }
    assert_eq!(buffer.len(), CAPACITY);
}

/// Past the newest snapshot the body is carried on its velocity for 100 ms, then holds.
#[test]
fn extrapolation_stops_at_a_hundred_milliseconds() {
    let mut buffer = filled(&[at(1.0, 0.0, Some(2.0)), at(2.0, 2.0, Some(2.0))]);
    let delay = buffer.delay();
    let short = buffer.sample(2.05 + delay).unwrap();
    assert!((short.position.x - 2.1).abs() < 1e-9);
    let long = buffer.sample(5.0 + delay).unwrap();
    assert!((long.position.x - (2.0 + 2.0 * EXTRAPOLATE_MAX)).abs() < 1e-6);
    assert_eq!((long.older, long.newer), (2, 2));
}

/// Before the oldest snapshot the body holds it, and without velocity it blends straight.
#[test]
fn without_velocity_the_blend_is_straight() {
    let mut buffer = filled(&[at(1.0, 0.0, None), at(2.0, 10.0, None)]);
    let delay = buffer.delay();
    let early = buffer.sample(0.5 + delay).unwrap();
    assert_eq!(early.position.x, 0.0);
    let middle = buffer.sample(1.25 + delay).unwrap();
    assert!((middle.position.x - 2.5).abs() < 1e-6);
    assert_eq!((middle.older, middle.newer), (1, 2));
    assert!((middle.blend - 0.25).abs() < 1e-4);
}

/// Rotations take the shortest way, even when the two ends have opposite signs.
#[test]
fn rotations_take_the_shortest_way() {
    let mut start = at(1.0, 0.0, None);
    let mut end = at(2.0, 0.0, None);
    start.rotation = Quat::from_rotation_y(0.1);
    end.rotation = -Quat::from_rotation_y(0.3);
    let mut buffer = filled(&[start, end]);
    let delay = buffer.delay();
    let middle = buffer.sample(1.5 + delay).unwrap();
    assert!(middle.rotation.angle_between(Quat::from_rotation_y(0.2)) < 1e-3);
}

/// After a reset the buffer is empty and takes snapshots older than the ones it forgot.
#[test]
fn reset_forgets_the_snapshots() {
    let mut buffer = filled(&[at(5.0, 0.0, None), at(6.0, 1.0, None)]);
    buffer.reset();
    assert!(buffer.is_empty());
    assert!(buffer.sample(7.0).is_none());
    assert!(buffer.push(at(3.0, 4.0, None), 7.0));
    assert_eq!(buffer.sample(8.0).unwrap().position.x, 4.0);
}
