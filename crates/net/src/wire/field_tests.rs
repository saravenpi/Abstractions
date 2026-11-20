//! Tests of the records of the field: vitals, blows, wounds, impacts and attacks, and how many
//! robots one datagram carries.

use super::body_tests::{planet, robot};
use super::hunt_tests::{animal, size};
use super::{
    Act, Attack, Blow, Body, ClientMsg, DEFLECTED, Event, Frame, Head, HuntWorld, Impact, Kind,
};
use super::{Record, Rock, ServerMsg, VitalsBody, Wound, split};
use crate::log::PlayerId;

/// Returns the head a robot record of player `n` carries on the test planet.
fn head(n: u32, kind: Kind) -> Head {
    Head {
        player: PlayerId(n),
        kind,
        frame: Frame::Surface(planet()),
        time: 123_456.789,
        flags: 0,
    }
}

/// Returns a wound on part 3.
fn wound() -> Wound {
    Wound {
        part: 3,
        depth: 0.04,
        width: 0.02,
        vessel: 1,
        opened: 123_456.789,
        plugged: false,
    }
}

/// Sixteen robot records, as their players send them, fit one datagram of the budget.
#[test]
fn sixteen_robots_fit_one_datagram() {
    let mut body = Vec::new();
    robot().encode(&Frame::Surface(planet()), &mut body);
    let records = (1..=16).map(|n| Record {
        head: head(n, Kind::Robot),
        body: &body,
    });
    let batches = split(records, 1100);
    assert_eq!(batches.len(), 1, "{} bytes", batches[0].len());
}

/// A vitals body takes 11 bytes, comes back within the packing error, and is refused at any
/// other length.
#[test]
fn a_vitals_body_comes_back() {
    let sent = VitalsBody {
        harm: [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 1.0],
        heat: 0.35,
    };
    let mut bytes = Vec::new();
    sent.encode(&Frame::Around(planet()), &mut bytes);
    assert_eq!(bytes.len(), 11);
    let got = VitalsBody::decode(&Frame::Around(planet()), &bytes).unwrap();
    assert!(
        got.harm
            .iter()
            .zip(sent.harm)
            .all(|(a, b)| (a - b).abs() < 0.002)
    );
    assert_eq!((got.harm[0], got.harm[9]), (0.0, 1.0));
    assert!((got.heat - sent.heat).abs() < 0.002);
    assert!(VitalsBody::decode(&Frame::Around(planet()), &bytes[..10]).is_none());
    bytes.push(0);
    assert!(VitalsBody::decode(&Frame::Around(planet()), &bytes).is_none());
}

/// Every blow, wound, impact and attack comes back, at a size that fits a datagram of acts.
#[test]
fn the_field_acts_come_back() {
    let blow = Blow {
        victim: PlayerId(2),
        at: [0.1, 0.9, -0.2],
        joules: 40.0,
        cut: true,
        dir: [0.0, 0.0, -1.0],
        momentum: 12.5,
    };
    assert!(size(&ClientMsg::Hunt(Act::Blow(blow))) <= 50);
    size(&ClientMsg::Hunt(Act::Wound {
        animal: animal(),
        wound: wound(),
    }));
    size(&ClientMsg::Hunt(Act::Plug {
        animal: animal(),
        part: 3,
    }));
}

/// An impact and an attack come back.
#[test]
fn impacts_and_attacks_come_back() {
    let impact = Impact {
        body: planet(),
        at: [1.0e6, -2.0e6, 3.5],
        joules: 40.0,
        cut: false,
        outer: 2,
        tool: 1,
        result: DEFLECTED,
    };
    assert!(size(&ServerMsg::Hunt(Event::Impact(impact))) <= 70);
    let attack = Attack {
        animal: animal(),
        kind: 2,
        target: Some(PlayerId(3)),
        started: 123_456.789,
    };
    size(&ServerMsg::Hunt(Event::Attack(attack)));
}

/// The state of wounds and rocks after joining comes back.
#[test]
fn wounds_and_rocks_come_back() {
    let world = HuntWorld {
        wounds: vec![(animal(), wound())],
        rocks: vec![Rock {
            planet: planet(),
            id: u64::MAX,
            grams: 1500,
            whole: u32::MAX,
        }],
        ..HuntWorld::default()
    };
    size(&ServerMsg::Hunt(Event::World(world)));
}
