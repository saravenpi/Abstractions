//! The tool drawn in the hands: plain blocks sized from the build, the grip in the right hand and
//! the shaft along the axis of the strike, or slung over the back while stowed.

use bevy::prelude::*;

use super::super::animate::Anim;
use super::super::own::Own;
use super::super::rig::{Seg, globals};
use super::super::show::Shown;
use super::act::{Act, Strike};
use super::feel::tune::{SLING, SLING_AXIS};
use super::held::Held;
use super::layer::Layer;
use super::shape::parts;
use super::shape_mesh::Meshes;
use super::tune::HAND_GRIP;
use crate::craft::build::Build;

/// Colour of the head of a drawn tool, as sRGB: the trail of a swing takes a share of it.
pub const HEAD: [f32; 3] = [0.32, 0.32, 0.34];

/// The meshes and the materials every drawn tool shares.
#[derive(Resource, Debug, Clone)]
pub struct Kit {
    /// A unit cube, kept for ground heaps and shared cuboids.
    pub cube: Handle<Mesh>,
    /// Unit procedural meshes for every tool figure.
    pub parts: Meshes,
    /// The shaft, the head and the dark binding or crossguard.
    pub paint: [Handle<StandardMaterial>; 3],
}

/// The drawn tool of a body, on its root.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wielded(pub Entity);

/// Returns the kit, made once.
pub fn kit(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Kit {
    let paint = |colour: Color, roughness: f32| StandardMaterial {
        base_color: colour,
        perceptual_roughness: roughness,
        ..default()
    };
    let parts = Meshes::new(meshes);
    Kit {
        cube: parts.cube.clone(),
        parts,
        paint: [
            materials.add(paint(Color::srgb(0.55, 0.48, 0.38), 0.9)),
            materials.add(paint(Color::srgb(HEAD[0], HEAD[1], HEAD[2]), 0.55)),
            materials.add(paint(Color::srgb(0.24, 0.18, 0.14), 0.85)),
        ],
    }
}

/// Returns the axis-aligned extent boxes of `build` along +Y from the hand, shifted by `shift`.
pub fn blocks(build: &Build, shift: f32) -> Vec<(Vec3, Vec3, usize)> {
    parts(build, shift)
        .into_iter()
        .map(|part| (part.at, part.size, part.paint.min(1)))
        .collect()
}

/// Spawns the drawn tool of `build` under `parent` and returns its root, its parts shifted by
/// `shift` along it.
pub fn tool(
    commands: &mut Commands,
    kit: &Kit,
    build: &Build,
    parent: Option<Entity>,
    shift: f32,
) -> Entity {
    let root = commands
        .spawn((
            Name::new("Tool"),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    for part in parts(build, shift) {
        let place = Transform::from_translation(part.at)
            .with_rotation(part.rot)
            .with_scale(part.size.max(Vec3::splat(1e-3)));
        let look = (
            Mesh3d(kit.parts.get(part.mesh).clone()),
            MeshMaterial3d(kit.paint[part.paint].clone()),
        );
        commands.spawn((look, place, ChildOf(root)));
    }
    if let Some(parent) = parent {
        commands.entity(root).insert(ChildOf(parent));
    }
    root
}

/// The drawn tool's kit and the stores of meshes and materials it is built in.
type Stores<'w> = (
    Option<Res<'w, Kit>>,
    ResMut<'w, Assets<Mesh>>,
    ResMut<'w, Assets<StandardMaterial>>,
);

/// Draws the tool of every body whose held item is new or changed, in place of the one drawn
/// before, and takes the drawn tool away from every body whose hands are empty.
pub fn wield(
    mut commands: Commands,
    (kit, mut meshes, mut materials): Stores,
    held: Query<(Entity, &Held, Option<&Wielded>), Changed<Held>>,
    empty: Query<(Entity, &Wielded), Without<Held>>,
) {
    for (body, wielded) in &empty {
        commands.entity(wielded.0).despawn();
        commands.entity(body).remove::<Wielded>();
    }
    if held.is_empty() {
        return;
    }
    let kit = kit.map_or_else(
        || kit_new(&mut meshes, &mut materials, &mut commands),
        |kit| kit.clone(),
    );
    for (body, item, before) in &held {
        if let Some(before) = before {
            commands.entity(before.0).despawn();
        }
        let drawn = tool(&mut commands, &kit, &item.build, Some(body), 0.0);
        commands.entity(body).insert(Wielded(drawn));
    }
}

/// Returns a new kit and keeps it as a resource.
fn kit_new(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
) -> Kit {
    let made = kit(meshes, materials);
    commands.insert_resource(made.clone());
    made
}

/// Returns where the grip of a tool is and which way it points in the frame of the body, from the
/// bones `bones` of the body: in the right hand along the axis of the `layer` while `out`, else
/// slung over the back.
fn grip(bones: &[Transform], layer: &Layer, out: bool) -> (Vec3, Vec3) {
    let (torso, hand) = (bones[Seg::Torso as usize], bones[Seg::HandR as usize]);
    match out {
        true => (
            hand.transform_point(Vec3::new(0.0, -HAND_GRIP, 0.0)),
            layer
                .aim
                .unwrap_or(torso.rotation * layer.axis)
                .normalize_or(Vec3::NEG_Z),
        ),
        false => (
            torso.transform_point(Vec3::from_array(SLING)),
            (torso.rotation * Vec3::from_array(SLING_AXIS)).normalize_or(Vec3::Y),
        ),
    }
}

/// Returns the turn that lays the shaft of a tool along `axis` and its edge, local +X, along the
/// way the tip `moved`, so a swung axe, hammer or knife leads with its cutting side. A tool that
/// does not move, is stowed or moves along its shaft keeps the shortest turn onto `axis`.
fn edge_up(axis: Vec3, moved: Vec3, out: bool) -> Quat {
    let lead = moved.reject_from(axis);
    if !out || lead.length() < 0.015 {
        return Quat::from_rotation_arc(Vec3::Y, axis);
    }
    let x = lead.normalize();
    Quat::from_mat3(&Mat3::from_cols(x, axis, x.cross(axis)))
}

/// Puts the drawn tool of every body in its right hand along the axis of its strike, or slung over
/// its back while stowed, and keeps the grip and the tip of this frame and the last in its strike.
/// A tool slung on the local body is hidden in first person, where it would fill the view.
pub fn place(
    mut bodies: Query<(Entity, &Anim, &mut Strike, &Held, &Wielded, &Layer)>,
    mut tools: Query<(&mut Transform, &mut Visibility)>,
    (shown, own): (Res<Shown>, Own),
) {
    let mine = own.body().map(|(_, body)| body);
    for (body, anim, mut strike, held, wielded, layer) in &mut bodies {
        let bones = globals(&anim.pose);
        let (grip, axis) = grip(&bones, layer, held.out);
        strike.blade = [[grip, grip + axis * held.build.reach()], strike.blade[0]];
        let Ok((mut transform, mut drawn)) = tools.get_mut(wielded.0) else {
            continue;
        };
        let moved = match strike.act {
            Act::Windup(_) => strike.blade[1][1] - strike.blade[0][1],
            Act::Recover(_) => Vec3::ZERO,
            _ => strike.blade[0][1] - strike.blade[1][1],
        };
        *transform =
            Transform::from_translation(grip).with_rotation(edge_up(axis, moved, held.out));
        let hidden = !held.out && mine == Some(body) && *shown != Shown::Full;
        drawn.set_if_neq(match hidden {
            false => Visibility::Inherited,
            true => Visibility::Hidden,
        });
    }
}
