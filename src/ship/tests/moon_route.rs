//! Routes to a moon and saves on a moon: the autopilot warps to the point beside the moon and
//! stops there, and a ship saved in the surface frame of a moon comes back to it.

use bevy::math::{DQuat, DVec3};

use super::moons::{GIANT, giant_system};
use crate::ship::body::{Ship, ShipState};
use crate::ship::flight::{STEP, Scene, rotate, step};
use crate::ship::helm::Helm;
use crate::ship::moons::{Standoff, goal};
use crate::ship::pilot::Goal;
use crate::ship::route::hold;
use crate::ship::warp::{CRUISE_CAP, Warp, advance, drop_out, request};
use crate::ship::wells::{PLANET_LOCK, Well, system_wells};
use crate::space::Frame;
use crate::universe::{BodyId, SystemSpec, Universe};

/// Seconds of one frame of the test voyage.
const DT: f64 = 1.0 / 30.0;

/// Flies one frame: the drive switches when the helm asks, a ship in warp only turns, and
/// otherwise four flight steps carry it among `wells`.
fn frame(ship: &mut Ship, helm: &mut Helm, wells: &[Well]) {
    if std::mem::take(&mut helm.warp) {
        if ship.warp == Warp::Off {
            request(ship, wells);
        } else {
            drop_out(ship, wells);
        }
    }
    advance(ship, wells, CRUISE_CAP, DT);
    if matches!(ship.warp, Warp::Active { .. }) {
        return rotate(ship, helm.controls.turn, DVec3::ZERO, DT);
    }
    for sub in 0..4 {
        let scene = Scene {
            planet: None,
            ground: &|_| 0.0,
            wells,
            elapsed: f64::from(sub) * STEP,
        };
        step(ship, &helm.controls, &scene, STEP);
    }
}

/// Flies the autopilot along `goal` to the moon of `system`, the last of its bodies, until it
/// arrives, as the route system does, and returns the time of arrival.
fn voyage(system: &SystemSpec, goal: &Goal, ship: &mut Ship) -> f64 {
    let (moon, beyond) = (system.planets.last().unwrap(), goal.beyond.unwrap());
    let (mut helm, mut time) = (Helm::default(), 0.0);
    while time < 1200.0 {
        let wells = system_wells(system, time);
        let to = moon.position(time) + beyond.offset(time) - ship.pos;
        if to.length() <= goal.arrival {
            drop_out(ship, &wells);
            return time;
        }
        hold(ship, &mut helm, &wells, (to, goal.lock));
        frame(ship, &mut helm, &wells);
        time += DT;
    }
    panic!("the route to the moon never arrived");
}

/// A route to a moon deep in the lock sphere of its giant warps there and drops out beside the
/// moon, at rest relative to it, outside its lock, the lock of the giant and the air of the giant.
#[test]
fn a_route_to_a_moon_ends_at_rest_beside_it() {
    let (system, index) = giant_system(2.5);
    let (giant, moon) = (&system.planets[index], system.planets.last().unwrap());
    let route = goal(moon, giant);
    let start = Well::planet(giant, 0.0);
    let out = start.pos.normalize();
    let rot = DQuat::from_rotation_arc(DVec3::NEG_Z, -out);
    let mut ship = Ship::at_rest(
        Frame::System(giant.id.star),
        start.pos + out * GIANT * 400.0,
        rot,
    );
    ship.vel = start.vel;
    let time = voyage(&system, &route, &mut ship);
    let wells = system_wells(&system, time);
    let (host, own) = (wells[index + 1], *wells.last().unwrap());
    assert_eq!(ship.warp, Warp::Off);
    assert!(time > 10.0, "arrived after {time} s");
    assert!(
        (ship.vel - own.vel).length() < 1e-9,
        "not at rest beside the moon"
    );
    assert!(own.clearance(ship.pos) > 0.0 && host.clearance(ship.pos) > 0.0);
    let air = giant.radius + giant.atmosphere_height;
    assert!(host.pos.distance(ship.pos) > air);
}

/// The point beside a moon clears the air of its giant when that reaches past the lock, and
/// lies just outside the lock of the moon when the moon is far from its giant.
#[test]
fn the_point_beside_a_moon_clears_its_giant() {
    let (mut system, index) = giant_system(2.5);
    system.planets[index].atmosphere_height = GIANT * 4.0;
    let (giant, moon) = (&system.planets[index], system.planets.last().unwrap());
    let beyond = Standoff::of(moon, giant);
    for time in [0.0, 500.0, 7777.0] {
        let point = moon.position(time) + beyond.offset(time);
        let from = point.distance(giant.position(time));
        assert!((from - GIANT * 5.0 * 1.3).abs() < 1e-6, "{from}");
    }
    let (far, index) = giant_system(40.0);
    let near = Standoff::of(far.planets.last().unwrap(), &far.planets[index]);
    assert_eq!(near.gap, 3000.0 * PLANET_LOCK * 1.3);
}

/// A ship saved in the surface frame of a moon comes back on that moon, and a record written
/// before moons existed, without the moon field, comes back on the planet it named.
#[test]
fn a_ship_saved_on_a_moon_resumes_there() {
    let planet = Universe::new(7).home();
    let moon = BodyId { moon: 2, ..planet };
    let ship = Ship::at_rest(Frame::Surface(moon), DVec3::Y * 3100.0, DQuat::IDENTITY);
    let text = toml::to_string(&ship.state()).unwrap();
    let state: ShipState = toml::from_str(&text).unwrap();
    assert_eq!(
        Ship::from_state(&state).unwrap().frame,
        Frame::Surface(moon)
    );
    let mut old = toml::Value::try_from(
        Ship {
            frame: Frame::Surface(planet),
            ..ship
        }
        .state(),
    )
    .unwrap();
    let body = old["frame"]["Surface"].as_table_mut().unwrap();
    assert!(body.remove("moon").is_some(), "the moon is not saved");
    let state: ShipState = old.try_into().unwrap();
    let back = Ship::from_state(&state).unwrap();
    assert_eq!(back.frame, Frame::Surface(planet));
    assert!(!planet.is_moon() && back.pos == ship.pos);
}
