//! Coherent procedural cave creatures spawned from cave-adapted planet species and bounded to
//! their cavern floor and ceiling via [`CaveBound`].

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::plan::{CaveId, CavePlan};
use crate::fauna::mesh::part_mesh_toned;
use crate::fauna::motion::{Detail, Drive, Motion};
use crate::fauna::{CaveBound, Creature, Part};
use crate::life::{Guild, Life, Species};
use crate::screens::Screen;
use crate::space::Placed;
use crate::universe::hash::{Rng, child};
use crate::world::Terrain;
use crate::world::mesh::Frame;

/// Active procedural cave creature moving within its cavern's [`CaveBound`].
#[derive(Component, Debug, Clone)]
pub struct CaveCritter {
    /// Cave that owns this creature.
    pub cave: CaveId,
    /// Index of its species in the planet's [`Life::species`].
    pub species_index: u16,
    /// Procedural gait and body rig.
    pub motion: Motion,
    /// Spawned child part entities, matching `species.body.parts`.
    pub parts: Vec<Option<Entity>>,
    /// Current wander destination in cave tangent frame.
    pub goal: Vec3,
    /// Seconds remaining until picking a new wander destination.
    pub timer: f32,
    /// Deterministic state seed for wander target selection.
    pub seed: u64,
}

/// Selects a cave-adapted species (`s.cave`), falling back to compact glowing or predator species.
fn pick_species(life: &Life, seed: u64) -> Option<(u16, &Species)> {
    let pick = |pred: fn(&Species) -> bool| {
        let pool: Vec<(u16, &Species)> = life
            .species
            .iter()
            .enumerate()
            .filter(|(_, s)| s.role.guild != Guild::Producer && !s.body.parts.is_empty() && pred(s))
            .map(|(i, s)| (i as u16, s))
            .collect();
        (!pool.is_empty()).then(|| pool[Rng::new(seed).below(pool.len() as u32) as usize])
    };
    pick(|s| s.cave)
        .or_else(|| {
            pick(|s| {
                s.glow.is_some()
                    || matches!(s.role.guild, Guild::Hunter | Guild::Ambusher)
                    || s.body.height < 2.2
            })
        })
        .or_else(|| pick(|_| true))
}

/// Builds the material for a cave creature, adding bioluminescent emissive glow when present.
fn build_skin(
    species: &Species,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    let emissive = species.glow.map_or(LinearRgba::BLACK, |g| {
        let s = (g.strength * 1.6).max(0.35);
        LinearRgba::rgb(g.colour[0] * s, g.colour[1] * s, g.colour[2] * s)
    });
    materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive,
        perceptual_roughness: 0.72,
        ..default()
    })
}

/// Spawns child [`Part`] entities for `species` at its initial `motion` pose under `root`.
fn spawn_parts(
    commands: &mut Commands,
    root: Entity,
    species: &Species,
    motion: &Motion,
    (meshes, skin): (&mut Assets<Mesh>, Handle<StandardMaterial>),
) -> Vec<Option<Entity>> {
    species
        .body
        .parts
        .iter()
        .zip(motion.pose(&species.body))
        .map(|(p, (translation, rotation, scale))| {
            let tissue = species.tissues.get(usize::from(p.tissue))?;
            let accent = species
                .tissues
                .iter()
                .enumerate()
                .find(|(i, _)| *i != usize::from(p.tissue))
                .map(|(_, t)| t);
            let mesh = meshes.add(part_mesh_toned(p, tissue, accent));
            let tf = Transform {
                translation,
                rotation,
                scale,
            };
            Some(
                commands
                    .spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(skin.clone()),
                        tf,
                        Part,
                        ChildOf(root),
                    ))
                    .id(),
            )
        })
        .collect()
}

/// Picks a walkable wander destination inside `bound`.
fn next_goal(bound: &CaveBound, seed: u64) -> Vec3 {
    let mut rng = Rng::new(seed);
    let ang = rng.unit() as f32 * TAU;
    let r = bound.radius * (0.15 + rng.unit() as f32 * 0.72);
    Vec3::new(
        bound.center.x + ang.cos() * r,
        bound.floor_y,
        bound.center.z + ang.sin() * r,
    )
}

