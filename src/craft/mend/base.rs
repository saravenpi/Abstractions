//! Mending at the base: free, for every part, while the robot stands within the bay.

use bevy::prelude::*;

use super::tune::{BAY, BAY_RATE};
use crate::condition::{Condition, Part};
use crate::home::Base;
use crate::player::Player;
use crate::space::{Anchor, Frame};

/// Returns true when a robot at the render position `at` is within the bay of `base`.
pub fn inside(anchor: &Anchor, base: &Base, at: Vec3) -> bool {
    anchor.frame == Frame::Surface(base.body) && anchor.to_render(base.centre).distance(at) <= BAY
}

/// Gives every part of a robot within the bay back its health, [`BAY_RATE`] a second.
pub fn tend(
    time: Res<Time>,
    (anchor, base): (Res<Anchor>, Res<Base>),
    player: Query<&Transform, With<Player>>,
    mut condition: ResMut<Condition>,
) {
    let Ok(body) = player.single() else { return };
    if condition.intact() || !inside(&anchor, &base, body.translation) {
        return;
    }
    for part in Part::ALL {
        condition.mend(part, BAY_RATE * time.delta_secs(), 1.0);
    }
}
