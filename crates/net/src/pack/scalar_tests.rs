//! Tests of the one and two byte numbers: the round-trip error, the ends and the clamping.

use std::f32::consts::PI;

use super::{
    pack_i8, pack_i16, pack_u16, pack_unit, unpack_i8, unpack_i16, unpack_u16, unpack_unit,
};

/// A share comes back within 1/510, and the ends are exact.
#[test]
fn a_share_comes_back() {
    for n in 0..=1000 {
        let share = n as f32 / 1000.0;
        assert!((unpack_unit(pack_unit(share)) - share).abs() <= 1.0 / 510.0 + 1e-6);
    }
    assert_eq!(unpack_unit(pack_unit(0.0)), 0.0);
    assert_eq!(unpack_unit(pack_unit(1.0)), 1.0);
}

/// An angle in 8 bits comes back within pi over 254, and zero and the ends are exact.
#[test]
fn an_angle_in_eight_bits_comes_back() {
    for n in -1000..=1000 {
        let angle = n as f32 / 1000.0 * PI;
        assert!((unpack_i8(pack_i8(angle, PI), PI) - angle).abs() <= PI / 254.0 + 1e-5);
    }
    assert_eq!(unpack_i8(pack_i8(0.0, PI), PI), 0.0);
    assert_eq!(unpack_i8(pack_i8(-PI, PI), PI), -PI);
    assert_eq!(unpack_i8(i8::MIN, PI), -PI);
}

/// An angle in 16 bits comes back within pi over 65534.
#[test]
fn an_angle_in_sixteen_bits_comes_back() {
    for n in -1000..=1000 {
        let angle = n as f32 / 1000.0 * PI;
        assert!((unpack_i16(pack_i16(angle, PI), PI) - angle).abs() <= PI / 65534.0 + 1e-5);
    }
    assert_eq!(unpack_i16(i16::MIN, PI), -PI);
}

/// A speed in 16 bits comes back within half a step of its range.
#[test]
fn a_speed_in_sixteen_bits_comes_back() {
    for n in 0..=1000 {
        let speed = n as f32 * 0.655;
        assert!((unpack_u16(pack_u16(speed, 655.35), 655.35) - speed).abs() <= 0.0051);
    }
}

/// Values out of range are clamped to the ends, and a missing number packs as zero.
#[test]
fn out_of_range_values_are_clamped() {
    assert_eq!(pack_unit(7.0), 255);
    assert_eq!(pack_unit(-7.0), 0);
    assert_eq!(pack_i8(9.0, 1.0), 127);
    assert_eq!(pack_i8(-9.0, 1.0), -127);
    assert_eq!(pack_i16(f32::INFINITY, 1.0), 32767);
    assert_eq!(pack_u16(-1.0, 10.0), 0);
    assert_eq!(pack_u16(11.0, 10.0), 65535);
    assert_eq!((pack_unit(f32::NAN), pack_i8(f32::NAN, 1.0)), (0, 0));
}
