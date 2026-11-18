//! Tests of the 32-bit rotation: the error stays small for many rotations and odd inputs survive.

use glam::{Quat, Vec3};

use super::{pack_rotation, unpack_rotation};

/// Returns a number from 0 to 1 that changes with `n`, the same on every run.
fn noise(n: u32) -> f32 {
    let mixed = n.wrapping_mul(0x9e37_79b9) ^ (n >> 3).wrapping_mul(0x85eb_ca6b);
    (mixed.wrapping_mul(0xc2b2_ae35) >> 8) as f32 / (1 << 24) as f32
}

/// Returns a rotation that changes with `n`, covering every axis and angle.
fn sample(n: u32) -> Quat {
    let axis = Vec3::new(
        noise(n) - 0.5,
        noise(n + 7919) - 0.5,
        noise(n + 104_729) - 0.5,
    );
    let angle = (noise(n + 31) - 0.5) * 4.0 * std::f32::consts::PI;
    Quat::from_axis_angle(axis.try_normalize().unwrap_or(Vec3::Y), angle)
}

/// Ten thousand rotations come back within 0.005 rad.
#[test]
fn a_rotation_comes_back_within_five_milliradians() {
    let worst = (0..10_000)
        .map(|n| {
            let rotation = sample(n);
            unpack_rotation(pack_rotation(rotation)).angle_between(rotation)
        })
        .fold(0.0, f32::max);
    assert!(worst < 0.005, "worst error {worst} rad");
}

/// A quaternion and its negative are one rotation and pack to the same bits.
#[test]
fn a_negated_quaternion_packs_the_same() {
    for n in 0..100 {
        let rotation = sample(n);
        assert_eq!(pack_rotation(rotation), pack_rotation(-rotation));
    }
}

/// The turns about each axis, where one component is exactly one, come back exactly.
#[test]
fn the_axis_turns_come_back() {
    let turns = [
        Quat::IDENTITY,
        Quat::from_xyzw(1.0, 0.0, 0.0, 0.0),
        Quat::from_xyzw(0.0, 1.0, 0.0, 0.0),
        Quat::from_xyzw(0.0, 0.0, 1.0, 0.0),
    ];
    for turn in turns {
        assert_eq!(unpack_rotation(pack_rotation(turn)), turn);
    }
}

/// A quaternion of no length or with a missing number packs as no turn, and any bits unpack
/// to a unit rotation.
#[test]
fn odd_inputs_give_unit_rotations() {
    let none = pack_rotation(Quat::IDENTITY);
    assert_eq!(pack_rotation(Quat::from_xyzw(0.0, 0.0, 0.0, 0.0)), none);
    assert_eq!(
        pack_rotation(Quat::from_xyzw(f32::NAN, 0.0, 0.0, 1.0)),
        none
    );
    for bits in [0, u32::MAX, 0x3fff_ffff, 0xdead_beef] {
        assert!(unpack_rotation(bits).is_normalized());
    }
}
