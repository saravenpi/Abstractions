//! Tests of the keys of the ship that no setting rebinds: flying home has no key of its own, it
//! is Fly home on the map.

use super::reference::SHIP_FIXED;

/// The controls page names no key to fly home in the ship: the H key is the stone key on foot.
#[test]
fn no_key_flies_home() {
    assert!(
        SHIP_FIXED
            .iter()
            .all(|(key, action)| *key != "H" && !action.contains("home"))
    );
}
