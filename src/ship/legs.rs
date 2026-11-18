//! The legs of the landing gear as parts of the ship: each one telescopes up into the hull as
//! the gear folds, and back down as it comes out.

use bevy::prelude::*;

use super::body::Ship;
use super::gear::reach;

/// One leg of the gear, a child of the ship hung from its hinge and scaled along y.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Leg;

/// Sets the length of every leg from the extension of the gear, when it changes.
pub fn fold(ship: Single<&Ship>, mut legs: Query<&mut Transform, With<Leg>>) {
    let length = reach(ship.gear) as f32;
    for mut leg in &mut legs {
        if leg.scale.y != length {
            leg.scale.y = length;
        }
    }
}
