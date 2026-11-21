//! The hands of the robots of the other players: the strike state of each puppet that holds a
//! tool is the one its records say, which the hands pose, swing, guard, aim and stow from.

use bevy::prelude::*;

use crate::avatar::strike::Strike;
use crate::net::state::Remote;
use crate::net::state::swing::unpack;
use crate::net::state::track::Remotes;

/// Sets the strike state of the puppet of each robot from its newest record.
pub fn swings(remotes: Res<Remotes>, mut hands: Query<&mut Strike, With<Remote>>) {
    for track in remotes.robots.values() {
        let Some(puppet) = track.puppet else {
            continue;
        };
        let Ok(mut strike) = hands.get_mut(puppet) else {
            continue;
        };
        let next = unpack(track.latest.swing, track.latest.phase);
        if strike.act != next.act || strike.dir != next.dir || strike.aim != next.aim {
            strike.act = next.act;
            strike.dir = next.dir;
            strike.aim = next.aim;
        }
    }
}
