//! Far positions in a save: a ship at galaxy scale is written and read back bit for bit.

use std::fs;

use abstractions_net::wire::{self, Body};
use bevy::math::{DQuat, DVec3, IVec3};

use super::store::{list, new_entry, write};
use super::store_tests::scratch;
use crate::ship::{SavedFrame, Ship, ShipState};
use crate::space::{Frame, UniPos};
use crate::universe::{GalaxyId, StarId};

/// Returns the save `ship` goes into and back out of a file on disk.
fn through_disk(tag: &str, ship: &Ship) -> Ship {
    let dir = scratch(tag);
    let mut entry = new_entry(&dir, "Far", 5);
    entry.save.ship = Some(toml::Value::try_from(ship.state()).unwrap());
    write(&dir, &entry.id, &entry.save).unwrap();
    let loaded = list(&dir).remove(0).save;
    let _ = fs::remove_dir_all(&dir);
    let state: ShipState = loaded.ship.unwrap().try_into().unwrap();
    Ship::from_state(&state).unwrap()
}

/// A ship in the frame of a galaxy, 4.7e20 m and 1e21 m from its centre, is read back equal.
#[test]
fn a_galaxy_frame_ship_round_trips_bit_for_bit() {
    let galaxy = GalaxyId(IVec3::new(3, -1, 8));
    let at = DVec3::new(4.7e20, 0.0, -1.0e21);
    let ship = Ship::at_rest(Frame::Galaxy(galaxy), at, DQuat::IDENTITY);
    let back = through_disk("far-galaxy", &ship);
    assert_eq!(back.frame, Frame::Galaxy(galaxy));
    assert_eq!(
        back.pos.to_array().map(f64::to_bits),
        at.to_array().map(f64::to_bits)
    );
}

/// A ship parked 1 m from a point 5e20 m from the galaxy centre keeps that metre in the
/// universe position, in the save and on the wire, in the frames the game stores it in.
#[test]
fn a_metre_survives_arithmetic_save_and_wire_at_5e20_metres() {
    let star = StarId {
        galaxy: GalaxyId(IVec3::new(1, 2, -3)),
        sector: IVec3::new(5285, -40, 17),
        index: 2,
    };
    let beside = DVec3::new(1.0, 0.0, 0.0);
    let there = UniPos::sectors(
        star.galaxy.0,
        IVec3::new(5285, 7, 120),
        DVec3::splat(1.0e15),
    );
    assert_eq!((there + beside) - there, beside);

    let ship = Ship::at_rest(Frame::System(star), beside, DQuat::IDENTITY);
    let back = through_disk("far-metre", &ship);
    assert_eq!(back.frame, Frame::System(star));
    assert_eq!(back.pos, beside);

    let frame = wire::Frame::System(star);
    let mut bytes = Vec::new();
    wire::ShipBody {
        position: back.pos,
        ..Default::default()
    }
    .encode(&frame, &mut bytes);
    let got = wire::ShipBody::decode(&frame, &bytes).unwrap();
    assert_eq!(got.position, beside);
    assert_eq!(SavedFrame::from(back.frame), SavedFrame::System(star));
}
