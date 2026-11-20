//! Tests of cell positions: the half-millimetre error at planet scale and bad numbers.

use glam::DVec3;

use super::{CELL, pack_cells, unpack_cells};

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
        let error = (unpack_cells(pack_cells(position)) - position)
            .abs()
            .max_element();
        assert!(error <= 0.000_500_001, "error {error} m at {position}");
    }
}

/// A position at 1.2e7 m, the radius of the largest rocky planet, and beyond it, keeps a
/// millimetre.
#[test]
fn a_position_at_planet_scale_keeps_a_millimetre() {
    for n in 0..10_000 {
        let t = f64::from(n);
        let position = DVec3::new(
            1.2e7 + (t * 0.37).sin() * 3.0e5 + 0.000_123,
            -1.2e7 * (t * 0.011).cos() + 0.000_7,
            7.0e7 * (t * 0.019).sin() + 0.000_3,
        );
        let packed = pack_cells(position);
        assert!(
            packed
                .mm
                .iter()
                .all(|mm| f64::from(mm.abs()) <= CELL * 500.0)
        );
        let error = (unpack_cells(packed) - position).abs().max_element();
        assert!(error <= 0.001, "error {error} m at {position}");
    }
}

/// A coordinate that is not a number packs as zero.
#[test]
fn a_missing_number_packs_as_zero() {
    let packed = pack_cells(DVec3::new(f64::NAN, 1.0, -0.0015));
    assert_eq!((packed.cell, packed.mm), ([0, 0, 0], [0, 1000, -2]));
}
