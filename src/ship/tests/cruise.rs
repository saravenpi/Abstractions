//! The speed hold of the flight assist in space: engines idle means stop, gravity is cancelled
//! and the path comes onto the nose.

use std::f64::consts::FRAC_PI_2;

use bevy::math::{DQuat, DVec3};

use crate::ship::body::Ship;
use crate::ship::cruise::top_speed;
use crate::ship::flight::{Controls, STEP, Scene, step};
use crate::ship::wells::Well;
use crate::space::Frame;

/// Velocity of the body the ship flies beside, in metres per second: a planet on its orbit.
pub(super) const ORBIT: DVec3 = DVec3::new(8600.0, 0.0, 120.0);

/// Metres between the ship and the surface of the body in open space, where the lever is at
/// its fastest.
pub(super) const FAR: f64 = 40_000.0;

/// Returns a body the size of the home planet whose surface was `gap` metres under the origin
/// at time zero, where it is after `time` seconds.
pub(super) fn body(gap: f64, time: f64) -> Well {
    Well {
        pos: DVec3::NEG_Y * (6000.0 + gap) + ORBIT * time,
        vel: ORBIT,
        mu: 9.81 * 6000.0 * 6000.0,
        radius: 6000.0,
        lock: 18_000.0,
    }
}

/// Returns a ship with the assist on at the origin, moving at `drift` relative to the body,
/// nose along -Z, with the lever at `throttle`.
pub(super) fn beside(drift: DVec3, throttle: f64) -> Ship {
    let frame = Frame::System(Default::default());
    let mut ship = Ship::at_rest(frame, DVec3::ZERO, DQuat::IDENTITY);
    (ship.vel, ship.throttle) = (ORBIT + drift, throttle);
    ship
}

/// Steps `ship` for `seconds` beside the body that started `gap` metres under the origin, and
/// calls `each` after every step with the seconds flown and the velocity relative to the body.
pub(super) fn fly(
    ship: &mut Ship,
    controls: &Controls,
    run: (f64, f64),
    mut each: impl FnMut(f64, DVec3),
) {
    let (gap, seconds) = run;
    for i in 0..(seconds / STEP).round() as usize {
        let wells = [body(gap, i as f64 * STEP)];
        let scene = Scene {
            planet: None,
            ground: &|_| 0.0,
            wells: &wells,
            elapsed: 0.0,
        };
        step(ship, controls, &scene, STEP);
        each((i + 1) as f64 * STEP, ship.vel - ORBIT);
    }
}

/// Returns the last moment in 8 seconds at which a ship that drifted at `speed` toward the
/// body and across it still moved beside it, with the engines idle.
fn stop(speed: f64, controls: &Controls) -> f64 {
    let mut ship = beside(DVec3::new(0.6, -0.48, 0.64) * speed, 0.0);
    let mut moving = 0.0;
    fly(&mut ship, controls, (FAR, 8.0), |time, drift| {
        if drift.length() > 1e-3 {
            moving = time;
        }
    });
    moving
}

/// With the engines idle the ship comes to rest beside the body it is nearest to and stays
/// there, from any speed: in 0.6 s from 5 m/s, 1.2 s from 50, 2 s from 500 and 4.5 s from the
/// top speed. The brake does it in under 3 s. With the assist off the ship drifts on.
#[test]
fn idle_engines_stop_the_ship() {
    let idle = Controls::default();
    for (speed, limit) in [(5.0, 0.6), (50.0, 1.2), (500.0, 2.0), (2500.0, 4.5)] {
        let time = stop(speed, &idle);
        assert!(time < limit, "{speed} m/s took {time} s to shed");
    }
    let brake = Controls {
        brake: true,
        ..idle
    };
    let (braked, coasted) = (stop(2500.0, &brake), stop(2500.0, &idle));
    assert!(braked < 3.0 && braked < coasted - 1.0, "brake {braked} s");
    let mut free = beside(DVec3::X * 50.0, 0.0);
    free.assist = false;
    fly(&mut free, &idle, (FAR, 5.0), |_, _| {});
    assert!((free.vel - ORBIT).length() > 49.0);
}

/// Two kilometres over a body the idle ship hangs where it is, within a centimetre over half
/// a minute, while the same ship without assist falls by more than a kilometre.
#[test]
fn the_assist_cancels_gravity_in_space() {
    let fallen = |assist: bool| {
        let mut ship = beside(DVec3::ZERO, 0.0);
        ship.assist = assist;
        fly(&mut ship, &Controls::default(), (2000.0, 30.0), |_, _| {});
        ship.pos.distance(ORBIT * 30.0)
    };
    let (held, free) = (fallen(true), fallen(false));
    assert!(held < 0.01, "the ship moved {held} m beside the body");
    assert!(free > 1000.0, "the free ship fell only {free} m");
}

/// Returns the last moment in 6 seconds at which the path of a ship at half lever was off its
/// nose after a quarter turn, and the largest shares of its speed by which it went past the
/// speed asked for and back along the way it came.
fn quarter_turn(gap: f64) -> (f64, f64, f64) {
    let speed = 0.5 * top_speed(gap);
    let mut ship = beside(DVec3::NEG_Z * speed, 0.5);
    ship.rot = DQuat::from_rotation_y(FRAC_PI_2);
    let nose = ship.forward();
    let (mut off, mut over, mut back) = (0.0, 0.0f64, 0.0f64);
    fly(
        &mut ship,
        &Controls::default(),
        (gap, 6.0),
        |time, drift| {
            let along = drift.dot(nose);
            let across = (drift - nose * along).length();
            if across > 0.02 * speed || (along / speed - 1.0).abs() > 0.05 {
                off = time;
            }
            over = over.max(along / speed - 1.0);
            back = back.max(drift.z / speed);
        },
    );
    (off, over, back)
}

/// After a quarter turn the ship flies where it points: within a second at the 20 m/s of half
/// lever near a surface, within three seconds at 1,250 m/s in open space. It never goes more
/// than 1 percent past the speed asked for, nor back along the way it came.
#[test]
fn the_path_comes_onto_the_nose_after_a_quarter_turn() {
    for (gap, limit) in [(0.0, 1.0), (FAR, 3.0)] {
        let (off, over, back) = quarter_turn(gap);
        assert!(off < limit, "{gap} m: off the nose for {off} s");
        assert!(over < 0.01 && back < 0.01, "{gap} m: {over} {back}");
    }
}
