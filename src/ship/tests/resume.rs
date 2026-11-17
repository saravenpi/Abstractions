//! The ship in a save: its record brings it back exactly, wherever it was.

use bevy::math::{DQuat, DVec3, IVec3};

use super::{above_pole, planet};
use crate::ship::body::{Ship, ShipState};
use crate::ship::save::record;
use crate::ship::warp::Warp;
use crate::ship::wells::{Well, system_wells};
use crate::space::Frame;
use crate::universe::{GLY, GalaxyId, Universe};

/// Returns `ship` after a trip through the text of a save file, and whether it was seated.
fn reloaded(ship: &Ship, wells: &[Well], seated: bool) -> (Ship, bool) {
    let text = toml::to_string(&record(ship, wells, seated)).unwrap();
    let state: ShipState = toml::from_str(&text).unwrap();
    (Ship::from_state(&state).unwrap(), state.seated)
}

/// Returns a flying ship in `frame` with every saved field set to an awkward value.
fn flying(frame: Frame, pos: DVec3) -> Ship {
    Ship {
        vel: DVec3::new(120.5, -0.30000000000000004, 7e-9),
        spin: DVec3::new(0.01, -0.2, 0.3),
        throttle: 0.37,
        damage: 0.125,
        assist: false,
        ..Ship::at_rest(frame, pos, DQuat::from_rotation_x(1.0 / 3.0))
    }
}

/// A ship landed, flying in the air, in orbit, between the planets and between the stars comes
/// back bit for bit, with the seat it was saved with.
#[test]
fn a_saved_ship_comes_back_exactly() {
    let universe = Universe::new(7);
    let home = universe.home();
    let land = planet(6000.0, 4000.0, 1200.0);
    let mut parked = above_pole(&land, 31.25);
    (parked.landed, parked.touching, parked.assist) = (true, true, true);
    let far = DVec3::new(1.0 / 3.0, -2.5e11, 7.0e9);
    let ships = [
        parked,
        flying(Frame::Surface(home), DVec3::new(100.1, 6900.7, -0.3)),
        flying(Frame::System(home.star), far),
        flying(Frame::System(home.star), far * 40.0),
        flying(Frame::Galaxy(GalaxyId(IVec3::new(-2, 0, 5))), far * GLY),
    ];
    for (index, ship) in ships.iter().enumerate() {
        let seated = index % 2 == 1;
        assert_eq!(
            reloaded(ship, &[], seated),
            (ship.clone(), seated),
            "{index}"
        );
    }
}

/// A ship saved in warp comes back out of warp and at rest relative to the nearest body, and a
/// ship saved while its drive spools comes back with the drive off and its own velocity.
#[test]
fn a_ship_saved_in_warp_comes_back_dropped_out() {
    let universe = Universe::new(7);
    let system = universe.system(universe.home().star);
    let wells = system_wells(&system, 4321.0);
    let near = wells[2];
    let pos = near.pos + DVec3::new(0.0, 1.0, 0.0) * near.lock * 4.0;
    let mut ship = flying(Frame::System(system.star.id), pos);
    ship.warp = Warp::Active { speed: 3.0e9 };
    let (back, _) = reloaded(&ship, &wells, true);
    assert_eq!(
        (back.warp, back.pos, back.rot),
        (Warp::Off, ship.pos, ship.rot)
    );
    assert_eq!(back.vel, near.vel, "not at rest beside the nearest body");
    ship.warp = Warp::Spooling { left: 2.5 };
    let (back, _) = reloaded(&ship, &wells, true);
    assert_eq!((back.warp, back.vel), (Warp::Off, ship.vel));
    ship.frame = Frame::Galaxy(system.star.id.galaxy);
    ship.warp = Warp::Active { speed: 1.0e13 };
    assert_eq!(reloaded(&ship, &[], true).0.vel, DVec3::ZERO);
}

/// A record written before the seat, the spin and the lever were saved still reads, and a
/// record with a number that is not finite is refused instead of flying off.
#[test]
fn old_and_broken_records_are_handled() {
    let ship = flying(Frame::Surface(Universe::new(7).home()), DVec3::Y * 6100.0);
    let mut value = toml::Value::try_from(ship.state()).unwrap();
    let table = value.as_table_mut().unwrap();
    for key in ["spin", "throttle", "damage", "manual", "seated"] {
        assert!(table.remove(key).is_some(), "{key} is not saved");
    }
    let old: ShipState = value.try_into().unwrap();
    let back = Ship::from_state(&old).unwrap();
    assert_eq!(
        (back.pos, back.vel, back.rot),
        (ship.pos, ship.vel, ship.rot)
    );
    assert!(back.assist && back.throttle == 0.0 && back.spin == DVec3::ZERO && !old.seated);
    let broken = ShipState {
        pos: [0.0, f64::NAN, 0.0],
        ..ship.state()
    };
    assert_eq!(Ship::from_state(&broken), None);
}
