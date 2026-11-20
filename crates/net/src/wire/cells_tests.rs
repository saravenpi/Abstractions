//! Tests of positions at planet scale on the wire.

use glam::DVec3;

use super::body_tests::{planet, robot};
use super::{Frame, RobotBody};

/// A robot on a planet of 1.2e7 m radius comes back within a millimetre.
#[test]
fn a_position_at_planet_scale_comes_back_within_a_millimetre() {
    let frame = Frame::Surface(planet());
    for position in [
        DVec3::new(1.2e7, 0.000_4, -3.3),
        DVec3::new(-8.5e6, 8.5e6, 1.2e7 - 0.123),
    ] {
        let sent = RobotBody {
            position,
            ..robot()
        };
        let mut bytes = Vec::new();
        sent.encode(&frame, &mut bytes);
        let got = RobotBody::decode(&frame, &bytes).unwrap();
        assert!((got.position - position).abs().max_element() <= 0.001);
    }
}
