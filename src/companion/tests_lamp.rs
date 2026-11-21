//! Tests of the screen light of the companion and of the place it keeps in the cockpit: the lamp
//! has a set power and the reach of the inverse square law, it is off by day, it shrinks with
//! the companion, and the riding spot is beside the displays and under the forward view.

use bevy::prelude::*;

use super::shine::{dark, lumens};
use super::tune::{HALF, LAMP_LUX, LIGHT_POWER, SEAT, SEAT_SCALE};
use crate::render::reach;
use crate::ship::cockpit::{BAYS, RAKE};

/// Illuminance of a clear night and of a full day in the game, in lux.
const PLACES: (f32, f32) = (310.0, 13_000.0);

/// Half width of the tub of the cockpit at the consoles, in meters: their outer face is the wall.
const WALL: f32 = 0.39;

/// Returns the radius of the ball around the riding companion, its hands included, in meters.
fn girth() -> f32 {
    (HALF[0] + 0.095) * SEAT_SCALE
}

/// In the dark the lamp gives the power of the lamp of the robot, 850 lm, and reaches as far:
/// 3.5 m. Its reach follows the inverse square law, so four times the power reaches twice as
/// far, and a companion a third of its size lights a third as far.
#[test]
fn the_lamp_has_a_set_power_and_the_reach_of_the_inverse_square_law() {
    assert_eq!(lumens(0.0, 1.0), LIGHT_POWER);
    assert_eq!(LIGHT_POWER, crate::avatar::tune::LIGHT_POWER);
    assert!((reach(lumens(0.0, 1.0)) - 3.5).abs() < 0.05);
    assert!((reach(4.0 * LIGHT_POWER) - 2.0 * reach(LIGHT_POWER)).abs() < 1e-3);
    let small = reach(lumens(0.0, SEAT_SCALE));
    assert!((small - SEAT_SCALE * reach(LIGHT_POWER)).abs() < 1e-3);
    assert!((lumens(0.0, SEAT_SCALE) - LIGHT_POWER * SEAT_SCALE * SEAT_SCALE).abs() < 1e-3);
}

/// The lamp is for the dark: full at night, off from the daylight threshold up, off in a full
/// day, and never brighter under more light.
#[test]
fn the_lamp_is_off_by_day() {
    assert_eq!(lumens(PLACES.0, 1.0), LIGHT_POWER);
    assert_eq!(lumens(LAMP_LUX.1, 1.0), 0.0);
    assert_eq!(lumens(PLACES.1, 1.0), 0.0);
    assert!(dark(PLACES.1 / 4.0) <= 0.0);
    let steps: Vec<f32> = (0..40).map(|i| dark(10.0 * 1.25_f32.powi(i))).collect();
    assert!(steps.windows(2).all(|pair| pair[1] <= pair[0]));
    assert!(steps.iter().all(|share| (0.0..=1.0).contains(share)));
    let dusk = dark((LAMP_LUX.0 * LAMP_LUX.1).sqrt());
    assert!((dusk - 0.5).abs() < 1e-3, "half way at dusk: {dusk}");
}

/// Returns the angle in radians between the directions `a` and `b` from the eye.
fn between(a: Vec3, b: Vec3) -> f32 {
    a.normalize().dot(b.normalize()).clamp(-1.0, 1.0).acos()
}

/// Seated, the companion is outside the cone from the eye of the pilot to each of the three
/// displays, with its hands, and under the picture when the pilot looks level at the default
/// field of view of 75 degrees. It is ahead and to the side, inside the wall of the tub.
#[test]
fn the_riding_spot_is_beside_the_displays_and_under_the_forward_view() {
    let seat = Vec3::from_array(SEAT);
    let eye = crate::ship::SEAT.as_vec3();
    let wide = girth().atan2(seat.length());
    for (across, width) in BAYS {
        let middle = RAKE.0 + Vec3::X * across - eye;
        let size = (width * 0.5).hypot(BAYS[1].1 * 0.5);
        let needed = size.atan2(middle.length()) + wide;
        let found = between(seat, middle);
        assert!(
            found > needed,
            "display at {across}: {found} rad, needs {needed}"
        );
    }
    let half = (75f32.to_radians() / 2.0).tan();
    let top = (seat.y + girth()) / -seat.z / half;
    assert!(
        top < -1.0,
        "its top is at {top} of the half height of the picture"
    );
    assert!(seat.z < -0.3 && seat.x.abs() + girth() < WALL);
}
