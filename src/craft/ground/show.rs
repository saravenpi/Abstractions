//! Drawing: every thing within [`DRAW`] metres of the robot lies as a real object at its place, the
//! blocks of its tool or the cube of its heap, as the icons of the pack draw them, with a collider,
//! resting on the ground. What lies farther stays stored and is not drawn.

use std::f64::consts::TAU;

use abstractions_net::wire::Lying;
use avian3d::prelude::*;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::tune::{DRAW, HEAP_SIDE, THIN};
use super::{Ground, Thing, codec};
use crate::avatar::strike::{Kit, blocks, kit, tool};
use crate::craft::matter::Stuff;
use crate::player::Player;
use crate::screens::Screen;
use crate::space::{Anchor, Frame, Placed};
use crate::universe::BodyId;

/// A thing of the ground drawn in the world.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Lies {
    /// Its id in the ground.
    pub id: u64,
    /// Its mass in kilograms, as drawn.
    pub mass: f32,
    /// The plain word for it.
    pub name: &'static str,
}

/// Returns the size of `thing` as it lies, in metres across, up and along, and for a tool the
/// shift along its shaft that centres its blocks. Nothing for a tool that lost a material.
pub(super) fn extent(thing: &Thing) -> Option<(Vec3, f32)> {
    let item = match thing {
        Thing::Tool(item) => item,
        Thing::Gun(gun) => {
            let size = match gun.kind {
                crate::craft::gun::GunKind::FlintlockPistol => Vec3::new(0.06, 0.14, 0.34),
                crate::craft::gun::GunKind::MineralRifle => Vec3::new(0.07, 0.16, 0.92),
            };
            return Some((size, 0.0));
        }
        Thing::Heap(stack) => {
            let volume = stack.mass / stack.material.density.max(1.0);
            let side = volume.cbrt().clamp(HEAP_SIDE.0, HEAP_SIDE.1);
            return Some((Vec3::splat(side), 0.0));
        }
    };
    let parts = blocks(&item.build()?, 0.0);
    let ends = |&(at, size, _): &(Vec3, Vec3, usize)| (at - size * 0.5, at + size * 0.5);
    let (lo, hi) = parts
        .iter()
        .map(ends)
        .fold((Vec3::MAX, Vec3::MIN), |(lo, hi), (low, high)| {
            (lo.min(low), hi.max(high))
        });
    let size = (hi - lo).max(Vec3::splat(THIN));
    Some((Vec3::new(size.x, size.z, size.y), -(lo.y + hi.y) * 0.5))
}

/// Returns how `lying` is turned in the frame of its body: upright on the ground, spun about the
/// local up by an angle its id gives, so that things put down together do not all line up.
fn turn(anchor: &Anchor, lying: &Lying) -> DQuat {
    let up = match anchor.flat {
        true => DVec3::Y,
        false => DVec3::from(lying.at).normalize_or(DVec3::Y),
    };
    let spin = (lying.id as f64 * 2.399_963).rem_euclid(TAU);
    DQuat::from_rotation_arc(DVec3::Y, up) * DQuat::from_rotation_y(spin)
}

/// Spawns `thing`, the thing of `lying`, drawn from `kit`, resting on the ground where it lies.
fn raise(commands: &mut Commands, kit: &Kit, (lying, thing): (&Lying, Thing), anchor: &Anchor) {
    let Some((size, shift)) = extent(&thing) else {
        return;
    };
    let rot = turn(anchor, lying);
    let pos = DVec3::from(lying.at) + rot * DVec3::Y * f64::from(size.y * 0.5);
    let shown = Transform {
        translation: anchor.to_render(pos),
        rotation: anchor.rotation_to_render(rot),
        scale: Vec3::ONE,
    };
    let lies = Lies {
        id: lying.id,
        mass: lying.mass,
        name: codec::word(&thing),
    };
    let solid = (RigidBody::Static, Collider::cuboid(size.x, size.y, size.z));
    let marks = (
        lies,
        Name::new("On the ground"),
        DespawnOnExit(Screen::Gameplay),
    );
    let placed = (Placed { pos, rot }, shown, Visibility::default());
    let root = commands.spawn((marks, placed, solid)).id();
    dress(commands, kit, root, (thing, size, shift));
}

/// Draws `thing` under `root`, `size` metres as it lies: a tool lies flat along its shaft, its
/// blocks moved `shift` along it to centre them; a heap is a cube of its volume.
fn dress(
    commands: &mut Commands,
    kit: &Kit,
    root: Entity,
    (thing, size, shift): (Thing, Vec3, f32),
) {
    match thing {
        Thing::Tool(item) => {
            let Some(build) = item.build() else { return };
            let drawn = tool(commands, kit, &build, Some(root), shift);
            let flat = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            commands
                .entity(drawn)
                .insert(Transform::from_rotation(flat));
        }
        Thing::Gun(_) => {
            let look = (
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.paint[1].clone()),
            );
            commands.spawn((look, Transform::from_scale(size), ChildOf(root)));
        }
        Thing::Heap(stack) => {
            let paint = kit.paint[usize::from(matches!(stack.stuff, Stuff::Mineral(_)))].clone();
            let look = (Mesh3d(kit.cube.clone()), MeshMaterial3d(paint));
            commands.spawn((look, Transform::from_scale(size), ChildOf(root)));
        }
    }
}

/// Returns true when `lying` lies on the body of `here` within `reach` metres of the robot there.
fn near(lying: &Lying, here: Option<(BodyId, DVec3)>, reach: f64) -> bool {
    here.is_some_and(|(body, me)| lying.body == body && DVec3::from(lying.at).distance(me) < reach)
}

/// The kit of drawn tools, when it is made, and the stores it is made in.
type Stores<'w> = (
    Option<Res<'w, Kit>>,
    ResMut<'w, Assets<Mesh>>,
    ResMut<'w, Assets<StandardMaterial>>,
);

/// The robot, and what is drawn on the ground already.
type Seen<'w, 's> = (
    Query<'w, 's, &'static Transform, With<Player>>,
    Query<'w, 's, (Entity, &'static Lies)>,
);

/// Draws what lies within [`DRAW`] metres of the robot and is not drawn yet, and takes away what
/// was taken, changed or left behind: farther than a quarter more, or on another body.
pub(super) fn show(
    mut commands: Commands,
    (ground, anchor): (Res<Ground>, Res<Anchor>),
    (shown, mut meshes, mut materials): Stores,
    (robot, drawn): Seen,
    mut ids: Local<Vec<u64>>,
) {
    let here = match (anchor.frame, robot.single()) {
        (Frame::Surface(body), Ok(at)) => Some((body, anchor.to_frame(at.translation))),
        _ => None,
    };
    ids.clear();
    for (entity, lies) in &drawn {
        let same = |l: &&Lying| (l.mass - lies.mass).abs() < 1e-6 && near(l, here, DRAW * 1.25);
        match ground.pile.get(lies.id).filter(same) {
            Some(_) => ids.push(lies.id),
            None => commands.entity(entity).despawn(),
        }
    }
    let fresh = ground.pile.lying.iter();
    let fresh = fresh.filter(|l| near(l, here, DRAW) && !ids.contains(&l.id));
    let mut made = shown.map(|kit| (*kit).clone());
    let had = made.is_some();
    for lying in fresh {
        let Some(thing) = codec::thing(lying) else {
            continue;
        };
        let paints = made.get_or_insert_with(|| kit(&mut meshes, &mut materials));
        raise(&mut commands, paints, (lying, thing), &anchor);
    }
    if let Some(made) = made.filter(|_| !had) {
        commands.insert_resource(made);
    }
}
