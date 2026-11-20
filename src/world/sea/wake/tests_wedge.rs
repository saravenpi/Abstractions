//! Tests of the angle of the arms of a wake, the wavelength of its waves and the height it raises.

use std::f32::consts::TAU;

use super::packet::Packet;
use super::physics::{
    amplitude, arm_angle, froude, transverse_share, transverse_wavelength, wedge,
};
use super::tune::{ARM_FLOOR, KELVIN, MAX_AMPLITUDE};

/// A slow source leaves the Kelvin wedge of 19.47 degrees.
#[test]
fn a_slow_source_leaves_the_kelvin_wedge() {
    let angle = wedge(0.1, 9.81, 2.0).to_degrees();
    assert!((angle - 19.47).abs() < 0.01, "{angle}");
    assert!((wedge(0.0, 9.81, 2.0) - KELVIN).abs() < 1e-6);
}

/// A faster source leaves narrower arms, following `atan(1 / (2 sqrt(2 pi) Fr))`.
#[test]
fn a_faster_source_leaves_narrower_arms() {
    let angles: Vec<f32> = [4.0, 10.0, 20.0, 40.0, 150.0]
        .iter()
        .map(|speed| wedge(*speed, 9.81, 2.0))
        .collect();
    assert!(
        angles.windows(2).all(|pair| pair[1] < pair[0]),
        "{angles:?}"
    );
    let fr = froude(40.0, 9.81, 2.0);
    let rabaud = (1.0 / (2.0 * (TAU).sqrt() * fr)).atan();
    assert!((wedge(40.0, 9.81, 2.0) - rabaud).abs() < 1e-6);
}

/// A heavier planet makes a wider wedge at the same speed.
#[test]
fn a_heavier_planet_makes_a_wider_wedge() {
    assert!(wedge(40.0, 22.0, 2.0) > wedge(40.0, 9.81, 2.0));
    assert!(wedge(150.0, 22.0, 40.0) > wedge(150.0, 9.81, 40.0));
}

/// The arms drawn are never narrower than the floor, and equal the wedge above it.
#[test]
fn the_arms_keep_a_floor() {
    assert_eq!(arm_angle(150.0, 9.81, 2.0), ARM_FLOOR);
    assert_eq!(arm_angle(0.1, 9.81, 2.0), wedge(0.1, 9.81, 2.0));
}

/// The transverse wavelength is `2 pi v^2 / g`: four times as long at twice the speed, and shorter
/// where gravity is stronger.
#[test]
fn transverse_wavelength_scales_with_speed_squared_over_gravity() {
    let base = transverse_wavelength(10.0, 9.81);
    assert!((base - TAU * 100.0 / 9.81).abs() < 1e-3);
    assert!((transverse_wavelength(20.0, 9.81) / base - 4.0).abs() < 1e-4);
    assert!((transverse_wavelength(10.0, 22.0) / base - 9.81 / 22.0).abs() < 1e-4);
    let packet = Packet::moving(10.0, 9.81, 2.0, (0.1, 0.0));
    assert!((TAU / packet.k_t - base).abs() < 1e-3);
}

/// Fast sources leave arms only, slow ones transverse waves too.
#[test]
fn only_slow_sources_leave_transverse_waves() {
    assert_eq!(transverse_share(0.2), 1.0);
    assert_eq!(transverse_share(5.0), 0.0);
    assert!(Packet::moving(3.0, 9.81, 2.0, (0.1, 0.0)).amp_t > 0.0);
    assert_eq!(Packet::moving(40.0, 9.81, 2.0, (0.1, 0.0)).amp_t, 0.0);
}

/// The ring of a moving source grows at the sine of the half angle times its speed.
#[test]
fn the_ring_grows_at_the_group_speed_of_the_arms() {
    let packet = Packet::moving(20.0, 9.81, 2.0, (0.1, 0.0));
    assert!((packet.front - arm_angle(20.0, 9.81, 2.0).sin() * 20.0).abs() < 1e-5);
    assert!(packet.k <= TAU / 6.0 + 1e-6);
}

/// The height follows the push, grows with the head of the liquid, and stays under its cap.
#[test]
fn the_height_follows_the_push() {
    let soft = amplitude(100.0, 1000.0, 9.81, (10.0, false));
    assert!(amplitude(400.0, 1000.0, 9.81, (10.0, false)) > soft);
    assert!(amplitude(100.0, 2700.0, 9.81, (10.0, false)) < soft);
    assert!(amplitude(100.0, 1000.0, 22.0, (10.0, false)) < soft);
    assert!(amplitude(100.0, 1000.0, 9.81, (10.0, true)) > soft);
    assert_eq!(amplitude(1.0e9, 1000.0, 9.81, (500.0, true)), MAX_AMPLITUDE);
    assert_eq!(amplitude(0.0, 1000.0, 9.81, (10.0, false)), 0.0);
}
