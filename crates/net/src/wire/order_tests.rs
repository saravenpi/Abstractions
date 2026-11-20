//! Tests of the order of the messages and of the refusal of old bytes.

use glam::{DVec3, Vec3};

use super::body_tests::{planet, robot};
use super::{Act, ClientMsg, Event, Frame, RobotBody, Rock, ServerMsg, ShipBody, encode};
use crate::log::PlayerId;

/// The new messages sit after the old ones, so the numbers of the old ones never change.
#[test]
fn the_hunting_messages_come_last() {
    let pull = ClientMsg::Hunt(Act::Held(None));
    assert_eq!(encode(&pull).unwrap()[4..], [8, 6, 0]);
    let held = ServerMsg::Hunt(Event::Held {
        by: PlayerId(1),
        item: None,
    });
    assert_eq!(encode(&held).unwrap()[4..], [10, 7, 1, 0]);
    let rock = ClientMsg::Hunt(Act::Rock(Rock {
        planet: planet(),
        id: 0,
    }));
    assert_eq!(encode(&rock).unwrap()[4], 8);
    assert_eq!(encode(&rock).unwrap()[5], 12);
    assert_eq!(encode(&ClientMsg::Waypoint(None)).unwrap()[4..], [7, 0]);
}

/// A robot or ship body of the old format, 3 or 1 bytes short, is refused and never read with
/// its fields shifted.
#[test]
fn bodies_of_the_old_format_are_refused() {
    let frame = Frame::Surface(planet());
    let mut bytes = Vec::new();
    robot().encode(&frame, &mut bytes);
    assert_eq!(bytes.len(), 47);
    assert!(RobotBody::decode(&frame, &bytes[..44]).is_none());
    bytes.clear();
    let ship = ShipBody {
        position: DVec3::new(1.0, 2.0, 3.0),
        velocity: Vec3::ZERO,
        ..ShipBody::default()
    };
    ship.encode(&frame, &mut bytes);
    assert_eq!(bytes.len(), 41);
    assert!(ShipBody::decode(&frame, &bytes[..40]).is_none());
    let mut old_ship = bytes.clone();
    old_ship.pop();
    assert!(ShipBody::decode(&frame, &old_ship).is_none());
}
