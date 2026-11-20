//! The wingsuit flown by intents in the controller: the keys ask for rates in the frame of the
//! body, so a held roll key rolls it over and round, a held back key on a plain glide stalls out
//! of the climb, and let go the wings level.

use bevy::prelude::*;

use super::stage::body;
use super::wings::{hold, world};
use std::f32::consts::{FRAC_PI_2, TAU};

use crate::gear::{FlightState, Gear, Mode};
use crate::player::Player;
use crate::player::controller::Motion;
use crate::sim::{Intent, PlayerId};

/// Height in metres the test glides start at above the ground, low enough for thick air.
const HEIGHT: f32 = 400.0;

/// Returns the move intent of the forward and strafe keys.
pub(super) fn keys(forward: f32, strafe: f32) -> Intent {
    Intent::Move { forward, strafe }
}

/// Returns the flight path angle of `gear` above the horizon.
pub(super) fn path(gear: &Gear) -> f32 {
    let v = gear.state.velocity;
    v.y.atan2(v.with_y(0.0).length())
}

/// Returns the heading of the flight path of `gear`.
pub(super) fn heading(gear: &Gear) -> f32 {
    let v = gear.state.velocity;
    (-v.x).atan2(-v.z)
}

/// Returns a world whose body glides at `velocity` with the wings spread, [`HEIGHT`] above the
/// ground, hands off for `settle` seconds. The view stays north, as no look is sent.
pub(super) fn launch(velocity: Vec3, settle: f32) -> App {
    let mut app = world();
    let world = app.world_mut();
    let mut bodies =
        world.query_filtered::<(&mut Transform, &mut Motion, &mut Gear), With<Player>>();
    for (mut transform, mut motion, mut gear) in bodies.iter_mut(world) {
        transform.translation.y += HEIGHT;
        (motion.velocity, motion.wings, motion.grounded) = (velocity, true, false);
        let climb = velocity.y.atan2(velocity.with_y(0.0).length());
        gear.state = FlightState {
            mode: Mode::Glide,
            velocity,
            wings: 1.0,
            pitch: climb,
            ..FlightState::default()
        };
        gear.state.servo.yaw = (-velocity.x).atan2(-velocity.z);
        gear.state.servo.climb = climb;
    }
    hold(&mut app, (settle * 30.0) as usize, &[]);
    app
}

/// Holding A rolls the body left all the way round within three seconds, through flying on
/// its back, and the wings stay spread.
#[test]
fn holding_a_rolls_the_body_over_and_round() {
    let mut app = launch(Vec3::new(0.0, -7.0, -30.0), 2.0);
    let (mut turned, mut lowest) = (0.0_f32, 1.0_f32);
    let mut last = body(&mut app, PlayerId::LOCAL).2.state.bank;
    for _ in 0..90 {
        let (_, gear, _) = hold(&mut app, 1, &[keys(0.0, -1.0)]);
        turned += crate::player::wrap(gear.state.bank - last);
        last = gear.state.bank;
        lowest = lowest.min((gear.state.frame() * Vec3::Y).y);
        assert_eq!(gear.state.mode, Mode::Glide);
    }
    assert!(turned < -TAU, "rolled only {turned}");
    assert!(lowest < -0.8, "never on its back: {lowest}");
}

/// Holding S on a plain glide climbs and stalls out of the climb: it never loops and never ends
/// on its back, too slow for it. A glide already under the stall speed gets its nose down with
/// S still held.
#[test]
fn holding_s_on_a_plain_glide_stalls_out_and_a_stall_drops_the_nose() {
    let mut app = launch(Vec3::new(0.0, -7.0, -30.0), 2.0);
    for frame in 0..180 {
        let (_, gear, _) = hold(&mut app, 1, &[keys(-1.0, 0.0)]);
        let state = gear.state;
        assert!(
            state.pitch < FRAC_PI_2,
            "frame {frame}: pitch {}",
            state.pitch
        );
        assert!(
            (state.frame() * Vec3::Y).y > -0.3,
            "frame {frame}: on its back"
        );
    }
    let mut slow = launch(Vec3::new(0.0, 0.0, -11.0), 0.0);
    let (_, gear, _) = hold(&mut slow, 45, &[keys(-1.0, 0.0)]);
    assert!(
        gear.state.pitch < 0.0,
        "the nose stayed at {}",
        gear.state.pitch
    );
    assert_eq!(gear.state.mode, Mode::Glide);
}

/// Let go after a roll into a bank, the wings are within 3 degrees of level within 1.3 s.
#[test]
fn letting_go_levels_the_wings() {
    for strafe in [-1.0, 1.0] {
        let mut app = launch(Vec3::new(0.0, -7.0, -30.0), 2.0);
        let (_, banked, _) = hold(&mut app, 10, &[keys(0.0, strafe)]);
        assert!(banked.state.bank.abs() > 0.6, "bank {}", banked.state.bank);
        let (_, level, _) = hold(&mut app, 40, &[]);
        let bank = level.state.bank.to_degrees();
        assert!(bank.abs() < 3.0, "still banked {bank} degrees");
    }
}

/// Q and E turn the heading flat: the wings stay within 5 degrees of level the whole time.
#[test]
fn the_yaw_keys_turn_the_heading_with_the_wings_level() {
    for yaw in [-1.0, 1.0] {
        let mut app = launch(Vec3::new(0.0, -7.0, -30.0), 2.0);
        let from = heading(&body(&mut app, PlayerId::LOCAL).2);
        for frame in 0..180 {
            let (_, gear, _) = hold(&mut app, 1, &[Intent::Yaw(yaw)]);
            let bank = gear.state.bank.to_degrees();
            assert!(bank.abs() < 5.0, "frame {frame}: banked {bank} degrees");
        }
        let gear = body(&mut app, PlayerId::LOCAL).2;
        let turned = crate::player::wrap(heading(&gear) - from);
        assert!(turned * yaw < -1.0, "turned {turned} for {yaw}");
    }
}
