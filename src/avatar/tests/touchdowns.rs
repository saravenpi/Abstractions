//! Touchdowns: each impact picks the way the body takes it, from what the gear recorded.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::stage::Touchdown;
use crate::avatar::Mode;
use crate::avatar::touch::{Touch, touch};
use crate::gear::Mode as Flight;

/// Impacts and the family each picks: the gear's touchdown, the velocity the body came in at in
/// its own frame, the speed lost into the ground and the hardness of the landing.
const IMPACTS: [(Flight, [f32; 3], f32, f32, Touch); 8] = [
    (Flight::Off, [0.0, -5.0, 0.0], 5.0, 0.0, Touch::Soft),
    (Flight::Off, [0.0, -20.0, 0.0], 20.0, 0.85, Touch::Hard),
    (Flight::Roll, [0.0, -16.0, -2.0], 16.0, 0.7, Touch::Hard),
    (Flight::Roll, [1.0, -14.0, -15.0], 14.0, 0.7, Touch::Ahead),
    (Flight::Roll, [0.0, -14.0, 12.0], 14.0, 0.7, Touch::Ahead),
    (Flight::Roll, [-15.0, -14.0, -3.0], 14.0, 0.7, Touch::Side),
    (Flight::Slide, [0.0, -5.0, -30.0], 5.0, 0.0, Touch::Skid),
    (Flight::Roll, [0.0, -40.0, -45.0], 40.0, 1.0, Touch::Tumble),
];

/// Each impact picks its family: soft or hard on the spot, a roll ahead or over the side, a skid
/// or a tumble, and each family its mode.
#[test]
fn each_impact_picks_its_family() {
    for (flight, struck, lost, hard, family) in IMPACTS {
        let struck = Vec3::from_array(struck);
        let found = touch(flight, struck, lost, hard);
        assert_eq!(found, family, "{flight:?} at {struck}, {lost} m/s lost");
    }
    let families = [Touch::Soft, Touch::Hard, Touch::Ahead, Touch::Side];
    let modes = [Mode::Land, Mode::Land, Mode::Roll, Mode::Roll];
    assert_eq!(families.map(Touch::mode), modes);
    assert_eq!(Touch::Tumble.mode(), Mode::Roll);
    assert_eq!(Touch::Skid.mode(), Mode::Skid);
}

/// Brought down by the real gear, the body reads the hit in its own frame whatever way it heads,
/// and poses the family the gear's touchdown and the impact call for.
#[test]
fn the_gear_hit_chooses_the_pose() {
    let yaw = FRAC_PI_2;
    let cases = [
        (Vec3::new(0.0, -20.0, -1.0), Flight::Roll, Mode::Land),
        (Vec3::new(0.0, -15.0, -18.0), Flight::Roll, Mode::Roll),
        (Vec3::new(-17.0, -15.0, 2.0), Flight::Roll, Mode::Roll),
        (Vec3::new(0.0, -4.0, -30.0), Flight::Slide, Mode::Skid),
        (Vec3::new(0.0, -45.0, -40.0), Flight::Roll, Mode::Roll),
        (Vec3::new(0.0, -5.0, -2.0), Flight::Off, Mode::Land),
    ];
    for (velocity, flight, wanted) in cases {
        let world = Quat::from_rotation_y(yaw) * velocity;
        let mut body = Touchdown::new(Flight::Fall, world, (yaw, 0.0), Mode::Fall);
        body.hit();
        body.ground(2);
        assert_eq!(body.state.mode, flight, "{velocity}");
        assert_eq!(body.pose.mode, wanted, "{velocity}");
        let read = body.pose.struck;
        assert!(read.distance(velocity) < 1e-3, "{velocity} read as {read}");
        assert!(
            (body.pose.lost + velocity.y).abs() < 1e-3,
            "lost {}",
            body.pose.lost
        );
    }
}
