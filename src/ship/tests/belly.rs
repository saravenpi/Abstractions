//! The lift nozzles under the belly fire for vertical lift only: in a hover, a landing and slow
//! flight. A pitch in forward flight, in air or in space, turns the ship with its attitude jets
//! and never fires them.

use bevy::math::DVec3;
use bevy::prelude::Vec3;

use super::cruise::{FAR, ORBIT, beside, body};
use super::hover::{assisted, stick};
use super::{fly, planet};
use crate::ship::exhaust::{demand, lift};
use crate::ship::flight::{Controls, STEP, Scene, step};
use crate::ship::tune::LIFT_ACCEL;
use crate::universe::{PlanetSpec, STANDARD_GRAVITY};

/// Returns an Earth-sized planet with an atmosphere 100 km high, a scale height of 8.3 km.
fn earth() -> PlanetSpec {
    planet(6.0e6, 100_000.0, 86_400.0)
}

/// Returns controls with the stick pulled fully back: the nose pitches up.
fn pull() -> Controls {
    Controls {
        turn: DVec3::X,
        ..Controls::default()
    }
}

/// Returns the weakest and the strongest fire of the lift nozzles of an assisted ship let go
/// `height` metres over `planet` with `controls`, between `run.0` and `run.1` seconds of
/// flight. With a speed `run.2` over nothing it starts at that speed along its nose with the
/// full lever, else at rest with the lever idle.
fn belly(
    planet: &PlanetSpec,
    height: f64,
    controls: &Controls,
    run: (f64, f64, f64),
) -> (f32, f32) {
    let mut ship = assisted(planet, height);
    (ship.throttle, ship.vel) = (f64::from(u8::from(run.2 > 0.0)), ship.forward() * run.2);
    let land = (planet, &(|_| 0.0) as &dyn Fn(DVec3) -> f64);
    let (mut time, mut least, mut most) = (0.0, f32::INFINITY, 0.0f32);
    fly(&mut ship, controls, land, run.1, |ship, report| {
        time += STEP;
        if time > run.0 {
            let fire = lift(ship, report, (controls.lift, ship.vel));
            (least, most) = (least.min(fire), most.max(fire));
        }
    });
    (least, most)
}

/// A full pitch up for a second at 150 m/s and the full lever fires no lift nozzle, low in
/// thick air and at 20 km in thin air, where the nozzles fired before 0.11.0.
#[test]
fn a_pitch_in_forward_flight_never_fires_the_belly() {
    for height in [300.0, 20_000.0] {
        let (_, most) = belly(&earth(), height, &pull(), (0.0, 1.0, 150.0));
        assert_eq!(most, 0.0, "{height} m: the belly fired {most}");
    }
}

/// A released ship hovers on its lift nozzles, which hold its weight: 0.4 of their push under
/// standard gravity. On the way down to land they hold most of it.
#[test]
fn a_hover_and_a_landing_fire_the_belly() {
    let weight = (STANDARD_GRAVITY / LIFT_ACCEL) as f32;
    let (least, most) = belly(&earth(), 60.0, &Controls::default(), (2.0, 3.0, 0.0));
    assert!(
        (least - weight).abs() < 0.02 && (most - weight).abs() < 0.02,
        "{least} {most}"
    );
    let (least, _) = belly(&earth(), 30.0, &stick(-1.0), (0.5, 3.0, 0.0));
    assert!(
        least > 0.5 * weight,
        "the belly fell to {least} on the way down"
    );
}

/// In space a pitch at 300 m/s beside a body turns the ship with the attitude jets, which fire
/// fully as the turn starts, and never with the lift nozzles.
#[test]
fn a_pitch_in_space_fires_the_attitude_jets() {
    let mut ship = beside(DVec3::NEG_Z * 300.0, 1.0);
    let (mut jets, mut most) = (0.0f32, 0.0f32);
    for i in 0..120 {
        jets = jets.max(demand(Vec3::X, Some(&ship)).x);
        let wells = [body(FAR, f64::from(i) * STEP)];
        let scene = Scene {
            planet: None,
            ground: &|_| 0.0,
            wells: &wells,
            elapsed: 0.0,
        };
        let report = step(&mut ship, &pull(), &scene, STEP);
        most = most.max(lift(&ship, &report, (0.0, ship.vel - ORBIT)));
    }
    assert_eq!(jets, 1.0);
    assert_eq!(most, 0.0, "the belly fired {most}");
}
