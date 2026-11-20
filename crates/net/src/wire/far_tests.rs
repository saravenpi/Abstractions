//! Tests of far positions on the wire: the frames of deep space carry three `f64`, whole.

use glam::{DVec3, IVec3, Quat, Vec3};

use super::body_tests::planet;
use super::{Body, Frame, ShipBody};
use crate::ids::{GalaxyId, StarId};

/// Returns a star 5e20 m from the centre of its galaxy, in sector 5285 of the first axis.
fn far_star() -> StarId {
    StarId {
        galaxy: GalaxyId(IVec3::new(-2, 1, 4)),
        sector: IVec3::new(5285, -3171, 12),
        index: 3,
    }
}

/// Returns a ship at `position` with every other field at rest.
fn ship(position: DVec3) -> ShipBody {
    ShipBody {
        position,
        velocity: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        ..ShipBody::default()
    }
}

/// A galaxy-scale position in a deep-space frame comes back bit for bit.
#[test]
fn a_galaxy_scale_position_comes_back_bit_for_bit() {
    let at = DVec3::new(5.0e20, -3.0e20, 2.4e22);
    for frame in [Frame::System(far_star()), Frame::Around(planet())] {
        let mut bytes = Vec::new();
        ship(at).encode(&frame, &mut bytes);
        assert_eq!(bytes.len(), 50);
        let got = ShipBody::decode(&frame, &bytes).unwrap();
        assert_eq!(
            got.position.to_array().map(f64::to_bits),
            at.to_array().map(f64::to_bits)
        );
    }
}

/// A ship a metre from the star of a far system stays a metre from it on the wire.
#[test]
fn a_metre_from_a_far_star_stays_a_metre() {
    let frame = Frame::System(far_star());
    for at in [
        DVec3::X,
        DVec3::new(0.0, -1.0, 0.0),
        DVec3::new(0.3, 0.4, 0.5 + 1e-9),
    ] {
        let mut bytes = Vec::new();
        ship(at).encode(&frame, &mut bytes);
        let got = ShipBody::decode(&frame, &bytes).unwrap();
        assert_eq!(got.position, at);
    }
}