/// Spawns one cave creature of `species` inside `bound` on the cave's tangent `frame`.
fn spawn_one(
    commands: &mut Commands,
    (plan, frame, bound): (&CavePlan, &Frame, CaveBound),
    (species_index, species): (u16, &Species),
    (slot, count): (usize, usize),
    (meshes, materials): (&mut Assets<Mesh>, &mut Assets<StandardMaterial>),
) -> Entity {
    let seed = child(plan.seed, 0xCA7E + slot as u64);
    let mut rng = Rng::new(seed);
    let ang = TAU * slot as f32 / count as f32 + rng.unit() as f32 * 0.6;
    let r = bound.radius * (0.2 + rng.unit() as f32 * 0.45);
    let at = Vec3::new(
        bound.center.x + ang.cos() * r,
        bound.floor_y,
        bound.center.z + ang.sin() * r,
    );
    let motion = Motion::new(
        &species.body,
        at,
        Vec3::new((-ang).sin(), 0.0, (-ang).cos()),
        seed,
    );
    let skin = build_skin(species, materials);
    let creature = Creature {
        species: species.id,
        herd: plan.seed ^ 0xCAFE,
        slot: slot as u8,
    };
    let placed = Placed {
        pos: frame.origin,
        rot: frame.rot,
    };
    let root = commands
        .spawn((
            Name::new("CaveCritter"),
            creature,
            bound,
            placed,
            Transform::default(),
            Visibility::default(),
            DespawnOnEnter(Screen::Title),
        ))
        .id();
    let parts = spawn_parts(commands, root, species, &motion, (meshes, skin));
    let critter = CaveCritter {
        cave: plan.id,
        species_index,
        motion,
        parts,
        goal: next_goal(&bound, seed),
        timer: 2.0 + rng.unit() as f32 * 3.0,
        seed,
    };
    commands.entity(root).insert(critter);
    root
}

/// Spawns 1 to 3 coherent cave creatures on the main cavern floor for `plan`.
pub fn spawn_cave_fauna(
    commands: &mut Commands,
    plan: &CavePlan,
    frame: &Frame,
    life: &Life,
    stores: (&mut Assets<Mesh>, &mut Assets<StandardMaterial>),
) -> Vec<Entity> {
    let count = 1 + Rng::new(child(plan.seed, 0xFA07)).below(3) as usize;
    let bound = CaveBound {
        floor_y: plan.chamber_floor_y,
        ceiling_y: plan.chamber_ceiling_y,
        center: plan.chamber_center,
        radius: (plan.chamber_radius - 1.2).max(2.0),
    };
    let (meshes, materials) = stores;
    (0..count)
        .filter_map(|slot| {
            let pick = pick_species(life, child(plan.seed, 0xCA7E + slot as u64))?;
            Some(spawn_one(
                commands,
                (plan, frame, bound),
                pick,
                (slot, count),
                (meshes, materials),
            ))
        })
        .collect()
}

/// Advances one cave creature inside `bound` and updates its child part transforms.
fn step_one(
    critter: &mut CaveCritter,
    bound: &CaveBound,
    species: &Species,
    (gravity, dt): (f32, f32),
    parts: &mut Query<&mut Transform, With<Part>>,
) {
    critter.timer -= dt;
    let to_goal = (critter.goal - critter.motion.root.at).with_y(0.0);
    if critter.timer <= 0.0 || to_goal.length_squared() < 1.2 {
        critter.seed = child(critter.seed, 0x57E9);
        critter.goal = next_goal(bound, critter.seed);
        critter.timer = 2.5 + Rng::new(critter.seed).unit() as f32 * 3.5;
    }
    let vel = to_goal.normalize_or_zero() * (species.history.speed * 0.38).clamp(0.6, 2.8);
    let drive = Drive {
        velocity: vel,
        look: Some(critter.goal + Vec3::Y * 0.5),
        gravity,
        excite: 0.0,
        detail: Detail::Cheap,
    };
    critter
        .motion
        .step(&species.body, &drive, &|_| bound.floor_y, dt);
    let offset = (critter.motion.root.at - bound.center).with_y(0.0);
    if offset.length() > bound.radius {
        let clamped = bound.center + offset.normalize_or_zero() * bound.radius;
        (critter.motion.root.at.x, critter.motion.root.at.z) = (clamped.x, clamped.z);
    }
    critter.motion.root.at.y = critter
        .motion
        .root
        .at
        .y
        .clamp(bound.floor_y, (bound.ceiling_y - 0.5).max(bound.floor_y));
    for (slot, (at, turn, scale)) in critter.parts.iter().zip(critter.motion.pose(&species.body)) {
        if let Some(entity) = slot
            && let Ok(mut tf) = parts.get_mut(*entity)
        {
            (tf.translation, tf.rotation, tf.scale) = (at, turn, scale);
        }
    }
}

/// Steps all active cave creatures inside their [`CaveBound`] and updates child part transforms.
pub fn step_cave_fauna(
    time: Res<Time>,
    terrain: Option<Res<Terrain>>,
    mut critters: Query<(&mut CaveCritter, &CaveBound)>,
    mut parts: Query<&mut Transform, With<Part>>,
) {
    let Some(terrain) = terrain else { return };
    let Some(life) = terrain.globe().life() else {
        return;
    };
    let (dt, gravity) = (
        time.delta_secs().clamp(0.0, 0.05),
        terrain.globe().gravity() as f32,
    );
    for (mut critter, bound) in &mut critters {
        if let Some(species) = life.species.get(usize::from(critter.species_index)) {
            step_one(&mut critter, bound, species, (gravity, dt), &mut parts);
        }
    }
}
