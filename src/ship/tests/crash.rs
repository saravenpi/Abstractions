//! Crashes: a ship dropped hard on flat ground or on a slope stops dead, does not bounce,
//! comes to rest landed and tells of the crash once.

use bevy::math::DVec3;

use super::{above_pole, fly, planet};
use crate::ship::body::Ship;
use crate::ship::flight::Controls;
use crate::ship::gear::feet;
use crate::ship::landing::{GEAR_DROP, HULL};
use crate::ship::strike::CRASH;
use crate::universe::PlanetSpec;

/// Height in metres above sea level of the ground at the +Y pole in every drop.
const BASE: f64 = 200.0;

/// Highest any contact point may rise above the ground after the first touchdown, in metres.
const REBOUND: f64 = 0.1;

/// Seconds after the first touchdown within which the ship must be landed.
const SETTLE: f64 = 2.0;

/// What a drop did.
#[derive(Debug, Default)]
struct Drop {
    /// Highest the lowest contact point rose above the ground after the first touchdown.
    rebound: f64,
    /// Seconds from the first touchdown until the ship was landed, if it was.
    rest: Option<f64>,
    /// Speeds of every crash reported.
    crashes: Vec<f64>,
    /// Whether every number stayed finite.
    finite: bool,
}

/// Returns the height of a ground that rises by `degrees` toward +X around the +Y pole of `planet`.
fn ground(planet: &PlanetSpec, degrees: f64) -> impl Fn(DVec3) -> f64 {
    let (radius, rise) = (planet.radius, degrees.to_radians().tan());
    move |p: DVec3| BASE + radius * p.x.atan2(p.y) * rise
}

/// Returns how far in metres the lowest contact point of `ship` is above the ground.
fn clearance(ship: &Ship, planet: &PlanetSpec, height: &dyn Fn(DVec3) -> f64) -> f64 {
    let points = feet(ship).into_iter().chain(HULL);
    let above = |body: DVec3| {
        let at = ship.pos + ship.rot * body;
        at.length() - planet.radius - height(at)
    };
    points.map(above).fold(f64::INFINITY, f64::min)
}

/// Drops a ship without assist, its gear `gear` extended, at `vel` onto a ground of `degrees`
/// from 0.3 m above it, and watches it for eight seconds.
fn drop(vel: DVec3, degrees: f64, gear: f64) -> Drop {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let height = ground(&planet, degrees);
    let mut ship = above_pole(&planet, BASE + GEAR_DROP + 0.3);
    (ship.vel, ship.gear) = (vel, gear);
    let (mut seen, mut time, mut first) = (Drop::default(), 0.0, None::<f64>);
    seen.finite = true;
    fly(
        &mut ship,
        &Controls::default(),
        (&planet, &height),
        8.0,
        |ship, report| {
            time += 1.0 / 120.0;
            first = first.or((report.impact > 0.0).then_some(time));
            seen.crashes.extend(report.crash.map(|_| report.impact));
            seen.finite &= ship.pos.is_finite() && ship.vel.is_finite() && ship.spin.is_finite();
            if first.is_some() {
                seen.rebound = seen.rebound.max(clearance(ship, &planet, &height));
            }
            if ship.landed && seen.rest.is_none() {
                seen.rest = first.map(|at| time - at);
            }
        },
    );
    seen
}

/// Drops at 5, 15, 30 and 60 m/s, straight down and at 60 m/s with 40 m/s across, on flat
/// ground and on a 20 degree slope: no point rises 0.1 m after the touchdown, the ship is
/// landed within 2 seconds, every number stays finite and a crash is told once, at the speed
/// the gear met the ground with, only from 25 m/s.
#[test]
fn a_crash_stops_dead_and_comes_to_rest() {
    let drops = [5.0, 15.0, 30.0, 60.0].map(|down| DVec3::NEG_Y * down);
    let slid = DVec3::new(0.0, -60.0, 40.0);
    for degrees in [0.0, 20.0] {
        for vel in drops.into_iter().chain([slid]) {
            let seen = drop(vel, degrees, 1.0);
            let label = format!("{vel} on {degrees} degrees");
            assert!(seen.finite, "{label}: not finite");
            assert!(seen.rebound < REBOUND, "{label}: rose {} m", seen.rebound);
            assert!(
                seen.rest.is_some_and(|rest| rest < SETTLE),
                "{label}: {:?}",
                seen.rest
            );
            let normal = -vel.y * degrees.to_radians().cos();
            let crashes = usize::from(normal >= CRASH[0]);
            assert_eq!(seen.crashes.len(), crashes, "{label}: {:?}", seen.crashes);
            let near = |speed: &f64| (speed - normal).abs() < 1.5;
            assert!(seen.crashes.iter().all(near), "{label}: {:?}", seen.crashes);
        }
    }
}

/// With the gear folded the hull takes the touchdown, and 15 m/s is a crash on the belly that
/// the locked gear takes without one. The ship still comes to rest.
#[test]
fn a_belly_landing_is_a_crash_sooner() {
    let belly = drop(DVec3::NEG_Y * 15.0, 0.0, 0.0);
    assert_eq!(belly.crashes.len(), 1, "{:?}", belly.crashes);
    assert!(belly.rebound < REBOUND && belly.rest.is_some_and(|rest| rest < SETTLE));
    assert!(drop(DVec3::NEG_Y * 15.0, 0.0, 1.0).crashes.is_empty());
}
