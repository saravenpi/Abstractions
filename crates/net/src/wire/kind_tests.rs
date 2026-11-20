//! Tests of the kind table and the body trait, written once over every body.

use super::body_tests::planet;
use super::{BODY_MAX, Body, CompanionBody, Frame, Kind, RobotBody, ShipBody, VitalsBody};

/// Checks `B` against the rules every body follows: its kind names its own slot, it never
/// takes more than [`Body::MAX_LEN`] bytes, which fit [`BODY_MAX`], it takes exactly that in a
/// frame with `f64` positions, and it is refused at any other length.
fn follows_the_rules<B: Body + PartialEq + std::fmt::Debug>() {
    assert_eq!(Kind::ALL[B::KIND.index()], B::KIND);
    assert!(B::MAX_LEN <= BODY_MAX);
    let frames = [
        Frame::Surface(planet()),
        Frame::Around(planet()),
        Frame::System(planet().star),
    ];
    for frame in frames {
        let mut bytes = Vec::new();
        B::default().encode(&frame, &mut bytes);
        assert!(
            bytes.len() <= B::MAX_LEN,
            "{:?} takes {}",
            B::KIND,
            bytes.len()
        );
        assert_eq!(B::decode(&frame, &bytes), Some(B::default()));
        assert!(B::decode(&frame, &bytes[..bytes.len() - 1]).is_none());
        bytes.push(0);
        assert!(B::decode(&frame, &bytes).is_none());
    }
    let mut bytes = Vec::new();
    B::default().encode(&Frame::Around(planet()), &mut bytes);
    assert_eq!(bytes.len(), B::MAX_LEN);
}

/// Every body type follows the rules of the trait.
#[test]
fn every_body_follows_the_rules() {
    follows_the_rules::<RobotBody>();
    follows_the_rules::<ShipBody>();
    follows_the_rules::<CompanionBody>();
    follows_the_rules::<VitalsBody>();
}

/// The kinds are numbered from 0 in the order of [`Kind::ALL`], and each body names its own.
#[test]
fn kinds_are_numbered_in_order() {
    for (place, kind) in Kind::ALL.into_iter().enumerate() {
        assert_eq!(kind.index(), place);
    }
    let kinds = [
        RobotBody::KIND,
        ShipBody::KIND,
        CompanionBody::KIND,
        VitalsBody::KIND,
    ];
    assert_eq!(kinds, Kind::ALL);
}
