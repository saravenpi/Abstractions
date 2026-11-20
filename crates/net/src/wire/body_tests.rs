//! Tests of the record bodies: sizes, round trips within the packing error, and bad bytes.

use glam::{DVec3, IVec3, Quat, Vec2, Vec3};

use super::{Body, CompanionBody, Frame, RobotBody, ShipBody, WARP_ACTIVE, stride_delta};
use crate::ids::{BodyId, GalaxyId, StarId};

/// Returns a planet far from the origin of the universe.
pub(super) fn planet() -> BodyId {
    let star = StarId {
        galaxy: GalaxyId(IVec3::new(3, -2, 1)),
        sector: IVec3::new(-40, 12, 55),
        index: 4,
    };
    BodyId {
        star,
        index: 2,
        moon: 1,
    }
}

/// Returns a robot gliding over a planet, every field away from zero.
pub(super) fn robot() -> RobotBody {
    RobotBody {
        position: DVec3::new(1_203_456.789, -803_211.25, 96_000.5),
        velocity: Vec3::new(12.5, -3.25, 40.0),
        yaw: 2.5,
        mode: 10,
        speed: 41.75,
        vertical_speed: -3.25,
        stride_distance: 123.45,
        bank: -0.6,
        pitch: 0.3,
        thrust: 0.8,
        wings: 1.0,
        airspeed: 42.0,
        look: Vec2::new(0.7, -0.4),
        landed_hard: 0.25,
        crouch: 0.6,
        swing: 2 | 1 << 3,
        phase: 0.4,
    }
}

/// A robot body takes 46 bytes on a surface, so sixteen robots fit one datagram, and comes back
/// within the packing error, crouch and the swing of its hands included.
#[test]
fn a_robot_body_is_small_and_comes_back() {
    let (frame, sent) = (Frame::Surface(planet()), robot());
    let mut bytes = Vec::new();
    sent.encode(&frame, &mut bytes);
    assert_eq!(bytes.len(), 46);
    let got = RobotBody::decode(&frame, &bytes).unwrap();
    assert!((got.position - sent.position).abs().max_element() <= 0.000_500_001);
    assert_eq!((got.velocity, got.mode), (sent.velocity, sent.mode));
    assert!((got.yaw - sent.yaw).abs() < 1e-4);
    assert!((got.speed - sent.speed).abs() < 0.006);
    assert!((got.vertical_speed - sent.vertical_speed).abs() < 0.006);
    assert!((got.stride_distance - sent.stride_distance).abs() < 0.011);
    assert!((got.bank - sent.bank).abs() < 0.013 && (got.pitch - sent.pitch).abs() < 0.013);
    assert!((got.thrust - sent.thrust).abs() < 0.002 && got.wings == 1.0);
    assert!((got.airspeed - sent.airspeed).abs() < 0.51);
    assert!((got.look - sent.look).abs().max_element() < 0.013);
    assert!((got.landed_hard - sent.landed_hard).abs() < 0.002);
    assert!((got.crouch - sent.crouch).abs() < 0.04);
    assert_eq!(got.swing, sent.swing);
    assert!((got.phase - sent.phase).abs() < 0.07);
}

/// A heading past half a turn wraps instead of sticking at the end, and the stride distance
/// wraps in a way the difference of two samples sees through.
#[test]
fn the_heading_and_the_stride_wrap() {
    let frame = Frame::Surface(planet());
    let turned = RobotBody {
        yaw: 4.0,
        stride_distance: 656.36,
        ..robot()
    };
    let mut bytes = Vec::new();
    turned.encode(&frame, &mut bytes);
    let got = RobotBody::decode(&frame, &bytes).unwrap();
    assert!((got.yaw - (4.0 - std::f32::consts::TAU)).abs() < 1e-4);
    assert!((got.stride_distance - 1.0).abs() < 0.011);
    assert!((stride_delta(655.0, got.stride_distance) - 1.36).abs() < 0.011);
    assert!((stride_delta(1.0, 655.0) + 1.36).abs() < 0.001);
}

/// Away from a surface the position is sent in full, so it comes back exactly.
#[test]
fn a_position_in_space_comes_back_exactly() {
    let position = DVec3::new(1.5e11, -7.25e10, 3.123_456_789e9);
    for frame in [Frame::Around(planet()), Frame::System(planet().star)] {
        let sent = RobotBody {
            position,
            ..robot()
        };
        let mut bytes = Vec::new();
        sent.encode(&frame, &mut bytes);
        assert_eq!(bytes.len(), 55);
        assert_eq!(
            RobotBody::decode(&frame, &bytes).unwrap().position,
            position
        );
    }
}

