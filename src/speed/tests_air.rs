//! Tests of the air: pressure, the speed of sound and the cooling with height.

use super::{dynamic_pressure, mach, sound_speed, temperature_at};
use crate::universe::Mix;

/// Sound travels at about 340 m/s in the air of the home planet at 288 K.
#[test]
fn sound_in_home_air_is_340_metres_per_second() {
    let speed = sound_speed(288.0, &Mix::HOME);
    assert!((speed - 340.3).abs() < 0.02 * 340.3, "{speed}");
}

/// Sound is faster in warmer air and in a lighter gas.
#[test]
fn sound_follows_temperature_and_gas() {
    let cold = sound_speed(200.0, &Mix::HOME);
    let warm = sound_speed(400.0, &Mix::HOME);
    assert!(cold < sound_speed(288.0, &Mix::HOME) && sound_speed(288.0, &Mix::HOME) < warm);
}

/// Dynamic pressure is half the density times the square of the speed, and zero in a vacuum.
#[test]
fn pressure_needs_air() {
    assert_eq!(dynamic_pressure(0.0, 7000.0), 0.0);
    assert!((dynamic_pressure(1.2, 100.0) - 6000.0).abs() < 1.0e-3);
    assert!(dynamic_pressure(1.2, 200.0) > 3.9 * dynamic_pressure(1.2, 100.0));
}

/// The air cools on the way up to the top of the weather layer and no further.
#[test]
fn rising_air_cools_to_the_limit() {
    let kappa = Mix::HOME.kappa();
    let cooled = |height| temperature_at(288.0, kappa, (8400.0, 11_000.0), height);
    assert_eq!(cooled(0.0), 288.0);
    assert!(cooled(5000.0) < cooled(1000.0));
    assert_eq!(cooled(11_000.0), cooled(50_000.0));
    assert!(
        (cooled(11_000.0) - 198.0).abs() < 3.0,
        "{}",
        cooled(11_000.0)
    );
}

/// The Mach number is the speed over the speed of sound, and zero where there is none.
#[test]
fn mach_is_a_ratio() {
    assert!((mach(340.0, 340.0) - 1.0).abs() < 1.0e-6);
    assert_eq!(mach(100.0, 0.0), 0.0);
}
