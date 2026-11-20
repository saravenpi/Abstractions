//! The flight assist switched off: the Newtonian ship, frozen bit for bit so that no change to
//! the assist can reach it.

use bevy::math::{DQuat, DVec3};

use super::{above_pole, coast, planet};
use crate::ship::body::Ship;
use crate::ship::flight::{Controls, STEP, Scene, step};
use crate::ship::wells::Well;
use crate::space::Frame;

/// Returns one number that changes when any bit of the position, the velocity, the attitude or
/// the spin of `ship` changes.
fn digest(ship: &Ship) -> u64 {
    let parts = [
        ship.pos.to_array().to_vec(),
        ship.vel.to_array().to_vec(),
        ship.rot.to_array().to_vec(),
        ship.spin.to_array().to_vec(),
    ];
    let mix = |sum: u64, value: &f64| (sum ^ value.to_bits()).wrapping_mul(0x0100_0000_01b3);
    parts.iter().flatten().fold(0xcbf2_9ce4_8422_2325, mix)
}

/// Returns controls that use every thruster at once.
fn busy() -> Controls {
    Controls {
        turn: DVec3::new(0.3, -0.2, 0.5),
        lift: 0.4,
        boost: true,
        brake: true,
    }
}

/// Returns the ship after 8 seconds of powered flight without assist in the air of the home
/// planet, over ground 300 m high.
fn in_the_air() -> Ship {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let mut ship = above_pole(&planet, 900.0);
    (ship.vel, ship.throttle) = (DVec3::new(40.0, -20.0, -90.0), 0.7);
    coast(&mut ship, &busy(), (&planet, &|_| 300.0), 8.0);
    ship
}

/// Returns the ship after 5 seconds of powered flight without assist beside a moving body.
fn in_space() -> Ship {
    let well = Well {
        pos: DVec3::new(9000.0, -2000.0, 4000.0),
        vel: DVec3::new(120.0, 30.0, -8600.0),
        mu: 3.5e8,
        radius: 6000.0,
        lock: 18_000.0,
        hill: 1.0e6,
    };
    let frame = Frame::System(Default::default());
    let mut ship = Ship::at_rest(frame, DVec3::ZERO, DQuat::from_rotation_y(0.4));
    (ship.vel, ship.throttle, ship.assist) = (DVec3::new(15.0, 4.0, -8500.0), 0.6, false);
    for i in 0..600 {
        let scene = Scene {
            planet: None,
            ground: &|_| 0.0,
            wells: &[well],
            elapsed: f64::from(i % 4) * STEP,
        };
        step(&mut ship, &busy(), &scene, STEP);
    }
    ship
}

/// Returns the ship after a drop without assist onto a 10 degree slope and 4 seconds on it.
fn on_the_ground() -> Ship {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let rise = 10.0f64.to_radians().tan();
    let ground = move |p: DVec3| 200.0 + 6000.0 * p.x.atan2(p.y) * rise;
    let mut ship = above_pole(&planet, 204.0);
    coast(&mut ship, &Controls::default(), (&planet, &ground), 4.0);
    ship
}

/// With the assist off the ship flies exactly as it did before the assist learned to hold a
/// speed: the same bits in the air and in space. On the ground the number was taken again in
/// 0.9, when a touchdown above the safe speed, as this 7 m/s drop is, stopped bouncing. All three
/// were taken again in 0.11, with the standard gravity, the air of the chosen home and the new thrust.
#[test]
fn the_manual_ship_is_bit_identical() {
    let seen = [in_the_air(), in_space(), on_the_ground()].map(|ship| digest(&ship));
    let frozen = [
        10_802_390_405_902_409_831,
        15_576_487_431_479_213_754,
        7_546_659_858_878_267_499,
    ];
    assert_eq!(seen, frozen);
}
