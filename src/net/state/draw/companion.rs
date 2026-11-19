//! The companions of the other players: the shared companion look with no brain, placed from
//! the records. The companion code measures how it moves, leans and bobs it and lays its trail
//! from that motion alone; its pose fields are not shared yet, and its face keeps the picture
//! it was spawned with.

use abstractions_net::wire::CompanionBody;
use bevy::prelude::*;

use super::{Eye, Sight, drop_puppet, seen, shown, visible};
use crate::avatar::Mode;
use crate::companion::{self, Stock};
use crate::net::state::Remote;
use crate::net::state::robot::mode;
use crate::net::state::track::{Remotes, Track};
use crate::space::{Placed, Rebased};

/// The companion puppets.
type Bots<'w, 's> = Query<
    'w,
    's,
    (&'static mut Placed, &'static mut Visibility),
    (With<Remote>, With<companion::Companion>),
>;

/// What spawning a companion puppet needs.
type Spawner<'w, 's> = (Commands<'w, 's>, Stock<'w>);

/// Spawns the puppet of a companion at `placed` and returns it. It is placed from its pose,
/// not carried by the rebase.
fn spawn((commands, stock): &mut Spawner, eye: &Eye, placed: Placed) -> Entity {
    let (at, _) = seen(eye, &placed);
    let root = companion::look(commands, stock, at);
    commands.entity(root).remove::<Rebased>().insert((
        Name::new("Remote companion"),
        Remote,
        placed,
    ));
    root
}

/// Draws one companion of `track`, hidden while its player sits in the ship when `seated`.
fn draw(
    (spawner, bots): (&mut Spawner, &mut Bots),
    sight: &mut Sight,
    eye: &Eye,
    track: &mut Track<CompanionBody>,
    seated: bool,
) {
    let Some((_, placed)) = sight.locate(track, eye) else {
        return;
    };
    let Some(entity) = track.puppet else {
        track.puppet = Some(spawn(spawner, eye, placed));
        return;
    };
    let Ok((mut at, mut shown)) = bots.get_mut(entity) else {
        track.puppet = None;
        return;
    };
    *at = placed;
    let (_, far) = seen(eye, &placed);
    shown.set_if_neq(visible(far, seated));
}

/// Returns whether the robot of `track`, whose player flies a ship when `aboard`, sits in it.
fn seated(track: Option<&Track<abstractions_net::wire::RobotBody>>, aboard: bool) -> bool {
    aboard && track.is_some_and(|track| mode(track.latest.mode) == Mode::Seated)
}

/// Draws the companion of every other player at the body this player is at, and takes away
/// the puppets of the others.
pub fn companions(
    mut spawner: Spawner,
    mut sight: Sight,
    mut remotes: ResMut<Remotes>,
    mut bots: Bots,
) {
    let Some(eye) = sight.eye() else {
        return;
    };
    let Remotes {
        robots,
        ships,
        companions,
    } = &mut *remotes;
    for (player, track) in companions.iter_mut() {
        if !shown(track.frame, eye.body) {
            drop_puppet(&mut spawner.0, track);
            continue;
        }
        let sitting = seated(robots.get(player), ships.contains_key(player));
        draw((&mut spawner, &mut bots), &mut sight, &eye, track, sitting);
    }
}
