//! The legs of the landing gear as parts of the ship: each one telescopes up into the hull as
//! the gear folds, and back down as it comes out.

use bevy::prelude::*;

use super::body::Ship;
use super::gear::reach;

/// One leg of the gear, a child of the ship hung from its hinge and scaled along y.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Leg;

/// Sets the length of every leg of the ship from the extension of its gear, when it changes.
/// The legs of other hulls are left to whatever draws them.
pub fn fold(ship: Single<(Entity, &Ship)>, mut legs: Query<(&ChildOf, &mut Transform), With<Leg>>) {
    let (hull, ship) = *ship;
    let length = reach(ship.gear) as f32;
    for (parent, mut leg) in &mut legs {
        if parent.parent() == hull && leg.scale.y != length {
            leg.scale.y = length;
        }
    }
}
