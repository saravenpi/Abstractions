//! Spears that stay the robot's. A thrown spear is a real thing in the world: the pack keeps a
//! record of where each of its tools came to rest. When the spear of a record is gone (the animal
//! it stood in left the world, the world was closed and opened again), the tool is put back in the
//! world as a spear that falls to the ground where the record says, to be picked up like any other.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::flight::{Rest, Spear};
use super::held::Held;
use super::input::Robot;
use super::show::{Kit, kit, tool};
use super::tune::{STRAY_LIFT, STRAY_NEAR, STRAY_SLACK};
use crate::craft::build::Place;
use crate::craft::{Out, Pack};
use crate::player::Player;
use crate::screens::Screen;
use crate::space::{Anchor, Frame, Rebased};
use crate::universe::BodyId;
use crate::world::Terrain;

/// Run condition: a tool of the pack is out in the world.
pub fn astray(pack: Option<Res<Pack>>) -> bool {
    pack.is_some_and(|pack| !pack.out.is_empty())
}

/// Notes in the pack where every spear lent from it rests, once it moved more than
/// [`STRAY_SLACK`]: in the ground, on it, or in an animal that carries it about.
pub fn note(
    anchor: Res<Anchor>,
    mut pack: ResMut<Pack>,
    spears: Query<(&Spear, &GlobalTransform)>,
) {
    let Frame::Surface(body) = anchor.frame else {
        return;
    };
    for (spear, placed) in &spears {
        let Some((item, _)) = spear.lent.as_ref().filter(|_| spear.rest != Rest::Flying) else {
            continue;
        };
        let at = anchor.to_frame(placed.translation());
        let moved = pack
            .bypass_change_detection()
            .rest(item.id, (body, at), STRAY_SLACK);
        if moved {
            pack.set_changed();
        }
    }
}

/// Returns the render position the tool of `out` falls from, when it rests on `body` within
/// [`STRAY_NEAR`] of a robot at `robot`: [`STRAY_LIFT`] above its record, so that a spear that
/// stood in the ground does not start under it. A tool that never came to rest falls at the robot.
pub fn above(out: &Out, anchor: &Anchor, body: BodyId, robot: Vec3) -> Option<Vec3> {
    let Some(on) = out.body else {
        return Some(robot);
    };
    let frame = DVec3::from(out.at);
    let up = (anchor.rotation * frame.normalize_or(DVec3::Y)).as_vec3();
    let at = anchor.to_render(frame) + up * STRAY_LIFT;
    (on == body && at.distance(robot) < STRAY_NEAR).then_some(at)
}

/// Returns the spear the tool of `out` makes at `place`, in the air and at rest, thrown by the
/// player `by.0` from the body `by.1`, starting at `at`.
pub fn fallen(out: &Out, place: Place, by: (Entity, Entity), at: Vec3) -> Option<Spear> {
    let item = Held::new(out.item.recipe.clone(), out.item.build()?, place);
    Some(Spear {
        item: Held {
            id: out.item.id,
            ..item
        },
        velocity: Vec3::ZERO,
        rest: Rest::Flying,
        thrower: by.0,
        body: by.1,
        was: at,
        lent: Some((out.item.clone(), out.slot)),
    })
}

/// Spawns `spear`, drawn from `kit`, at the render position `at`.
fn raise(commands: &mut Commands, kit: &Kit, spear: Spear, at: Vec3) {
    let reach = spear.item.build.reach();
    let drawn = tool(commands, kit, &spear.item.build, None, -reach);
    let marks = (Rebased, DespawnOnEnter(Screen::Title));
    commands
        .entity(drawn)
        .insert((spear, Transform::from_translation(at), marks));
}

/// The stores the kit of drawn tools is made in, and the kit when it is made.
type Stores<'w> = (
    Option<Res<'w, Kit>>,
    ResMut<'w, Assets<Mesh>>,
    ResMut<'w, Assets<StandardMaterial>>,
);

/// Puts every tool of the pack that is out and has no spear back in the world, near the local
/// robot: a spear that falls to the ground from where the tool last rested.
pub fn ground(
    mut commands: Commands,
    (anchor, terrain, pack): (Res<Anchor>, Res<Terrain>, Res<Pack>),
    (shown, mut meshes, mut materials): Stores,
    (robot, players): (Robot, Query<&Transform, With<Player>>),
    spears: Query<&Spear>,
) {
    let (Some(by), Frame::Surface(body)) = (robot.pair(), anchor.frame) else {
        return;
    };
    let Ok(stands) = players.get(by.1) else {
        return;
    };
    let flown = |id: u32| {
        spears
            .iter()
            .any(|s| s.lent.as_ref().is_some_and(|l| l.0.id == id))
    };
    let place = Place::on(terrain.globe().gravity(), terrain.globe().record());
    let mut shown = shown.map(|kit| (*kit).clone());
    let had = shown.is_some();
    for out in pack.out.iter().filter(|out| !flown(out.item.id)) {
        let at = above(out, &anchor, body, stands.translation);
        let Some((at, spear)) = at.and_then(|at| Some((at, fallen(out, place, (by.1, by.0), at)?)))
        else {
            continue;
        };
        let made = shown.get_or_insert_with(|| kit(&mut meshes, &mut materials));
        raise(&mut commands, made, spear, at);
    }
    if let Some(made) = shown.filter(|_| !had) {
        commands.insert_resource(made);
    }
}
