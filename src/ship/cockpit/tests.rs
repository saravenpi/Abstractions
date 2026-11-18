//! Cockpit rules: the stick, the lever and the instruments follow the real state of the ship,
//! smoothly.

use bevy::prelude::*;

use super::{Dial, LEVER, Levers, STICK, STICK_REACH};
use crate::ship::cruise::top_speed;
use crate::ship::telemetry::{ShipTelemetry, WarpState};
use crate::ship::warp::JUMP_CAP;

/// Returns the controls with the stick at `stick` and the lever at `throttle`.
pub(super) fn levers(stick: [f32; 3], throttle: f32) -> Levers {
    Levers {
        stick: Vec3::from_array(stick),
        throttle,
    }
}

/// The grip of the stick comes back for pitch up and goes left for a roll to the left; a yaw
/// twists the stick in place; the handle of the lever goes forward with the throttle.
#[test]
fn the_stick_and_the_lever_follow_the_input() {
    let rest = levers([0.0; 3], 0.0).grips();
    let grip = |stick: [f32; 3]| levers(stick, 0.0).grips()[1];
    assert!(rest[1].distance(STICK + Vec3::Y * 0.3) < 1.0e-5);
    let (back, ahead) = (grip([1.0, 0.0, 0.0]).z, grip([-1.0, 0.0, 0.0]).z);
    assert!(back > rest[1].z + 0.04 && ahead < rest[1].z - 0.04);
    assert!(grip([0.0, 0.0, 1.0]).x < -0.04 && grip([0.0, 0.0, -1.0]).x > 0.04);
    assert!(grip([0.0, 1.0, 0.0]).distance(rest[1]) < 1.0e-5);
    let twist = levers([0.0, 1.0, 0.0], 0.0).stick_turn();
    assert!((twist.to_scaled_axis() - Vec3::Y * STICK_REACH.1).length() < 1.0e-5);
    let handle: Vec<f32> = (0..=10)
        .map(|i| levers([0.0; 3], i as f32 / 10.0).grips()[0].z)
        .collect();
    assert!(
        handle.windows(2).all(|pair| pair[1] < pair[0]),
        "{handle:?}"
    );
    assert!(handle[0] > LEVER.z + 0.05 && handle[10] < LEVER.z - 0.05);
}

/// A key that goes down in one frame moves the stick over several, at 30 and at 144 frames
/// per second alike: it never jumps, never overshoots and gets there.
#[test]
fn the_controls_ease_to_what_is_asked() {
    for rate in [30.0_f32, 144.0] {
        let asked = levers([1.0, -1.0, 1.0], 1.0);
        let mut held = Levers::default();
        let mut largest = 0.0_f32;
        for _ in 0..(rate * 0.2) as usize {
            let next = held.eased(asked, 1.0 / rate);
            largest = largest.max((next.stick - held.stick).abs().max_element());
            assert!(next.stick.max_element() <= 1.0 && next.throttle <= 1.0);
            held = next;
        }
        assert!(largest < 0.5, "a step of {largest} at {rate} fps");
        assert!(
            (held.stick.x - 0.939).abs() < 0.01,
            "{held:?} at {rate} fps"
        );
        for _ in 0..(rate * 0.8) as usize {
            held = held.eased(asked, 1.0 / rate);
        }
        assert!(held.stick.distance(asked.stick) < 1.0e-3 && held.throttle > 0.999);
    }
}

/// Every instrument reads a real number of the ship: the speed against the top speed of the
/// place or the warp speed against the fastest jump, the spool of the drive, the gear.
#[test]
fn the_instruments_read_the_ship() {
    let flying = |speed: f64, height: f64, warp: WarpState| ShipTelemetry {
        surface_speed: speed,
        altitude_ground: height,
        warp,
        ..default()
    };
    let off = |speed: f64| Dial::Speed.reading(&flying(speed, 800.0, WarpState::Off), 1.0);
    assert!((off(0.5 * top_speed(800.0)) - 0.5).abs() < 1.0e-6);
    assert_eq!((off(0.0), off(1.0e9), off(f64::NAN)), (0.0, 1.0, 0.0));
    let jump = flying(0.0, f64::INFINITY, WarpState::Active(JUMP_CAP));
    assert_eq!(Dial::Speed.reading(&jump, 0.0), 1.0);
    assert_eq!(Dial::Charge.reading(&jump, 0.0), 1.0);
    let spool = flying(0.0, 0.0, WarpState::Spooling(0.25));
    assert_eq!(Dial::Charge.reading(&spool, 0.0), 0.25);
    assert_eq!(
        Dial::Charge.reading(&flying(9.0, 9.0, WarpState::Off), 0.0),
        0.0
    );
    let gear = |down: f64| Dial::Gear.reading(&spool, down);
    assert_eq!((gear(0.0), gear(0.6), gear(1.0)), (0.0, 0.6, 1.0));
    for dial in Dial::ALL {
        let (empty, full) = (dial.place(0.0), dial.place(1.0));
        assert!(empty.scale.x > 0.0 && full.scale.x > 20.0 * empty.scale.x);
    }
}
