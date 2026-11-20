//! Cockpit rules: each control stands at an angle that is a pure function of its input and
//! stays in its range, and the controls follow what is asked without a step.

use bevy::prelude::*;

use super::Levers;
use super::tune::{LEVER, LEVER_SWING, PEDAL_REACH, STICK, STICK_REACH, THROW};

/// Returns the controls with the stick at `stick` and the lever at `throttle`.
pub(super) fn levers(stick: [f32; 3], throttle: f32) -> Levers {
    Levers {
        stick: Vec3::from_array(stick),
        throttle,
        ..default()
    }
}

/// Returns the angle of a turn from upright.
fn angle(turn: Quat) -> f32 {
    turn.angle_between(Quat::IDENTITY)
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

/// Every control turns by an angle that depends on its input alone and never leaves its range,
/// whatever the input: a value out of range is held to the end of the stroke.
#[test]
fn every_control_stays_in_its_range() {
    let swing = STICK_REACH.0 * 2.0 + STICK_REACH.1;
    for i in -30..=30 {
        let x = i as f32 * 0.1;
        let set = Levers {
            stick: Vec3::new(x, -x, x * 0.5),
            throttle: x,
            gear: x,
            warp: x,
        };
        assert_eq!(set.stick_turn(), set.stick_turn());
        assert!(angle(set.stick_turn()) <= swing + 1.0e-4);
        assert!(angle(set.lever_turn()) <= LEVER_SWING + 1.0e-4);
        assert!(angle(set.gear_turn()) <= THROW.0 + 1.0e-4);
        assert!(angle(set.switch_turn()) <= THROW.1 + 1.0e-4);
        assert!(angle(set.pedal_turn(0)) <= PEDAL_REACH + 1.0e-4);
        assert!(angle(set.pedal_turn(1)) <= PEDAL_REACH + 1.0e-4);
    }
}

/// The gear lever is back with the gear up and forward with it down, the warp switch is back
/// with the drive off, upright while it spools and forward in warp.
#[test]
fn the_gear_lever_and_the_warp_switch_follow_their_state() {
    let tip = |turn: Quat| (turn * Vec3::Y).z;
    let up = Levers::default();
    let down = Levers {
        gear: 1.0,
        warp: 1.0,
        ..default()
    };
    let spooling = Levers {
        warp: 0.5,
        ..default()
    };
    assert!(tip(up.gear_turn()) > 0.3 && tip(down.gear_turn()) < -0.3);
    assert!(tip(up.switch_turn()) > 0.3 && tip(down.switch_turn()) < -0.3);
    assert!(tip(spooling.switch_turn()).abs() < 1.0e-5);
}

/// A yaw to the left pushes the left pedal forward, toe down, and brings the right pedal back
/// by as much.
#[test]
fn the_pedals_follow_the_yaw() {
    let yaw = |value: f32| levers([0.0, value, 0.0], 0.0);
    assert_eq!(yaw(1.0).pedals(), [1.0, -1.0]);
    assert_eq!(yaw(0.0).pedals(), [0.0, 0.0]);
    assert_eq!(yaw(9.0).pedals(), [1.0, -1.0]);
    let toe = |turn: Quat| (turn * Vec3::NEG_Z).y;
    assert!(toe(yaw(1.0).pedal_turn(0)) < -0.1 && toe(yaw(1.0).pedal_turn(1)) > 0.1);
}

/// A key that goes down in one frame moves the stick over several, at 30 and at 144 frames
/// per second alike: it never jumps, never overshoots and gets there.
#[test]
fn the_controls_ease_to_what_is_asked() {
    for rate in [30.0_f32, 144.0] {
        let asked = Levers {
            gear: 1.0,
            ..levers([1.0, -1.0, 1.0], 1.0)
        };
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
        assert!(held.stick.distance(asked.stick) < 1.0e-3 && held.gear > 0.999);
    }
}
