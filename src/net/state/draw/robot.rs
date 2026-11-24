//! The robots of the other players: a puppet of the shared robot look each, posed from the
//! blend of its two records around now, laying trails of its own. A robot seated in its ship
//! sits in the seat of the puppet of that ship, hands on its controls, carried by the hull
//! instead of placed; it stands up where its records put it. Seated in a ship not drawn here,
//! it is hidden.

use bevy::prelude::*;

use super::ship::Seat;
use super::{Eye, Sight, drop_puppet, seen, shown, visible};
use crate::avatar::{self, AvatarPose, Marked, Mode, Stock};
use crate::gear::Wake;
use crate::net::state::Remote;
use crate::net::state::robot::blend;
use crate::net::state::track::{Remotes, Track};
use crate::screens::Screen;
use crate::ship::cockpit::Cockpit;
use crate::space::Placed;
use abstractions_net::wire::RobotBody;

/// The robot puppets.
type Puppets<'w, 's> = Query<
    'w,
    's,
    (
        Option<&'static mut Placed>,
        &'static mut AvatarPose,
        &'static mut Visibility,
        Has<ChildOf>,
    ),
    With<Remote>,
>;

/// The seats of the ship puppets, with the cockpit of each hull.
type Seats<'w, 's> = Query<'w, 's, (&'static Seat, &'static mut Cockpit)>;

/// The ship a robot may sit in: the puppet of its hull when one is drawn, and whether its
/// player flies a ship at all.
type Ride = (Option<Entity>, bool);

/// What spawning a robot puppet needs.
type Spawner<'w, 's> = (Commands<'w, 's>, Stock<'w>, Res<'w, avatar::Looks>);

/// Spawns the puppet of a robot at `placed` and returns it.
fn spawn((commands, stock, looks): &mut Spawner, eye: &Eye, placed: Placed) -> Entity {
    let root = avatar::puppet(commands, stock, looks, None);
    let (at, _) = seen(eye, &placed);
    let marks = (Remote, placed, at, DespawnOnEnter(Screen::Title));
    let wake = (Marked::default(), Wake::default());
    commands
        .entity(root)
        .insert((Name::new("Remote robot"), marks, wake));
    root
}

/// Sits the robot `entity` in `seat`, the seat of the hull whose cockpit is `cockpit`. Its
/// pose is no longer placed: the hull carries it.
fn sit(
    commands: &mut Commands,
    entity: Entity,
    (seat, cockpit): (&Seat, &mut Cockpit),
    sitting: bool,
) {
    if !sitting {
        let at = (ChildOf(seat.0), Transform::IDENTITY);
        commands.entity(entity).remove::<Placed>().insert(at);
    }
    if cockpit.pilot != Some(seat.0) {
        cockpit.pilot = Some(seat.0);
    }
}

/// Stands the robot `entity` up at `placed`, out of the seat it sat in when `sitting`.
fn stand(
    commands: &mut Commands,
    entity: Entity,
    (at, sitting): (Option<Mut<Placed>>, bool),
    placed: Placed,
) {
    match at {
        Some(mut at) if !sitting => *at = placed,
        _ => {
            commands.entity(entity).remove::<ChildOf>().insert(placed);
        }
    }
}

/// Draws one robot of `track`, which may sit in the ship of `ride`.
fn draw(
    (spawner, puppets, seats): (&mut Spawner, &mut Puppets, &mut Seats),
    sight: &mut Sight,
    eye: &Eye,
    track: &mut Track<RobotBody>,
    (hull, aboard): Ride,
) {
    let Some((view, placed)) = sight.locate(track, eye) else {
        return;
    };
    let Some(entity) = track.puppet else {
        track.puppet = Some(spawn(spawner, eye, placed));
        return;
    };
    let Ok((at, mut pose, mut shown, sitting)) = puppets.get_mut(entity) else {
        track.puppet = None;
        return;
    };
    *pose = blend(&view, &mut track.stride);
    let seated = pose.mode == Mode::Seated;
    let mut ship = hull.and_then(|hull| seats.get_mut(hull).ok());
    match ship.as_mut().filter(|_| seated) {
        Some((seat, cockpit)) => sit(&mut spawner.0, entity, (seat, cockpit), sitting),
        None => {
            stand(&mut spawner.0, entity, (at, sitting), placed);
            if let Some((_, cockpit)) = ship.as_mut().filter(|(_, c)| c.pilot.is_some()) {
                cockpit.pilot = None;
            }
        }
    }
    let (_, far) = seen(eye, &placed);
    shown.set_if_neq(visible(far, aboard && seated && ship.is_none()));
}

/// Draws the robot of every other player at the body this player is at, and takes away the
/// puppets of the others.
pub fn robots(
    mut spawner: Spawner,
    mut sight: Sight,
    mut remotes: ResMut<Remotes>,
    (mut puppets, mut seats): (Puppets, Seats),
) {
    let Some(eye) = sight.eye() else {
        return;
    };
    let Remotes {
        robots,
        ships,
        pilots,
        ..
    } = &mut *remotes;
    for (player, track) in robots.iter_mut() {
        if !shown(track.frame, eye.body) {
            drop_puppet(&mut spawner.0, track);
            continue;
        }
        let ship_id = pilots
            .iter()
            .find_map(|(ship, pilot)| (*pilot == *player).then_some(*ship))
            .unwrap_or(*player);
        let ship = ships.get(&ship_id);
        let ride = (ship.and_then(|ship| ship.puppet), ship.is_some());
        let parts = (&mut spawner, &mut puppets, &mut seats);
        draw(parts, &mut sight, &eye, track, ride);
    }
}
