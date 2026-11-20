//! The tool this robot holds, as its recipe, sent to the server when it changes.

use abstractions_net::wire::{Act, ClientMsg};
use bevy::prelude::*;

use super::Hunted;
use super::item;
use crate::avatar::Avatar;
use crate::avatar::strike::Held;
use crate::net::link::Link;
use crate::net::state::Remote;
use crate::player::Player;
use crate::sim::PlayerId;

/// Tells the server which tool the local robot holds in its hands, or that it holds none.
pub fn holds(
    (mut link, mut hunted): (ResMut<Link>, ResMut<Hunted>),
    players: Query<&PlayerId, With<Player>>,
    avatars: Query<(&ChildOf, Option<&Held>), (With<Avatar>, Without<Remote>)>,
) {
    let mine = avatars.iter().find(|(of, _)| {
        players
            .get(of.parent())
            .is_ok_and(|id| *id == PlayerId::LOCAL)
    });
    let Some((_, held)) = mine else {
        return;
    };
    let out = held.filter(|held| held.out);
    let id = out.map(|held| held.id);
    if hunted.held == id {
        return;
    }
    hunted.held = id;
    let bytes = out.and_then(|held| item::pack(&held.recipe));
    link.send(ClientMsg::Hunt(Act::Held(bytes)));
}
