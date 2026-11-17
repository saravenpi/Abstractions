//! The flight gear inside the controller: touchdown, keys and the velocity it hands back.

use bevy::prelude::*;

use crate::gear::FlightState;
use crate::gear::{Gear, Mode};
use crate::player::controller::Motion;
use crate::player::flight::*;
use crate::player::ground::Ground;
use crate::player::intent::Intent;

/// Returns a gear in `mode` moving at `velocity`.
fn gear(mode: Mode, velocity: Vec3) -> Gear {
    Gear {
        state: FlightState {
            mode,
            velocity,
            ..FlightState::default()
        },
        ..Gear::default()
    }
}

/// A flying body lands only on contact, a sliding one stays on the ground within the snap,
/// and a slide follows the slope while a glide keeps its own velocity.
#[test]
fn the_gear_touches_down_on_contact_and_slides_along_the_ground() {
    let near = Ground {
        normal: Vec3::new(0.0, 0.8, 0.6),
        gap: 0.3,
    };
    let on = Ground { gap: 0.02, ..near };
    assert!(!touching(Mode::Fly, Some(near)) && touching(Mode::Fly, Some(on)));
    assert!(touching(Mode::Slide, Some(near)) && touching(Mode::Off, Some(near)));
    assert!(!touching(Mode::Off, None));
    let fast = Vec3::new(0.0, -2.0, -12.0);
    assert_eq!(carried(&gear(Mode::Off, fast), Some(on), 0.016), None);
    assert_eq!(
        carried(&gear(Mode::Glide, fast), Some(on), 0.016),
        Some(fast)
    );
    let slid = carried(&gear(Mode::Slide, fast), Some(on), 0.016).unwrap();
    assert!(slid.dot(on.normal).abs() < 1e-4 && (slid.length() - 12.0).abs() < 1e-3);
    let mut sliding = gear(Mode::Slide, fast);
    assert_eq!(settle(&mut sliding, Vec3::new(1.0, -3.0, -9.0)).y, 0.0);
}

/// The jump key fires the boots, the sprint key only with the wingsuit asked open, and the
/// forward key pushes the nose down.
#[test]
fn keys_map_to_the_gear() {
    let air = Air {
        density: 1.2,
        wind: Vec3::X,
        clearance: 9.0,
        look: Vec3::NEG_Z,
        gravity: 22.0,
    };
    let mut intent = Intent {
        wish: Vec3::ZERO,
        forward: 1.0,
        right: -1.0,
        sprint: true,
        jump: false,
    };
    let mut motion = Motion::default();
    let told = input(&motion, &intent, &air, (false, false));
    assert!(!told.boost && !told.deploy);
    assert_eq!((told.pitch, told.roll, told.wind), (-1.0, -1.0, Vec3::X));
    motion.wings = true;
    assert!(input(&motion, &intent, &air, (false, false)).boost);
    (motion.wings, intent.sprint, motion.boots) = (false, false, true);
    let told = input(&motion, &intent, &air, (true, true));
    assert!(told.boost && told.grounded && told.in_water);
}
