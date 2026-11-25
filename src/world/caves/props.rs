//! Mineable underground cave mineral crystal and ore boulder nodes sampled from deep planet veins.

use std::f32::consts::TAU;

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::plan::CavePlan;
use crate::universe::hash::{Rng, child};
use crate::world::mesh::Frame;
use crate::world::rocks::{Mineral, Rock, RockId, RockTaken, Taken, mined_vein};
use crate::world::{Globe, NOT_ANIMALS};

/// Salt for deterministic cave mineral [`RockId`] values.
const NODE_SALT: u64 = 0x04E5_CA7E_9911_3301;

/// One quarryable underground mineral boulder or crystal node inside a cave.
#[derive(Component, Debug, Clone, Copy)]
pub struct CaveMineralNode {
    /// Quarryable rock record compatible with [`Taken`] and [`RockTaken`].
    pub rock: Rock,
    /// Deep underground mineral yielded by this node.
    pub mineral: Mineral,
    /// Original full scale before quarrying shrinkage.
    pub base_scale: f32,
}

/// Returns a distinct sRGB colour for `mineral` so cave crystal veins read clearly underground.
pub fn mineral_rgb(mineral: Mineral) -> [f32; 3] {
    match mineral {
        Mineral::Amethyst => [0.68, 0.34, 0.96],
        Mineral::Corundum => [0.94, 0.18, 0.34],
        Mineral::Cryolite => [0.66, 0.92, 0.98],
        Mineral::Cobaltite => [0.22, 0.44, 0.95],
        Mineral::Wolframite => [0.42, 0.46, 0.54],
        Mineral::Chromite => [0.56, 0.62, 0.66],
        Mineral::Osmium => [0.62, 0.74, 0.88],
        Mineral::Moissanite => [0.22, 0.92, 0.72],
        Mineral::Lonsdaleite => [0.98, 0.84, 0.32],
        Mineral::Ilmenite => [0.34, 0.32, 0.40],
        Mineral::Cinnabar => [0.92, 0.24, 0.18],
        Mineral::Copper => [0.88, 0.46, 0.24],
        Mineral::Pyrite => [0.86, 0.76, 0.32],
        Mineral::Malachite => [0.20, 0.78, 0.46],
        Mineral::Jade => [0.28, 0.82, 0.54],
        Mineral::Quartz => [0.88, 0.90, 0.94],
        Mineral::Brimstone => [0.95, 0.88, 0.22],
        _ => [0.52, 0.56, 0.62],
    }
}

/// Mutable vertex attribute buffers for procedural crystal node meshes.
type RawBufs<'a> = (
    &'a mut Vec<[f32; 3]>,
    &'a mut Vec<[f32; 3]>,
    &'a mut Vec<[f32; 4]>,
);

/// Appends one pointed 5-sided crystal spire to raw mesh buffers.
fn push_spire(
    (pos, norm, col): RawBufs<'_>,
    (ang, tilt, h, r): (f32, f32, f32, f32),
    rgba: [f32; 4],
) {
    let tip = Vec3::new(ang.cos() * tilt * h, h, ang.sin() * tilt * h);
    let ring: [Vec3; 5] = std::array::from_fn(|k| {
        let a = TAU * k as f32 / 5.0;
        Vec3::new(
            a.cos() * r + ang.cos() * 0.18,
            0.0,
            a.sin() * r + ang.sin() * 0.18,
        )
    });
    for k in 0..5 {
        let (a, b) = (ring[k], ring[(k + 1) % 5]);
        let n = (b - a).cross(tip - a).normalize_or(Vec3::Y).to_array();
        pos.extend([a.to_array(), b.to_array(), tip.to_array()]);
        norm.extend([n; 3]);
        col.extend([rgba; 3]);
    }
}

/// Builds a faceted 3-spire crystal/ore cluster mesh for `mineral`.
fn build_node_mesh(mineral: Mineral, seed: u64) -> Mesh {
    let mut rng = Rng::new(seed);
    let rgb = mineral_rgb(mineral);
    let (mut pos, mut norm, mut col) = (Vec::new(), Vec::new(), Vec::new());
    for spire in 0..3 {
        let ang = TAU * spire as f32 / 3.0 + rng.unit() as f32 * 0.5;
        let params = if spire == 0 {
            (ang, 0.08, 1.15, 0.42)
        } else {
            (ang, 0.38, 0.72 + rng.unit() as f32 * 0.32, 0.28)
        };
        let s = 0.85 + spire as f32 * 0.08;
        let rgba = LinearRgba::from(Color::srgb(rgb[0] * s, rgb[1] * s, rgb[2] * s)).to_f32_array();
        push_spire((&mut pos, &mut norm, &mut col), params, rgba);
    }
    let count = pos.len() as u32;
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, norm)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, col)
    .with_inserted_indices(Indices::U32((0..count).collect()))
}

