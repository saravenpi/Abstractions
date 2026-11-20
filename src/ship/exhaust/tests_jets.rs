//! Jet tests: the table of the jets of a ship matches the sockets and nozzles of its hull, and
//! every attitude jet turns the ship the way of the stick axis it answers to.

use bevy::prelude::*;

use super::jets::{Kind, jets};
use crate::ship::mesh::{ENGINES, EXIT, LIFT, LIFT_AT, NOZZLE_Z, ports};

/// A ship has two main nozzles at the tail, four lift nozzles and fourteen attitude ports.
#[test]
fn a_ship_has_twenty_jets() {
    let all = jets();
    let count = |kind| all.iter().filter(|jet| jet.kind == kind).count();
    assert_eq!(
        (count(Kind::Main), count(Kind::Lift), count(Kind::Rcs)),
        (2, 4, 14)
    );
    for jet in all.iter().filter(|jet| jet.kind == Kind::Main) {
        assert_eq!(jet.at.z, NOZZLE_Z);
        assert_eq!(jet.dir, Vec3::Z);
        assert_eq!(jet.exit, EXIT);
        assert!(
            ENGINES
                .iter()
                .any(|e| (e.x - jet.at.x).abs() < 1e-6 && (e.y - jet.at.y).abs() < 1e-6)
        );
    }
}

/// The lift nozzles blow straight down from under the belly plate, 0.65 m under the middle of the
/// ship, and the four are as far ahead of the middle as behind it.
#[test]
fn the_lift_nozzles_blow_down_from_the_belly() {
    let lifts: Vec<_> = jets()
        .into_iter()
        .filter(|jet| jet.kind == Kind::Lift)
        .collect();
    for jet in &lifts {
        assert_eq!(jet.dir, Vec3::NEG_Y);
        assert!(jet.at.y < -0.65 && jet.exit == LIFT.1);
    }
    let mean = LIFT_AT.iter().map(|at| at.z).sum::<f32>() / 4.0;
    assert!(
        mean.abs() < 0.25,
        "the nozzles are centred {mean} m off the middle"
    );
    let across: f32 = LIFT_AT.iter().map(|at| at.x).sum();
    assert!(across.abs() < 1e-6);
}

/// Every attitude jet pushes the hull, against its exhaust, in the turn of the axis it answers to:
/// pitch up around +x, yaw left around +y and roll left around +z. Each way of each axis has jets.
#[test]
fn every_attitude_jet_turns_the_ship_the_way_of_its_axis() {
    let mut ways = [[0; 2]; 3];
    for port in ports() {
        assert_eq!(port.out.length(), 1.0);
        let torque = port.at.cross(-port.out);
        assert!(
            torque[port.axis] * port.sign > 1.0,
            "{port:?} turns {torque}"
        );
        ways[port.axis][usize::from(port.sign > 0.0)] += 1;
    }
    assert!(ways.iter().flatten().all(|n| *n >= 2), "{ways:?}");
}
