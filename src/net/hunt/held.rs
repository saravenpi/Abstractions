//! The tool or firearm this robot holds, sent to the server when it changes.

use abstractions_net::wire::{Act, ClientMsg};
use bevy::prelude::*;

use super::Hunted;
use super::item;
use crate::avatar::Avatar;
use crate::avatar::strike::Held;
use crate::craft::Pack;
use crate::net::link::Link;
use crate::net::state::Remote;
use crate::player::Player;
use crate::sim::PlayerId;

/// The robots of this machine, with the hand tool each holds.
type Hands<'w, 's> =
    Query<'w, 's, (&'static ChildOf, Option<&'static Held>), (With<Avatar>, Without<Remote>)>;

/// Tells the server which tool or firearm the local robot holds in its hands, or that it holds none.
pub fn holds(
    (mut link, mut hunted): (ResMut<Link>, ResMut<Hunted>),
    pack: Option<Res<Pack>>,
    players: Query<&PlayerId, With<Player>>,
    avatars: Hands,
) {
    if let Some(gun) = pack.as_deref().and_then(Pack::held_gun) {
        let key = Some((true, gun.id));
        if hunted.held != key {
            hunted.held = key;
            link.send(ClientMsg::Hunt(Act::Held(Some(item::pack_gun(gun)))));
        }
        return;
    }
    let mine = avatars.iter().find(|(of, _)| {
        players
            .get(of.parent())
            .is_ok_and(|id| *id == PlayerId::LOCAL)
    });
    let Some((_, held)) = mine else {
        return;
    };
    let out = held.filter(|held| held.out);
    let key = out.map(|held| (false, held.id));
    if hunted.held != key {
        hunted.held = key;
        let bytes = out.and_then(|held| item::pack(&held.recipe));
        link.send(ClientMsg::Hunt(Act::Held(bytes)));
    }
}