/// A ship body takes 41 bytes on a surface and comes back within the packing error.
#[test]
fn a_ship_body_comes_back() {
    let frame = Frame::Surface(planet());
    let sent = ShipBody {
        position: DVec3::new(-1_000_000.123, 2.0, 3.0),
        velocity: Vec3::new(0.0, 250.0, -1.0),
        rotation: Quat::from_euler(glam::EulerRot::YXZ, 1.0, -0.4, 0.2),
        throttle: -0.5,
        gear: 0.4,
        damage: 0.1,
        landed: true,
        warp: WARP_ACTIVE,
        warp_level: 0.6,
        stick: [0.5, -1.0],
        boost: 0.75,
        heat: 0.5,
    };
    let mut bytes = Vec::new();
    sent.encode(&frame, &mut bytes);
    assert_eq!(bytes.len(), 41);
    let got = ShipBody::decode(&frame, &bytes).unwrap();
    assert!((got.position - sent.position).abs().max_element() <= 0.000_500_001);
    assert!(got.rotation.angle_between(sent.rotation) < 0.005);
    assert_eq!(
        (got.velocity, got.landed, got.warp),
        (sent.velocity, true, WARP_ACTIVE)
    );
    assert!((got.throttle - sent.throttle).abs() < 0.005);
    assert!((got.gear - sent.gear).abs() < 0.002 && (got.damage - sent.damage).abs() < 0.002);
    assert!((got.warp_level - sent.warp_level).abs() < 0.002 && got.stick[1] == -1.0);
    assert!((got.boost - sent.boost).abs() < 0.002 && (got.heat - sent.heat).abs() < 0.002);
}

/// A companion body takes 28 bytes on a surface and comes back within the packing error.
#[test]
fn a_companion_body_comes_back() {
    let frame = Frame::Surface(planet());
    let sent = CompanionBody {
        position: DVec3::new(10.0, 1_500_000.0, -20.0),
        rotation: Quat::from_rotation_y(2.0),
        lift: -0.2,
        yaw: 5.0,
        pitch: 0.25,
        roll: -0.24,
        spread: 0.5,
        wave: 1.0,
        expr: 4,
        face: 0b101_1011,
    };
    let mut bytes = Vec::new();
    sent.encode(&frame, &mut bytes);
    assert_eq!(bytes.len(), 28);
    let got = CompanionBody::decode(&frame, &bytes).unwrap();
    assert_eq!((got.position, got.wave), (sent.position, 1.0));
    assert!(got.rotation.angle_between(sent.rotation) < 0.005);
    assert!((got.lift - sent.lift).abs() < 0.004 && (got.yaw - sent.yaw).abs() < 1e-3);
    assert!((got.pitch - sent.pitch).abs() < 0.013 && (got.roll - sent.roll).abs() < 0.013);
    assert!((got.spread - sent.spread).abs() < 0.002);
    assert_eq!((got.expr, got.face), (4, 0b101_1011));
}

/// A body that is cut short, too long, read in the wrong frame or holding a number that is not
/// finite is refused.
#[test]
fn bad_bodies_are_refused() {
    let surface = Frame::Surface(planet());
    let space = Frame::System(planet().star);
    let mut bytes = Vec::new();
    robot().encode(&surface, &mut bytes);
    assert!(RobotBody::decode(&surface, &bytes[..40]).is_none());
    assert!(RobotBody::decode(&surface, &bytes[..44]).is_none());
    assert!(RobotBody::decode(&space, &bytes).is_none());
    assert!(ShipBody::decode(&surface, &bytes).is_none());
    assert!(CompanionBody::decode(&surface, &[]).is_none());
    bytes.push(0);
    assert!(RobotBody::decode(&surface, &bytes).is_none());
    bytes.pop();
    bytes[15..19].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(RobotBody::decode(&surface, &bytes).is_none());
    let mut far = Vec::new();
    let lost = RobotBody {
        position: DVec3::new(f64::INFINITY, 0.0, 0.0),
        ..robot()
    };
    lost.encode(&space, &mut far);
    assert!(RobotBody::decode(&space, &far).is_none());
}
