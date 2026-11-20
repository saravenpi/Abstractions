//! The glow of an entry around the robots and ships of the other players: the same sheath as the
//! one around the craft of this player, one on each puppet that glows, made when its glow starts
//! and gone when it dies. The colour is the one the gas of the air under this player's camera
//! gives, as the sheath of the local craft takes it.

use std::collections::HashMap;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use crate::net::state::Remote;
use crate::net::state::track::Remotes;
use crate::render::Vantage;
use crate::render::{Gas, LocalSystem, SHEATH_FROM, SHEATH_REACH, SheathMaterial, SheathUniform};
use crate::ship::HULL_RADIUS;

/// Radius in metres of the sphere that holds the body of a robot, which its sheath wraps.
const BODY_RADIUS: f32 = 0.9;

/// Marks the sheath of a puppet, a child of it.
#[derive(Component, Debug, Clone, Copy)]
pub struct Aura;

/// The sheaths made and the mesh they share.
#[derive(Default)]
pub struct Auras {
    /// The sphere every sheath is drawn on, made once.
    mesh: Option<Handle<Mesh>>,
    /// The sheath of each puppet that has one.
    made: HashMap<Entity, Entity>,
    /// The puppets that glow this frame.
    lit: Vec<Entity>,
}

/// Returns the puppets that glow, each with its glow, its radius and how high its centre is
/// above its origin: a robot is wrapped around its body, a ship around its hull.
fn hot(remotes: &Remotes) -> impl Iterator<Item = (Entity, f32, (f32, f32))> + '_ {
    let robots = remotes.robots.iter();
    let robots = robots.filter_map(|(player, t)| {
        let heat = remotes.vitals.get(player)?.heat;
        Some((t.puppet?, heat, (BODY_RADIUS, BODY_RADIUS)))
    });
    let ships = remotes.ships.values();
    let ships = ships.filter_map(|t| Some((t.puppet?, t.latest.heat, (HULL_RADIUS, 0.0))));
    robots
        .chain(ships)
        .filter(|(_, heat, _)| *heat > SHEATH_FROM)
}

/// Run condition: a puppet glows, or a sheath is left to take away.
pub fn burning(remotes: Res<Remotes>, auras: Query<(), With<Aura>>) -> bool {
    !auras.is_empty() || hot(&remotes).next().is_some()
}

/// Spawns the hidden sheath of `puppet` with its own material, on the sphere `mesh`.
fn spawn(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<SheathMaterial>,
    puppet: Entity,
) -> Entity {
    let marks = (Aura, NotShadowCaster, NoFrustumCulling, ChildOf(puppet));
    let look = (Mesh3d(mesh), MeshMaterial3d(material));
    commands
        .spawn((look, marks, Transform::default(), Visibility::Inherited))
        .id()
}

/// Writes the sheath `aura` of the puppet at `at`, glowing `heat` in `colour`: its place on the
/// puppet and the uniform of its material.
fn shape(
    (puppet, aura): (&GlobalTransform, (&mut Transform, &mut SheathMaterial)),
    (heat, (radius, lift)): (f32, (f32, f32)),
    colour: Vec3,
) {
    let (transform, material) = aura;
    *transform =
        Transform::from_translation(Vec3::Y * lift).with_scale(Vec3::splat(radius * SHEATH_REACH));
    let centre = puppet.translation() + puppet.up() * lift;
    material.sheath = SheathUniform {
        centre: centre.extend(radius),
        axis: puppet.forward().as_vec3().extend(0.0),
        glow: colour.extend(heat),
        load: Vec4::ZERO,
    };
}

/// Draws a sheath on every puppet of another player that glows, and takes away the sheath of every
/// puppet that stopped.
pub fn glows(
    mut commands: Commands,
    remotes: Res<Remotes>,
    sky: (Option<Res<Vantage>>, Option<Res<LocalSystem>>),
    (mut meshes, mut materials): (ResMut<Assets<Mesh>>, ResMut<Assets<SheathMaterial>>),
    (puppets, mut auras, mut state): (
        Query<&GlobalTransform, With<Remote>>,
        Query<(&mut Transform, &MeshMaterial3d<SheathMaterial>), With<Aura>>,
        Local<Auras>,
    ),
) {
    let gas = match sky {
        (Some(vantage), Some(local)) => local.gas(&vantage),
        _ => Gas::Unknown,
    };
    let Auras { mesh, made, lit } = &mut *state;
    lit.clear();
    for (puppet, heat, size) in hot(&remotes) {
        lit.push(puppet);
        let shared = mesh.get_or_insert_with(|| meshes.add(Sphere::new(1.0).mesh().uv(32, 16)));
        let aura = *made.entry(puppet).or_insert_with(|| {
            let material = materials.add(SheathMaterial::default());
            spawn(&mut commands, shared.clone(), material, puppet)
        });
        let (Ok(at), Ok((mut transform, handle))) = (puppets.get(puppet), auras.get_mut(aura))
        else {
            continue;
        };
        if let Some(mut material) = materials.get_mut(&handle.0) {
            shape(
                (at, (&mut *transform, &mut *material)),
                (heat, size),
                gas.color(heat),
            );
        }
    }
    retire(&mut commands, made, lit);
}

/// Takes away the sheath of every puppet that is not among `lit`.
fn retire(commands: &mut Commands, made: &mut HashMap<Entity, Entity>, lit: &[Entity]) {
    made.retain(|puppet, aura| {
        let keep = lit.contains(puppet);
        if !keep {
            commands.entity(*aura).try_despawn();
        }
        keep
    });
}
