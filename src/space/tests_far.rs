//! Far positions: a galaxy holds 2.8e22 m, and a metre or a light-year must survive in it.

use bevy::math::{DVec3, IVec3};

use super::{SECTORS_PER_CELL, UniPos};
use crate::universe::{GALAXY_CELL, LY, SECTOR_M};

/// Returns the next value of a splitmix64 stream, from 0 to 1.
fn unit(state: &mut u64) -> f64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
}

/// Returns three values from 0 to 1.
fn cube(state: &mut u64) -> DVec3 {
    DVec3::new(unit(state), unit(state), unit(state))
}

/// Returns a point about 3e20 m from the centre, the sector number 3171 on the first axis.
fn far(local: DVec3) -> UniPos {
    UniPos::sectors(IVec3::ZERO, IVec3::new(3171, 40, -9), local)
}

/// Two points 1,000 m apart and 3e20 m from the centre stay 1,000 m apart.
#[test]
fn a_kilometre_survives_at_3e20_metres() {
    let a = far(DVec3::new(1.0e15, 2.0e16, 3.0e15));
    let b = a + DVec3::new(1000.0, 0.0, 0.0);
    assert!(((a - b).length() - 1000.0).abs() < 20.0);
    assert!(((b - a).length() - 1000.0).abs() < 20.0);
    let c = far(DVec3::new(6.0e16, 5.0e16, 4.0e16));
    let d = c + DVec3::new(0.0, 0.0, -1000.0);
    assert!(((c - d).length() - 1000.0).abs() < 20.0);
}

/// A star 4 light-years away is 4 light-years away, to a part in 1e12.
#[test]
fn four_light_years_are_exact_to_a_part_in_1e12() {
    let a = far(DVec3::new(2.0e16, 3.0e16, 1.0e16));
    let step = DVec3::new(1.0, 2.0, 2.0) / 3.0 * 4.0 * LY;
    let b = a + step;
    let got = (b - a).length();
    assert!((got - 4.0 * LY).abs() / (4.0 * LY) < 1e-12, "{got}");
    assert!((a.distance(b) - got).abs() < 1.0);
}

/// Positions built from integers keep their parts in range and differences stay within the
/// step of one 64-bit float across galaxies.
#[test]
fn random_positions_round_trip_across_galaxies() {
    let mut state = 7;
    for _ in 0..1000 {
        let (p, q) = (
            cube(&mut state) * GALAXY_CELL,
            cube(&mut state) * GALAXY_CELL,
        );
        let cells = IVec3::new((unit(&mut state) * 2.0) as i32, 0, 1);
        let (a, b) = (UniPos::new(cells, p), UniPos::new(IVec3::ZERO, q));
        let wanted = cells.as_dvec3() * GALAXY_CELL + (p - q);
        let got = a - b;
        let slack = 1e7 + 1e-15 * wanted.abs().max_element();
        assert!((got - wanted).abs().max_element() < slack, "{got} {wanted}");
        assert!(a.sector.min_element() >= 0 && a.sector.max_element() < SECTORS_PER_CELL);
        assert!(a.local.min_element() >= 0.0 && a.local.max_element() <= SECTOR_M);
    }
}

/// A metre next to a point 5e20 m from the centre stays a metre, while the point lies in the
/// first 9e15 m of its sector, where a 64-bit float steps by less than a metre.
#[test]
fn a_parked_ship_keeps_its_metre_at_5e20_metres() {
    let point = UniPos::sectors(
        IVec3::new(2, -1, 0),
        IVec3::new(5285, 7, 120),
        DVec3::splat(1.0e15),
    );
    for step in [DVec3::X, DVec3::NEG_Y, DVec3::new(0.0, 0.5, 0.75)] {
        let ship = point + step;
        assert_eq!(ship - point, step);
        assert_eq!(point - ship, -step);
        assert_eq!((ship - step) - point, DVec3::ZERO);
    }
}

/// Deep in a sector the step of the local metres is 16 m, and no more.
#[test]
fn the_step_deep_in_a_sector_is_sixteen_metres() {
    let point = far(DVec3::splat(9.0e16));
    let ship = point + DVec3::X;
    assert!(((ship - point).x - 1.0).abs() <= 16.0);
}
