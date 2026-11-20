//! The wounds and the attacks of animals on the network.

use abstractions_net::log::PlayerId as Wired;
use abstractions_net::wire::Act;
use bevy::prelude::*;

use super::rig::{acts, app};
use crate::fauna::bleed::{Bled, Cut};
use crate::fauna::{Attacking, Creature, Kind};
use crate::life::SpeciesId;

/// A cut a blow of this machine opened goes out as a wound of the animal that was cut.
#[test]
fn a_cut_goes_out_as_a_wound() {
    let (mut app, script) = app();
    let creature = Creature {
        species: SpeciesId {
            planet: 1,
            index: 0,
        },
        herd: 11,
        slot: 2,
    };
    let victim = app.world_mut().spawn(creature).id();
    let cut = Cut {
        part: 4,
        at: Vec3::ZERO,
        normal: Vec3::Y,
        width: 0.03,
        depth: 0.05,
        major: true,
        age: 0.0,
        owed: 0.0,
        plugged: false,
    };
    app.world_mut().write_message(Bled { victim, cut });
    app.update();
    let all = acts(&script);
    let [Act::Wound { animal, wound }] = all.as_slice() else {
        panic!("one wound");
    };
    assert_eq!((animal.herd, animal.slot), (11, 2));
    assert_eq!((wound.part, wound.vessel, wound.plugged), (4, 1, false));
}

/// An attack an animal starts against this robot goes out once, with this player as its target.
#[test]
fn an_attack_goes_out_with_its_target() {
    let (mut app, script) = app();
    let creature = Creature {
        species: SpeciesId {
            planet: 1,
            index: 0,
        },
        herd: 11,
        slot: 2,
    };
    app.world_mut()
        .spawn((creature, Attacking::begin(Kind::Bite, 1.0, 9.8)));
    app.update();
    app.update();
    let all = acts(&script);
    let [Act::Attack(attack)] = all.as_slice() else {
        panic!("one attack");
    };
    assert_eq!(
        attack.kind,
        Kind::ORDER.iter().position(|k| *k == Kind::Bite).unwrap() as u8
    );
    assert_eq!(attack.target, Some(Wired(3)));
}
