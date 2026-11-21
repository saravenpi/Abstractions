//! Letting go: a drawn javelin leaves the hand toward the point the crosshair picked, its tip on the
//! line from the hand to that point, at the launch speed of the build times the square root of the
//! draw, with a whoosh from the hand and the whip of the arm after it.

use bevy::prelude::*;

use super::super::Avatar;
use super::act::Strike;
use super::flight::{Rest, Spear};
use super::held::Held;
use super::hurt::steadied;
use super::input::Loose;
use super::javelin::tune::THROW_SIGHT;
use super::javelin::{Release, launch};
use super::show::{Kit, tool};
use crate::audio::Whoosh;
use crate::craft::Pack;
use crate::player::Player;
use crate::screens::Screen;
use crate::space::Rebased;

/// Returns `held` flying from `point` at `velocity`, thrown by the player `by.0` from the body `by.1`.
fn launched(held: &Held, velocity: Vec3, by: (Entity, Entity), point: Vec3) -> Spear {
    Spear {
        item: Held {
            out: true,
            ..held.clone()
        },
        velocity,
        rest: Rest::Flying,
        thrower: by.0,
        body: by.1,
        was: point,
        lent: None,
    }
}

/// Returns where the javelin `held` by `strike` starts, in the frame of the body, the point it is
/// thrown at and the unit way it leaves along, shaken by hurt arms by the luck of `key`.
fn aimed(held: &Held, strike: &Strike, key: u32) -> (Vec3, Vec3, Vec3) {
    let grip = strike.blade[0][0];
    let at = strike.aim.unwrap_or(grip + Vec3::NEG_Z * THROW_SIGHT);
    let (tip, way) = launch(grip, at, held.build.reach());
    (tip, at, steadied(way, strike.hurt.steady, key))
}

/// What a throw reads: the players, the kit that draws the javelin and the clock.
type Aim<'w, 's> = (
    Query<'w, 's, &'static Transform, With<Player>>,
    Option<Res<'w, Kit>>,
    Res<'w, Time>,
);

/// Throws the javelin of every body that let one go, from its hand toward the point it aimed at.
pub fn loose(
    mut commands: Commands,
    mut loosed: MessageReader<Loose>,
    bodies: Query<(&Held, &Strike, &ChildOf), With<Avatar>>,
    (players, kit, time): Aim,
    (mut pack, mut whoosh): (Option<ResMut<Pack>>, MessageWriter<Whoosh>),
) {
    let Some(kit) = kit else { return };
    for loose in loosed.read() {
        let Ok((held, strike, of)) = bodies.get(loose.body) else {
            continue;
        };
        let Ok(player) = players.get(of.parent()) else {
            continue;
        };
        let (tip, at, way) = aimed(held, strike, (time.elapsed_secs() * 1000.0) as u32);
        let (point, way) = (player.transform_point(tip), player.rotation * way);
        let speed = held.throw.launch * loose.share.sqrt() * strike.hurt.swing;
        let mut spear = launched(held, way * speed, (of.parent(), loose.body), point);
        spear.lent = pack.as_mut().and_then(|pack| pack.lend(held.id));
        let drawn = tool(&mut commands, &kit, &held.build, None, -held.build.reach());
        let place =
            Transform::from_translation(point).with_rotation(Quat::from_rotation_arc(Vec3::Y, way));
        let marks = (Rebased, DespawnOnEnter(Screen::Title));
        commands.entity(drawn).insert((spear, place, marks));
        let (share, age) = (loose.share, 0.0);
        commands
            .entity(loose.body)
            .remove::<Held>()
            .insert(Release { at, share, age });
        let (at, inertia) = (point, held.swing.inertia);
        whoosh.write(Whoosh { at, speed, inertia });
    }
}
