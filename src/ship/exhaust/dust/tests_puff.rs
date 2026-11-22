//! Tests of how a puff of dust flies: it slows to the wind within its drag time, settles toward
//! the centre of the planet at its settling speed and never sinks under the ground.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::puff::{Mote, fling, reread, step};
use super::rig::{RADIUS, aloft};
use super::soil::settle;
use super::tune::DRAG_TIME;
use crate::avatar::Soil;
use crate::universe::STANDARD_GRAVITY;

/// Frame step of the tests, in seconds.
const DT: f32 = 0.01;

/// Steps `mote` for `seconds` in `wind`.
fn fly(mote: &mut Mote, wind: Vec3, seconds: f32) {
    for _ in 0..(seconds / DT).round() as usize {
        step(mote, wind, DT);
    }
}

/// The air that carries a puff thrown at 10 m/s keeps about e^-1 of its gap to the wind after
/// one drag time and under 6 percent of it after three.
#[test]
fn a_puff_slows_to_the_wind_within_its_drag_time() {
    let wind = Vec3::new(0.0, 0.0, 2.0);
    let mut mote = aloft(DVec3::Y, 50.0, (Vec3::new(10.0, 0.0, 0.0), 0.0));
    let gap = (mote.flow - wind).length();
    fly(&mut mote, wind, DRAG_TIME);
    let share = (mote.flow - wind).length() / gap;
    assert!((0.33..0.42).contains(&share), "{share}");
    fly(&mut mote, wind, 2.0 * DRAG_TIME);
    assert!((mote.flow - wind).length() < 0.06 * gap, "{:?}", mote.flow);
}

/// In still air a puff falls straight toward the centre of the planet at its settling speed,
/// wherever it is on the planet, not along a fixed axis.
#[test]
fn a_puff_settles_toward_the_centre_at_its_speed() {
    let dir = DVec3::new(1.0, 2.0, -3.0).normalize();
    let mut mote = aloft(dir, 10.0, (Vec3::ZERO, 0.3));
    fly(&mut mote, Vec3::ZERO, 1.0);
    let height = mote.at.length() - RADIUS;
    assert!((height - 9.7).abs() < 1e-3, "{height}");
    assert!(mote.at.normalize().distance(dir) < 1e-9);
}

/// A puff thrown down hard, then left to settle for a minute, never goes under the ground, and
/// once there the air no longer pushes it down.
#[test]
fn a_puff_never_goes_under_the_ground() {
    let mut mote = aloft(
        DVec3::X,
        2.0,
        (Vec3::new(0.0, 0.0, 3.0) - Vec3::X * 20.0, 0.4),
    );
    for _ in 0..6000 {
        step(&mut mote, Vec3::new(0.0, 0.0, 1.0), DT);
        assert!(
            mote.at.length() >= mote.floor - 1e-6,
            "{}",
            mote.at.length() - mote.floor
        );
    }
    assert!(
        mote.flow.dot(mote.at.normalize().as_vec3()) >= -1e-6,
        "{:?}",
        mote.flow
    );
    let mut thrown = fling(DVec3::Y * RADIUS, (7, 1.0), 0.4);
    for _ in 0..600 {
        step(&mut thrown, Vec3::ZERO, DT);
        assert!(thrown.at.length() >= RADIUS - 1e-6);
    }
}

/// A puff that drifted over rising ground reads it again, and is then held on it.
#[test]
fn a_puff_reads_the_ground_it_drifts_over() {
    let mut mote = aloft(DVec3::Y, 0.5, (Vec3::ZERO, 0.4));
    mote.read = DVec3::Y * (RADIUS - 100.0);
    reread(&mut mote, 3.0, |_| RADIUS + 2.0);
    assert_eq!(mote.floor, RADIUS + 2.0);
    fly(&mut mote, Vec3::ZERO, 1.0);
    assert!(mote.at.length() >= RADIUS + 2.0 - 1e-6);
}

/// Fine dust settles at a few tenths of a metre per second at home, every soil under a third of
/// the gravity settles three times slower, and none settles faster than half a metre a second.
#[test]
fn dust_settles_slowly_in_proportion_to_gravity() {
    let home = STANDARD_GRAVITY as f32;
    for soil in [Soil::Sand, Soil::Earth, Soil::Snow, Soil::Land] {
        let speed = settle(soil, home);
        assert!((0.1..=0.5).contains(&speed), "{soil:?} {speed}");
        assert!((settle(soil, home / 3.0) - speed / 3.0).abs() < 1e-5);
    }
}
