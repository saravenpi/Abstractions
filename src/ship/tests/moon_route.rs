//! Routes to a moon and saves on a moon: the autopilot warps to the point beside the moon and
//! stops there, and a ship saved in the surface frame of a moon comes back to it.

use bevy::math::{DQuat, DVec3};

use super::moons::{GIANT, giant_system};
use crate::ship::body::{Ship, ShipState};
use crate::ship::flight::{STEP, Scene, rotate, step};
use crate::ship::helm::Helm;
use crate::ship::moons::{Standoff, goal};
use crate::ship::pilot::Goal;
use crate::ship::route::shell;
use crate::ship::route::{hold, led, spool};
use crate::ship::warp::{CRUISE_CAP, Warp, advance, drop_out, request};
use crate::ship::wells::{Well, system_wells};
use crate::space::Frame;
use crate::universe::{BodyId, PlanetSpec, SystemSpec, Universe};

/// Seconds of one frame of the test voyage.
const DT: f64 = 1.0 / 30.0;

/// Sides a route to a moon starts from: degrees from the line out through the moon in the
/// orbit plane, then degrees toward +Y.
const SIDES: [(i16, i16); 8] = [
    (130, 0),
    (0, 0),
    (60, 0),
    (180, 0),
    (250, 0),
    (130, 50),
    (180, -40),
    (0, 90),
];

/// Times the year of the giant is made longer than its own in these tests. A body of a few
/// kilometres cannot be led within a lock sphere that small when it goes round its star at the
/// speed of a real planet, 30 kilometres per second; a giant that is slow leaves its moons the
/// speed of their own orbit around it.
const SLOWER: f64 = 1000.0;

/// Returns the system of [`giant_system`] with the year of the giant [`SLOWER`] times longer,
/// and the moon going round it as before.
fn slow_giant_system(orbit: f64) -> (SystemSpec, usize) {
    let (mut system, index) = giant_system(orbit);
    system.planets[index].orbit_seconds *= SLOWER;
    let host = system.planets[index].orbit();
    system.planets.last_mut().unwrap().host = Some(host);
    (system, index)
}

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
    let warping = matches!(ship.warp, Warp::Active { .. });
    advance(ship, wells, CRUISE_CAP, DT);
    if matches!(ship.warp, Warp::Active { .. }) {
        return rotate(ship, helm.controls.turn, DVec3::ZERO, DT);
    }
    if warping {
        return;
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
/// arrives, as the route system does, and returns the time of arrival and how many times the
/// drive spooled up on the way.
pub(super) fn voyage(system: &SystemSpec, goal: &Goal, ship: &mut Ship) -> (f64, u32) {
    let beyond = goal.beyond.as_ref().unwrap();
    let (mut helm, mut time, mut spools) = (Helm::default(), 0.0, 0);
    while time < 1200.0 {
        let mut wells = system_wells(system, time);
        if let Some(moon) = wells.last_mut() {
            moon.lock = goal.lock;
        }
        if beyond.reached(ship.pos, time) {
            drop_out(ship, &wells);
            return (time, spools);
        }
        let idle = !matches!(ship.warp, Warp::Spooling { .. });
        let here = ship.pos + ship.vel * spool(ship.warp);
        let point = |ahead: f64| Some(beyond.point(time + ahead) - here);
        let to = led(ship.warp, (goal.lock, CRUISE_CAP), point).unwrap();
        hold(ship, &mut helm, &wells, (to, goal.lock));
        frame(ship, &mut helm, &wells);
        spools += u32::from(idle && matches!(ship.warp, Warp::Spooling { .. }));
        if beyond.reached(ship.pos, time) {
            drop_out(ship, &wells);
            return (time, spools);
        }
        time += DT;
    }
    panic!("the route to the moon never arrived");
}

/// A route to a moon deep in the lock sphere of its giant warps there and drops out beside the
/// moon, at rest relative to it, outside its lock, the lock of the giant and the air of the giant.
#[test]
fn a_route_to_a_moon_ends_at_rest_beside_it() {
    let (system, index) = slow_giant_system(2.5);
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
    let (time, _) = voyage(&system, &route, &mut ship);
    let wells = system_wells(&system, time);
    let (host, own) = (wells[index + 1], *wells.last().unwrap());
    assert_eq!(ship.warp, Warp::Off);
    assert!(time > 10.0, "arrived after {time} s");
    assert!(
        (ship.vel - own.vel).length() < 1e-9,
        "not at rest beside the moon"
    );
    assert!(own.pos.distance(ship.pos) > shell::lock(moon) && host.clearance(ship.pos) > 0.0);
    let air = giant.radius + giant.atmosphere_height;
    assert!(host.pos.distance(ship.pos) > air);
}

/// Returns a ship at rest beside `moon` at time 0, 100 km from it, its nose on the moon, along
/// `out`, the line from its giant through it, turned by `side.0` degrees in the orbit plane and
/// then by `side.1` degrees toward +Y.
fn beside_moon(moon: &PlanetSpec, out: DVec3, side: (i16, i16)) -> Ship {
    let (turn, up) = (
        f64::from(side.0).to_radians(),
        f64::from(side.1).to_radians(),
    );
    let flat = DQuat::from_rotation_y(turn) * out;
    let dir = (flat * up.cos() + DVec3::Y * up.sin()).normalize();
    let well = Well::planet(moon, 0.0);
    let rot = DQuat::from_rotation_arc(DVec3::NEG_Z, -dir);
    let mut ship = Ship::at_rest(Frame::System(moon.id.star), well.pos + dir * 1.0e5, rot);
    ship.vel = well.vel;
    ship
}

/// A route to a moon far from its giant arrives from any side, the side of the giant at 130
/// degrees from the line out through the moon included, and above and below the orbit plane:
/// one spool and one warp, or two when the moon is in the way and the ship never turns in warp,
/// and the ship left at rest beside the moon, clear of both locks.
#[test]
fn a_route_to_a_moon_arrives_from_every_side() {
    let (system, index) = slow_giant_system(40.0);
    let (giant, moon) = (&system.planets[index], system.planets.last().unwrap());
    let out = (moon.position(0.0) - giant.position(0.0)).normalize();
    for side in SIDES {
        let mut ship = beside_moon(moon, out, side);
        let (time, spools) = voyage(&system, &goal(moon, giant), &mut ship);
        let wells = system_wells(&system, time);
        let (host, own) = (wells[index + 1], *wells.last().unwrap());
        assert!(
            time < 300.0 && (1..=2).contains(&spools),
            "{side:?}: {time} s, {spools} spools"
        );
        assert!(
            (ship.vel - own.vel).length() < 1e-9,
            "{side:?}: not at rest"
        );
        assert!(own.pos.distance(ship.pos) > shell::lock(moon) && host.clearance(ship.pos) > 0.0);
        assert!(own.pos.distance(ship.pos) <= own.lock * 1.45, "{side:?}");
    }
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
        assert!((from - GIANT * 5.0 * 1.3).abs() < 1e-3, "{from}");
    }
    let (far, index) = giant_system(40.0);
    let near = Standoff::of(far.planets.last().unwrap(), &far.planets[index]);
    assert_eq!(near.gap, shell::radius(far.planets.last().unwrap()));
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
