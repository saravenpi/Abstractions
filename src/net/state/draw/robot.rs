//! The robots of the other players: a puppet of the shared robot look each, posed from the
//! blend of its two records around now. A robot seated in its ship is hidden: the ship shows it.

use bevy::prelude::*;

use super::{Eye, Sight, drop_puppet, seen, shown, visible};
use crate::avatar::{self, AvatarPose, Mode, Stock};
use crate::net::state::Remote;
use crate::net::state::robot::blend;
use crate::net::state::track::{Remotes, Track};
use crate::screens::Screen;
use crate::space::Placed;
use abstractions_net::wire::RobotBody;

/// The robot puppets.
type Puppets<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Placed,
        &'static mut AvatarPose,
        &'static mut Visibility,
    ),
    With<Remote>,
>;

/// What spawning a robot puppet needs.
type Spawner<'w, 's> = (Commands<'w, 's>, Stock<'w>, Res<'w, avatar::Looks>);

/// Spawns the puppet of a robot at `placed` and returns it.
fn spawn((commands, stock, looks): &mut Spawner, eye: &Eye, placed: Placed) -> Entity {
    let root = avatar::puppet(commands, stock, looks, None);
    let (at, _) = seen(eye, &placed);
    let marks = (Remote, placed, at, DespawnOnEnter(Screen::Title));
    commands
        .entity(root)
        .insert((Name::new("Remote robot"), marks));
    root
}

/// Draws one robot of `track`, whose player also flies a ship when `aboard`.
fn draw(
    (spawner, puppets): (&mut Spawner, &mut Puppets),
    sight: &mut Sight,
    eye: &Eye,
    track: &mut Track<RobotBody>,
    aboard: bool,
) {
    let Some((view, placed)) = sight.locate(track, eye) else {
        return;
    };
    let Some(entity) = track.puppet else {
        track.puppet = Some(spawn(spawner, eye, placed));
        return;
    };
    let Ok((mut at, mut pose, mut shown)) = puppets.get_mut(entity) else {
        track.puppet = None;
        return;
    };
    *at = placed;
    *pose = blend(&view, &mut track.stride);
    let (_, far) = seen(eye, &placed);
    shown.set_if_neq(visible(far, aboard && pose.mode == Mode::Seated));
}

/// Draws the robot of every other player at the body this player is at, and takes away the
/// puppets of the others.
pub fn robots(
    mut spawner: Spawner,
    mut sight: Sight,
    mut remotes: ResMut<Remotes>,
    mut puppets: Puppets,
) {
    let Some(eye) = sight.eye() else {
        return;
    };
    let Remotes { robots, ships, .. } = &mut *remotes;
    for (player, track) in robots.iter_mut() {
        if !shown(track.frame, eye.body) {
            drop_puppet(&mut spawner.0, track);
            continue;
        }
        let aboard = ships.contains_key(player);
        draw(
            (&mut spawner, &mut puppets),
            &mut sight,
            &eye,
            track,
            aboard,
        );
    }
}
