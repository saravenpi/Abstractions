//! Tests of the batched creature pose codec: round trips, budget fit, and malformed inputs.

use glam::{DVec3, Vec3};

use super::body_tests::planet;
use super::{
    FAUNA_BATCH_MAX, FAUNA_BODY_MAX, FAUNA_HEADER_LEN, FAUNA_POSE_LEN, FaunaPose, Frame, Head,
    Kind, Record, decode_fauna, encode_fauna, max_fauna_poses, push,
};
use crate::host::DATAGRAM_BUDGET;
use crate::log::PlayerId;

/// Returns a sample creature pose with non-zero velocity, off-axis heading and active stance.
fn sample_pose(herd: u64, slot: u8, pos: DVec3) -> FaunaPose {
    FaunaPose {
        herd,
        slot,
        species: 7,
        pos,
        vel: Vec3::new(4.25, -1.5, 9.75),
        heading: Vec3::new(0.6, 0.2, -0.8).normalize(),
        stance: 4,
        excite: 0.85,
    }
}

/// Walker and flyer poses round-trip within sub-millimetre relative position and cm/s velocity.
#[test]
fn fauna_poses_round_trip() {
    let origin = DVec3::new(4_200_000.5, -1_100_000.25, 2_900_000.75);
    let poses = vec![
        sample_pose(11, 2, origin + DVec3::new(12.345, -3.21, 98.765)),
        FaunaPose {
            species: 19,
            vel: Vec3::new(-14.2, 6.4, -3.1),
            heading: Vec3::new(-0.3, 0.7, 0.65).normalize(),
            stance: 2,
            excite: 0.25,
            ..sample_pose(99, 5, origin + DVec3::new(-140.0, 22.5, 45.0))
        },
    ];
    let mut out = Vec::new();
    encode_fauna(origin, &poses, &mut out);
    assert_eq!(out.len(), FAUNA_HEADER_LEN + 2 * FAUNA_POSE_LEN);
    let batch = decode_fauna(&out).unwrap();
    assert_eq!(batch.origin, origin);
    assert_eq!(batch.poses.len(), 2);
    for (got, want) in batch.poses.iter().zip(&poses) {
        assert_eq!(
            (got.herd, got.slot, got.species),
            (want.herd, want.slot, want.species)
        );
        assert!((got.pos - want.pos).length() < 1e-3);
        assert!((got.vel - want.vel).abs().max_element() <= 0.011);
        assert!(got.heading.angle_between(want.heading) < 0.025);
        assert_eq!(got.stance, want.stance);
        assert!((got.excite - want.excite).abs() <= 1.0 / 31.0);
    }
}

/// An empty batch and a maximum batch both round-trip, and a full record fits `DATAGRAM_BUDGET`.
#[test]
fn max_batch_fits_datagram_budget() {
    let origin = DVec3::new(6_371_000.0, 0.0, 0.0);
    let mut empty = Vec::new();
    encode_fauna(origin, &[], &mut empty);
    assert_eq!(decode_fauna(&empty).unwrap().poses.len(), 0);

    let full: Vec<_> = (0..FAUNA_BATCH_MAX + 5)
        .map(|i| sample_pose(100 + i as u64, (i % 12) as u8, origin + DVec3::X * i as f64))
        .collect();
    let mut body = Vec::new();
    encode_fauna(origin, &full, &mut body);
    assert!(body.len() <= FAUNA_BODY_MAX);
    assert_eq!(decode_fauna(&body).unwrap().poses.len(), FAUNA_BATCH_MAX);
    assert_eq!(max_fauna_poses(DATAGRAM_BUDGET), FAUNA_BATCH_MAX);

    let head = Head {
        player: PlayerId(u32::MAX),
        kind: Kind::Fauna,
        frame: Frame::Surface(planet()),
        time: 98_765.432_1,
        flags: 0,
    };
    let mut datagram = Vec::new();
    push(&mut datagram, &Record { head, body: &body });
    assert!(
        datagram.len() <= DATAGRAM_BUDGET,
        "datagram {} > {}",
        datagram.len(),
        DATAGRAM_BUDGET
    );
}

/// Truncated, overlong, bad-count and non-finite fauna batches are rejected.
#[test]
fn bad_fauna_batches_are_refused() {
    let origin = DVec3::new(1000.0, 2000.0, 3000.0);
    let mut bytes = Vec::new();
    encode_fauna(origin, &[sample_pose(1, 0, origin)], &mut bytes);
    assert!(decode_fauna(&bytes[..bytes.len() - 1]).is_none());
    let mut over = bytes.clone();
    over.push(0);
    assert!(decode_fauna(&over).is_none());

    let mut bad_count = bytes.clone();
    bad_count[FAUNA_HEADER_LEN - 1] = (FAUNA_BATCH_MAX + 1) as u8;
    assert!(decode_fauna(&bad_count).is_none());

    let mut nan_origin = bytes.clone();
    nan_origin[0..8].copy_from_slice(&f64::NAN.to_le_bytes());
    assert!(decode_fauna(&nan_origin).is_none());

    let mut nan_pose = bytes;
    let rel_start = FAUNA_HEADER_LEN + 11;
    nan_pose[rel_start..rel_start + 4].copy_from_slice(&f32::INFINITY.to_le_bytes());
    assert!(decode_fauna(&nan_pose).is_none());
}