/// Computes 4 to 8 deterministic `(local_pos, depth)` sites inside the main chamber and alcoves.
fn node_spots(plan: &CavePlan) -> Vec<(Vec3, f32)> {
    let mut rng = Rng::new(child(plan.seed, 0x70DE));
    let count = 4 + rng.below(5) as usize;
    let mut spots = Vec::with_capacity(count);
    for k in 0..count {
        if k < plan.alcoves.len()
            && let Some(tip) = plan.alcoves[k].spine.last()
        {
            spots.push((
                Vec3::new(tip.pos.x, tip.pos.y - tip.ry * 0.40, tip.pos.z),
                6.5 + rng.unit() as f32 * 2.5,
            ));
        } else {
            let a = TAU * k as f32 / count as f32 + rng.unit() as f32 * 0.4;
            let dist = plan.chamber_radius * (0.58 + rng.unit() as f32 * 0.28);
            let p = Vec3::new(
                plan.chamber_center.x + a.cos() * dist,
                plan.chamber_floor_y,
                plan.chamber_center.z + a.sin() * dist,
            );
            spots.push((p, 4.0 + rng.unit() as f32 * 4.8));
        }
    }
    spots
}

/// Samples the planet's deep underground vein at the cavern chamber for crystal biome accent colour.
pub fn cave_accent(globe: &Globe, frame: &Frame, plan: &CavePlan) -> [f32; 3] {
    let world = frame.origin + frame.rot * plan.chamber_center.as_dvec3();
    mineral_rgb(mined_vein(globe.record(), globe.seed(), world, 6.5).unwrap_or(Mineral::Amethyst))
}

/// Spawns 4 to 8 quarryable deep mineral crystal/ore nodes as children of `root`.
pub fn spawn_cave_minerals(
    commands: &mut Commands,
    plan: &CavePlan,
    (globe, frame): (&Globe, &Frame),
    (meshes, material): (&mut Assets<Mesh>, Handle<StandardMaterial>),
    (taken, root): (&Taken, Entity),
) {
    for (idx, (local, depth)) in node_spots(plan).into_iter().enumerate() {
        let id = RockId(child(plan.seed ^ NODE_SALT, idx as u64));
        let left = taken.left(id);
        if left <= 0.0 {
            continue;
        }
        let world = frame.origin + frame.rot * local.as_dvec3();
        let mineral =
            mined_vein(globe.record(), globe.seed(), world, depth).unwrap_or(Mineral::Quartz);
        let base_scale = 0.85 + Rng::new(id.0).unit() as f32 * 0.55;
        let scale = base_scale * left.cbrt();
        let rock = Rock {
            id,
            plant: mineral.plant(),
            scale,
            at: local,
            mid: local + Vec3::Y * (0.55 * scale),
            left,
        };
        let mesh = meshes.add(build_node_mesh(mineral, id.0));
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material.clone()),
            RigidBody::Static,
            Collider::sphere(0.65 * scale),
            CollisionLayers::new(NOT_ANIMALS, LayerMask::ALL),
            CaveMineralNode {
                rock,
                mineral,
                base_scale,
            },
            Transform::from_translation(local).with_scale(Vec3::splat(scale)),
            ChildOf(root),
        ));
    }
}

/// Shrinks or despawns cave mineral nodes whenever a matching [`RockTaken`] blow is recorded.
pub fn sync_taken_nodes(
    mut commands: Commands,
    taken: Res<Taken>,
    mut told: MessageReader<RockTaken>,
    mut nodes: Query<(Entity, &mut CaveMineralNode, &mut Transform)>,
) {
    let ids: Vec<RockId> = told.read().map(|m| m.id).collect();
    if ids.is_empty() {
        return;
    }
    for (entity, mut node, mut tf) in &mut nodes {
        if !ids.contains(&node.rock.id) {
            continue;
        }
        let left = taken.left(node.rock.id);
        if left <= 0.0 {
            commands.entity(entity).try_despawn();
        } else {
            node.rock.left = left;
            node.rock.scale = node.base_scale * left.cbrt();
            tf.scale = Vec3::splat(node.rock.scale);
        }
    }
}
