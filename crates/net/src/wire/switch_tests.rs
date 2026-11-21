//! The switches of a ship record: the bits of its state byte.

use super::body_tests::planet;
use super::{Body, Frame, ShipBody};

/// The state byte of a ship body carries whether it is landed, whether its headlight is on and
/// whether its canopy is open, each on its own bit, and a byte that only says landed, as a
/// record written before the other two were sent, reads with the headlight off and the canopy shut.
#[test]
fn a_ship_body_keeps_its_switches() {
    let frame = Frame::Surface(planet());
    for bits in 0..8u8 {
        let sent = ShipBody {
            landed: bits & 1 != 0,
            headlight: bits & 2 != 0,
            canopy: bits & 4 != 0,
            ..ShipBody::default()
        };
        let mut bytes = Vec::new();
        sent.encode(&frame, &mut bytes);
        let state = bytes.len() - 7;
        assert_eq!(bytes[state], bits);
        let got = ShipBody::decode(&frame, &bytes).unwrap();
        let switches = (got.landed, got.headlight, got.canopy);
        assert_eq!(switches, (sent.landed, sent.headlight, sent.canopy));
    }
}
