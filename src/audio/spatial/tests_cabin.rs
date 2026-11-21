//! Tests of the cabin as the ears in the seat hear the world: through the closed hull, and in a
//! cabin that rings.

use bevy::prelude::*;

use super::air::Air;
use super::listen::Listener;
use super::spot::{Between, Origin, spot};
use super::tune::{HULL, PANEL};

/// Ears in the seat hear a source outside the cabin through the closed hull: 10 dB down and
/// nothing over its corner, and the cabin rings with it. A source inside the cabin is untouched.
#[test]
fn the_cockpit_takes_the_highs_of_the_world() {
    let (open, seat) = (
        Listener::default(),
        Listener {
            cabin: true,
            ..Listener::default()
        },
    );
    let far = Origin::At(Vec3::new(5.0, 0.0, -19.0), 1.0);
    let (out, through) = (
        spot(&open, far, Between::default()),
        spot(&seat, far, Between::default()),
    );
    assert!((through.air / out.air - HULL.1).abs() < 1.0e-5);
    assert!(through.corner <= HULL.2 && out.corner > 5.0 * HULL.2);
    assert!(through.room == 1.0 && out.room == 0.0);
    let beside = Origin::At(Vec3::new(0.6, 0.2, -0.4), 0.7);
    assert_eq!(
        spot(&seat, beside, Between::default()).air,
        spot(&open, beside, Between::default()).air
    );
}

/// Ears in the seat still hear cockpit cues in a vacuum, while sources outside the hull stay
/// silent.
#[test]
fn the_cockpit_still_sounds_in_a_vacuum() {
    let seat = Listener {
        cabin: true,
        air: Air::VACUUM,
        ..Listener::default()
    };
    let panel = Origin::At(Vec3::from_array(PANEL.0), PANEL.1);
    let inside = spot(&seat, panel, Between::default());
    assert!(inside.heard() && inside.air > 0.5);
    let far = Origin::At(Vec3::new(5.0, 0.0, -19.0), 1.0);
    let outside = spot(&seat, far, Between::default());
    assert!(!outside.heard() && outside.air == 0.0);
}
