//! Tests of millimetre positions: the half-millimetre error, the range and the clamping.

use glam::DVec3;

use super::{MM_RANGE, pack_mm, unpack_mm};

/// Positions all over a planet come back within half a millimetre on each axis.
#[test]
fn a_position_comes_back_within_half_a_millimetre() {
    for n in 0..10_000 {
        let t = f64::from(n);
        let position = DVec3::new(
            (t * 0.731).sin() * 2.0e6,
            (t * 1.913).cos() * 1.3e6,
            (t * 0.277).sin() * 4.2e5 + 0.000_49,
        );
        let error = (unpack_mm(pack_mm(position)) - position)
            .abs()
            .max_element();
        assert!(error <= 0.000_500_001, "error {error} m at {position}");
    }
}

/// The range is about 2147 km, and a coordinate beyond it is clamped to it.
#[test]
fn a_far_coordinate_is_clamped_to_the_range() {
    assert!((MM_RANGE - 2_147_483.647).abs() < 1e-6);
    let far = unpack_mm(pack_mm(DVec3::new(9.0e9, -9.0e9, f64::INFINITY)));
    assert!((far.x - MM_RANGE).abs() < 1e-6);
    assert!((far.y + MM_RANGE).abs() < 0.002);
    assert!((far.z - MM_RANGE).abs() < 1e-6);
}

/// A coordinate that is not a number packs as zero.
#[test]
fn a_missing_number_packs_as_zero() {
    assert_eq!(pack_mm(DVec3::new(f64::NAN, 1.0, -0.0015)), [0, 1000, -2]);
}
