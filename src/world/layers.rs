//! Collision layers: which colliders a spatial query of the game meets.

use avian3d::prelude::LayerMask;

/// The layer of the bodies of the drawn animals near the player. The body of the player and the ship
/// meet them; blows, thrown tools and the camera pass them by, as blows reach the parts of an animal
/// by their own aim.
pub const ANIMALS: LayerMask = LayerMask(1 << 1);

/// Every layer but the one of the animals: what a query that must not stop at an animal meets.
pub const NOT_ANIMALS: LayerMask = LayerMask(!(1 << 1));
