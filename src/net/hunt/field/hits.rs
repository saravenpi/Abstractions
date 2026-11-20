//! The blows of the others on the robot of this player: each arrives as the blow of an animal does,
//! in render space, so the harm, the shove and the stagger run the same code.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use crate::avatar::Own;
use crate::avatar::blow::Blow;
use crate::net::hunt::hooks::Echoes;
use crate::player::Player;
use crate::space::{Anchor, up};

/// Returns the turn from the axes of the network, whose Y is the local up, to render space at `at`.
pub fn upright(anchor: &Anchor, at: Vec3) -> Quat {
    let normal = up(anchor, at).as_dvec3().normalize_or(DVec3::Y);
    DQuat::from_rotation_arc(DVec3::Y, normal).as_quat()
}

/// Lets the blows the others landed on this robot hurt and push it.
pub fn play(
    mut echoes: ResMut<Echoes>,
    (anchor, own): (Res<Anchor>, Own),
    players: Query<&Transform, With<Player>>,
    mut blows: MessageWriter<Blow>,
) {
    let told = std::mem::take(&mut echoes.blows);
    let Some(at) = own.body().and_then(|(player, _)| players.get(player).ok()) else {
        return;
    };
    let turn = upright(&anchor, at.translation);
    for (_, blow) in told {
        blows.write(Blow {
            at: at.translation + turn * Vec3::from(blow.at),
            joules: blow.joules,
            cut: blow.cut,
            dir: (turn * Vec3::from(blow.dir)).normalize_or_zero(),
            momentum: blow.momentum,
        });
    }
}
